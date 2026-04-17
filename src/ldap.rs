/// TASK-003-003: LdapConnector — LDAP integration using ldap3 crate
/// TASK-003-004: User provisioning from LDAP
/// TASK-003-005: User deprovisioning from LDAP
/// TASK-003-006: LDAP group → collection mapping sync
/// TASK-003-007: Background LDAP sync job

use std::collections::HashSet;
use std::time::Duration;

use chrono::Utc;
use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry};

use crate::{db::DbPool, error::Error, CONFIG};

/// Core LDAP connector — holds a DB pool for provisioning results.
#[allow(dead_code)]
pub struct LdapConnector {
    pool: DbPool,
}

impl LdapConnector {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Full LDAP sync: connect → bind → search users → provision/deprovision → group sync.
    pub async fn sync(&self) -> Result<(), Error> {
        if !CONFIG.ldap_enabled() {
            return Ok(());
        }

        let host = CONFIG.ldap_host();
        let port = CONFIG.ldap_port();
        let use_tls = CONFIG.ldap_use_tls();
        let bind_dn = CONFIG.ldap_bind_dn();
        let bind_password = CONFIG.ldap_bind_password();

        let scheme = if use_tls { "ldaps" } else { "ldap" };
        let url = format!("{scheme}://{host}:{port}");
        info!("[LDAP] Connecting to {url}");

        let settings = LdapConnSettings::new().set_conn_timeout(Duration::from_secs(10));
        let (conn, mut ldap) = LdapConnAsync::with_settings(settings, &url)
            .await
            .map_err(|e| Error::new("LDAP Connection Failed", e.to_string()))?;

        ldap3::drive!(conn);

        if !bind_dn.is_empty() {
            ldap.simple_bind(&bind_dn, &bind_password)
                .await
                .map_err(|e| {
                    error!("[LDAP] Bind failed: {e:?}");
                    Error::new("LDAP Bind Failed", e.to_string())
                })?;
            debug!("[LDAP] Bound as {bind_dn}");
        }

        let db_conn = match self.pool.get().await {
            Ok(c) => c,
            Err(e) => {
                error!("[LDAP] DB pool error: {e:?}");
                let _u = ldap.unbind().await;
                return Err(Error::new("DB Pool Error", e.to_string()));
            }
        };

        // --- Search users ---
        let base_dn = CONFIG.ldap_base_dn();
        let user_filter = CONFIG.ldap_user_filter();
        let attr_email = CONFIG.ldap_user_attr_email();
        let attr_name = CONFIG.ldap_user_attr_name();
        let attr_uuid = CONFIG.ldap_user_attr_uuid();

        let (entries, _) = ldap
            .search(&base_dn, Scope::Subtree, &user_filter, vec![&attr_email, &attr_name, &attr_uuid])
            .await
            .map_err(|e| Error::new("LDAP Search Failed", e.to_string()))?
            .success()
            .map_err(|e| Error::new("LDAP Search Result", e.to_string()))?;

        let mut ldap_emails: HashSet<String> = HashSet::new();
        let mut provisioned = 0usize;
        let mut updated = 0usize;

        for raw in entries {
            let entry = SearchEntry::construct(raw);
            let email = match entry.attrs.get(&attr_email).and_then(|v| v.first()).map(|s| s.to_lowercase()) {
                Some(e) if !e.is_empty() => e,
                _ => {
                    warn!("[LDAP] Skipping entry without email: {}", entry.dn);
                    continue;
                }
            };
            let name = entry.attrs.get(&attr_name).and_then(|v| v.first()).cloned().unwrap_or_default();
            let ext_id = entry.attrs.get(&attr_uuid).and_then(|v| v.first()).cloned().unwrap_or_else(|| entry.dn.clone());

            ldap_emails.insert(email.clone());

            match self.provision_or_update_user(&email, &name, &ext_id, &db_conn).await {
                Ok(true)  => provisioned += 1,
                Ok(false) => updated += 1,
                Err(e)    => error!("[LDAP] Failed to provision {email}: {e:?}"),
            }
        }

        // --- Deprovision removed users ---
        if let Err(e) = self.deprovision_removed_users(&ldap_emails, &db_conn).await {
            error!("[LDAP] Deprovisioning error: {e:?}");
        }

        // --- Group sync ---
        if let Err(e) = self.sync_group_memberships(&mut ldap, &db_conn).await {
            error!("[LDAP] Group sync error: {e:?}");
        }

        info!("[LDAP] Sync done — provisioned={provisioned}, updated={updated}, ldap_users={}", ldap_emails.len());
        let _u = ldap.unbind().await;
        Ok(())
    }

    // ─── TASK-003-004: Provision / update ────────────────────────────────────

    async fn provision_or_update_user(
        &self,
        email: &str,
        name: &str,
        _ext_id: &str,
        conn: &crate::db::DbConn,
    ) -> Result<bool, Error> {
        use crate::db::models::User;

        if let Some(mut existing) = User::find_by_mail(email, conn).await {
            if !name.is_empty() && existing.name != name {
                existing.name = name.to_string();
                existing.save(conn).await?;
            }
            return Ok(false);
        }

        info!("[LDAP] Provisioning user: {email}");
        let mut user = User::new(email, Some(name.to_string()));
        // Set a random unusable password — user must reset via org invite
        user.set_password(&crate::util::get_uuid(), None, true, None);
        user.provisioning_source = Some("ldap".to_string());
        user.save(conn).await?;

        // Optionally add to configured org
        let org_uuid_str = CONFIG.ldap_sync_org_uuid();
        if !org_uuid_str.is_empty() {
            use crate::db::models::{Membership, MembershipStatus, MembershipType, Organization, OrganizationId, UserId};
            let org_id = OrganizationId::from(org_uuid_str);
            if Organization::find_by_uuid(&org_id, conn).await.is_some() {
                let user_id = UserId::from(user.uuid.to_string());
                let mut membership = Membership::new(user_id, org_id, None);
                membership.atype = MembershipType::User as i32;
                membership.status = MembershipStatus::Invited as i32;
                let _r = membership.save(conn).await;
            }
        }

        Ok(true)
    }

    // ─── TASK-003-005: Deprovision ────────────────────────────────────────────

    async fn deprovision_removed_users(
        &self,
        ldap_emails: &HashSet<String>,
        conn: &crate::db::DbConn,
    ) -> Result<(), Error> {
        use crate::db::models::{Device, User};

        let all_users = User::get_all(conn).await;
        let grace_days = i64::from(CONFIG.ldap_deprovision_grace_days());

        for (user, _sso) in all_users {
            if user.provisioning_source.as_deref() != Some("ldap") {
                continue;
            }
            let email = user.email.to_lowercase();
            if ldap_emails.contains(&email) {
                continue; // Still in LDAP, do nothing
            }

            if user.suspension_scheduled_at.is_none() {
                info!("[LDAP] Scheduling suspension for {email} in {grace_days} days");
                let deadline = Utc::now() + chrono::TimeDelta::try_days(grace_days).unwrap_or_default();
                let mut u = user;
                u.suspension_scheduled_at = Some(deadline.naive_utc());
                if let Err(e) = u.save(conn).await {
                    error!("[LDAP] Failed to schedule suspension for {email}: {e:?}");
                }
            } else if let Some(sched) = user.suspension_scheduled_at {
                if sched <= Utc::now().naive_utc() {
                    info!("[LDAP] Suspending overdue user {email}");
                    let user_id = user.uuid.clone();
                    if let Err(e) = Device::delete_all_by_user(&user_id, conn).await {
                        warn!("[LDAP] Failed to revoke sessions for {email}: {e:?}");
                    }
                }
            }
        }

        Ok(())
    }

    // ─── TASK-003-006: Group → collection sync ────────────────────────────────

    async fn sync_group_memberships(
        &self,
        ldap: &mut ldap3::Ldap,
        conn: &crate::db::DbConn,
    ) -> Result<(), Error> {
        let base_dn = CONFIG.ldap_base_dn();
        let group_filter = CONFIG.ldap_group_filter();
        let attr_member = CONFIG.ldap_group_attr_member();
        let attr_email = CONFIG.ldap_user_attr_email();

        if base_dn.is_empty() {
            debug!("[LDAP] base_dn not configured, skipping group sync");
            return Ok(());
        }

        let mappings = self.load_group_mappings(conn).await;

        for (group_dn, collection_uuid, _org_uuid) in &mappings {
            let group_result = ldap
                .search(group_dn, Scope::Base, &group_filter, vec![&attr_member])
                .await;

            let (entries, _) = match group_result.and_then(|r| r.success().map_err(Into::into)) {
                Ok(e) => e,
                Err(e) => {
                    warn!("[LDAP] Group search failed for {group_dn}: {e:?}");
                    continue;
                }
            };

            let mut member_emails: HashSet<String> = HashSet::new();

            for raw in entries {
                let entry = SearchEntry::construct(raw);
                let member_dns = entry.attrs.get(&attr_member).cloned().unwrap_or_default();

                for member_dn in &member_dns {
                    if let Ok(r) = ldap.search(member_dn, Scope::Base, "(objectClass=*)", vec![&attr_email]).await {
                        if let Ok((me_entries, _)) = r.success() {
                            for me_raw in me_entries {
                                let me = SearchEntry::construct(me_raw);
                                if let Some(emails) = me.attrs.get(&attr_email) {
                                    if let Some(email) = emails.first() {
                                        member_emails.insert(email.to_lowercase());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            self.sync_collection_membership(collection_uuid, &member_emails, conn).await;
        }

        Ok(())
    }

    async fn load_group_mappings(&self, conn: &crate::db::DbConn) -> Vec<(String, String, String)> {
        use crate::db::schema::ldap_group_mappings;
        use diesel::prelude::*;

        db_run! { conn: {
            ldap_group_mappings::table
                .select((ldap_group_mappings::ldap_group_dn, ldap_group_mappings::collection_uuid, ldap_group_mappings::org_uuid))
                .load::<(String, String, String)>(conn)
                .unwrap_or_default()
        }}
    }

    async fn sync_collection_membership(
        &self,
        collection_uuid_str: &str,
        member_emails: &HashSet<String>,
        conn: &crate::db::DbConn,
    ) {
        use crate::db::models::{Collection, CollectionId, CollectionUser, User};

        let coll_id = CollectionId::from(collection_uuid_str.to_string());
        if Collection::find_by_uuid(&coll_id, conn).await.is_none() {
            warn!("[LDAP] Collection {collection_uuid_str} not found");
            return;
        }

        for email in member_emails {
            if let Some(user) = User::find_by_mail(email, conn).await {
                let existing = CollectionUser::find_by_collection_and_user(&coll_id, &user.uuid, conn).await;
                if existing.is_none() {
                    info!("[LDAP] Adding {email} to collection {collection_uuid_str}");
                    if let Err(e) = CollectionUser::save(&user.uuid, &coll_id, false, false, false, conn).await {
                        warn!("[LDAP] Failed to add {email} to collection: {e:?}");
                    }
                }
            }
        }
    }
}

// ─── TASK-003-007: Background job ────────────────────────────────────────────

pub async fn ldap_sync_job(pool: DbPool) {
    if !CONFIG.ldap_enabled() {
        return;
    }
    info!("[LDAP Task] Starting sync at {}", Utc::now());
    let connector = LdapConnector::new(pool);
    if let Err(e) = connector.sync().await {
        error!("[LDAP Task] Sync failed: {e:?}");
    }
}
