table! {
    attachments (id) {
        id -> Text,
        cipher_uuid -> Text,
        file_name -> Text,
        file_size -> BigInt,
        akey -> Nullable<Text>,
    }
}

table! {
    ciphers (uuid) {
        uuid -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        user_uuid -> Nullable<Text>,
        organization_uuid -> Nullable<Text>,
        key -> Nullable<Text>,
        atype -> Integer,
        name -> Text,
        notes -> Nullable<Text>,
        fields -> Nullable<Text>,
        data -> Text,
        password_history -> Nullable<Text>,
        deleted_at -> Nullable<Timestamp>,
        reprompt -> Nullable<Integer>,
        is_privileged -> Bool,
        privileged_config_uuid -> Nullable<Text>,
        is_secret -> Bool,
        secret_project -> Nullable<Text>,
    }
}

table! {
    ciphers_collections (cipher_uuid, collection_uuid) {
        cipher_uuid -> Text,
        collection_uuid -> Text,
    }
}

table! {
    collections (uuid) {
        uuid -> Text,
        org_uuid -> Text,
        name -> Text,
        external_id -> Nullable<Text>,
    }
}

table! {
    devices (uuid, user_uuid) {
        uuid -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        user_uuid -> Text,
        name -> Text,
        atype -> Integer,
        push_uuid -> Nullable<Text>,
        push_token -> Nullable<Text>,
        refresh_token -> Text,
        twofactor_remember -> Nullable<Text>,
        is_trusted -> Bool,
        mdm_enrolled -> Bool,
        mdm_compliant -> Bool,
        mdm_last_check_at -> Nullable<Timestamp>,
        cert_subject -> Nullable<Text>,
        cert_serial -> Nullable<Text>,
        cert_expires_at -> Nullable<Timestamp>,
        cert_issuer -> Nullable<Text>,
    }
}

table! {
    device_trust_policies (uuid) {
        uuid -> Text,
        org_uuid -> Text,
        require_device_cert -> Bool,
        require_managed_device -> Bool,
        allowed_cert_issuers -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    mdm_compliance_cache (device_uuid) {
        device_uuid -> Text,
        org_uuid -> Text,
        is_compliant -> Bool,
        raw_status -> Nullable<Text>,
        checked_at -> Timestamp,
    }
}

table! {
    event (uuid) {
        uuid -> Text,
        event_type -> Integer,
        user_uuid -> Nullable<Text>,
        org_uuid -> Nullable<Text>,
        cipher_uuid -> Nullable<Text>,
        collection_uuid -> Nullable<Text>,
        group_uuid -> Nullable<Text>,
        org_user_uuid -> Nullable<Text>,
        act_user_uuid -> Nullable<Text>,
        device_type -> Nullable<Integer>,
        ip_address -> Nullable<Text>,
        event_date -> Timestamp,
        policy_uuid -> Nullable<Text>,
        provider_uuid -> Nullable<Text>,
        provider_user_uuid -> Nullable<Text>,
        provider_org_uuid -> Nullable<Text>,
        tenant_uuid -> Text,
    }
}

table! {
    favorites (user_uuid, cipher_uuid) {
        user_uuid -> Text,
        cipher_uuid -> Text,
    }
}

table! {
    audit_entries (id) {
        id -> Integer,
        timestamp -> Timestamp,
        event_type -> Text,
        severity -> Text,
        actor_user_uuid -> Nullable<Text>,
        actor_email -> Nullable<Text>,
        target_resource -> Nullable<Text>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        org_uuid -> Nullable<Text>,
        metadata -> Nullable<Text>,
        prev_hash -> Nullable<Binary>,
        entry_hash -> Binary,
        siem_delivered -> Bool,
        siem_attempts -> Integer,
        tenant_uuid -> Text,
    }
}

table! {
    audit_entries_archive (id) {
        id -> Integer,
        timestamp -> Timestamp,
        event_type -> Text,
        severity -> Text,
        actor_user_uuid -> Nullable<Text>,
        actor_email -> Nullable<Text>,
        target_resource -> Nullable<Text>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        org_uuid -> Nullable<Text>,
        metadata -> Nullable<Text>,
        prev_hash -> Nullable<Binary>,
        entry_hash -> Binary,
    }
}

table! {
    access_reviews (id) {
        id -> Integer,
        org_uuid -> Text,
        created_at -> Timestamp,
        deadline_at -> Timestamp,
        status -> Text,
    }
}

table! {
    access_review_items (id) {
        id -> Integer,
        access_review_id -> Integer,
        collection_uuid -> Text,
        user_uuid -> Text,
        reviewed_by -> Nullable<Text>,
        reviewed_at -> Nullable<Timestamp>,
        decision -> Nullable<Text>,
    }
}

table! {
    folders (uuid) {
        uuid -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        user_uuid -> Text,
        name -> Text,
    }
}

table! {
    folders_ciphers (cipher_uuid, folder_uuid) {
        cipher_uuid -> Text,
        folder_uuid -> Text,
    }
}

table! {
    invitations (email) {
        email -> Text,
    }
}

table! {
    org_policies (uuid) {
        uuid -> Text,
        org_uuid -> Text,
        atype -> Integer,
        enabled -> Bool,
        data -> Text,
    }
}

table! {
    ldap_sync_state (id) {
        id -> Integer,
        last_sync_at -> Timestamp,
        status -> Text,
        users_synced -> Integer,
        groups_synced -> Integer,
        error_message -> Nullable<Text>,
    }
}

table! {
    ldap_group_mappings (id) {
        id -> Integer,
        ldap_group_dn -> Text,
        collection_uuid -> Text,
        org_uuid -> Text,
    }
}

table! {
    organizations (uuid) {
        uuid -> Text,
        name -> Text,
        billing_email -> Text,
        private_key -> Nullable<Text>,
        public_key -> Nullable<Text>,
        tenant_uuid -> Text,
    }
}

table! {
    scim_tokens (id) {
        id -> Integer,
        token_hash -> Binary,
        org_uuid -> Text,
        created_at -> Timestamp,
        last_used_at -> Nullable<Timestamp>,
    }
}

table! {
    sends (uuid) {
        uuid -> Text,
        user_uuid -> Nullable<Text>,
        organization_uuid -> Nullable<Text>,
        name -> Text,
        notes -> Nullable<Text>,
        atype -> Integer,
        data -> Text,
        akey -> Text,
        password_hash -> Nullable<Binary>,
        password_salt -> Nullable<Binary>,
        password_iter -> Nullable<Integer>,
        max_access_count -> Nullable<Integer>,
        access_count -> Integer,
        creation_date -> Timestamp,
        revision_date -> Timestamp,
        expiration_date -> Nullable<Timestamp>,
        deletion_date -> Timestamp,
        disabled -> Bool,
        hide_email -> Nullable<Bool>,
    }
}

table! {
    twofactor (uuid) {
        uuid -> Text,
        user_uuid -> Text,
        atype -> Integer,
        enabled -> Bool,
        data -> Text,
        last_used -> BigInt,
    }
}

table! {
    twofactor_incomplete (user_uuid, device_uuid) {
        user_uuid -> Text,
        device_uuid -> Text,
        device_name -> Text,
        device_type -> Integer,
        login_time -> Timestamp,
        ip_address -> Text,
    }
}

table! {
    twofactor_duo_ctx (state) {
        state -> Text,
        user_email -> Text,
        nonce -> Text,
        exp -> BigInt,
    }
}

table! {
    users (uuid) {
        uuid -> Text,
        enabled -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        verified_at -> Nullable<Timestamp>,
        last_verifying_at -> Nullable<Timestamp>,
        login_verify_count -> Integer,
        email -> Text,
        email_new -> Nullable<Text>,
        email_new_token -> Nullable<Text>,
        name -> Text,
        password_hash -> Binary,
        salt -> Binary,
        password_iterations -> Integer,
        password_hint -> Nullable<Text>,
        akey -> Text,
        private_key -> Nullable<Text>,
        public_key -> Nullable<Text>,
        tenant_uuid -> Text,
        totp_secret -> Nullable<Text>,
        totp_recover -> Nullable<Text>,
        security_stamp -> Text,
        stamp_exception -> Nullable<Text>,
        equivalent_domains -> Text,
        excluded_globals -> Text,
        client_kdf_type -> Integer,
        client_kdf_iter -> Integer,
        client_kdf_memory -> Nullable<Integer>,
        client_kdf_parallelism -> Nullable<Integer>,
        api_key -> Nullable<Text>,
        avatar_color -> Nullable<Text>,
        external_id -> Nullable<Text>,
        pii_erasure_scheduled_at -> Nullable<Timestamp>,
        pii_erased_at -> Nullable<Timestamp>,
        provisioning_source -> Nullable<Text>,
        provisioning_external_id -> Nullable<Text>,
        suspension_scheduled_at -> Nullable<Timestamp>,
    }
}

table! {
    users_collections (user_uuid, collection_uuid) {
        user_uuid -> Text,
        collection_uuid -> Text,
        read_only -> Bool,
        hide_passwords -> Bool,
        manage -> Bool,
    }
}

table! {
    users_organizations (uuid) {
        uuid -> Text,
        user_uuid -> Text,
        org_uuid -> Text,
        invited_by_email -> Nullable<Text>,
        access_all -> Bool,
        akey -> Text,
        status -> Integer,
        atype -> Integer,
        reset_password_key -> Nullable<Text>,
        external_id -> Nullable<Text>,
        custom_role_uuid -> Nullable<Text>,
    }
}

table! {
    organization_api_key (uuid, org_uuid) {
        uuid -> Text,
        org_uuid -> Text,
        atype -> Integer,
        api_key -> Text,
        revision_date -> Timestamp,
    }
}

table! {
    sso_nonce (state) {
        state -> Text,
        nonce -> Text,
        verifier -> Nullable<Text>,
        redirect_uri -> Text,
        created_at -> Timestamp,
    }
}

table! {
    sso_users (user_uuid) {
        user_uuid -> Text,
        identifier -> Text,
    }
}

table! {
    emergency_access (uuid) {
        uuid -> Text,
        grantor_uuid -> Text,
        grantee_uuid -> Nullable<Text>,
        email -> Nullable<Text>,
        key_encrypted -> Nullable<Text>,
        atype -> Integer,
        status -> Integer,
        wait_time_days -> Integer,
        recovery_initiated_at -> Nullable<Timestamp>,
        last_notification_at -> Nullable<Timestamp>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

table! {
    groups (uuid) {
        uuid -> Text,
        organizations_uuid -> Text,
        name -> Text,
        access_all -> Bool,
        external_id -> Nullable<Text>,
        creation_date -> Timestamp,
        revision_date -> Timestamp,
    }
}

table! {
    groups_users (groups_uuid, users_organizations_uuid) {
        groups_uuid -> Text,
        users_organizations_uuid -> Text,
    }
}

table! {
    collections_groups (collections_uuid, groups_uuid) {
        collections_uuid -> Text,
        groups_uuid -> Text,
        read_only -> Bool,
        hide_passwords -> Bool,
        manage -> Bool,
    }
}

table! {
    auth_requests  (uuid) {
        uuid -> Text,
        user_uuid -> Text,
        organization_uuid -> Nullable<Text>,
        request_device_identifier -> Text,
        device_type -> Integer,
        request_ip -> Text,
        response_device_id -> Nullable<Text>,
        access_code -> Text,
        public_key -> Text,
        enc_key -> Nullable<Text>,
        master_password_hash -> Nullable<Text>,
        approved -> Nullable<Bool>,
        creation_date -> Timestamp,
        response_date -> Nullable<Timestamp>,
        authentication_date -> Nullable<Timestamp>,
    }
}

// TASK-SEC-HIGH-02-D: Revoked JWT tokens table (opt-in).
// Only queried when TOKEN_REVOCATION_ENABLED=true.
table! {
    revoked_tokens (jti) {
        jti -> Text,
        user_uuid -> Text,
        revoked_at -> Timestamp,
        expires_at -> Timestamp,
    }
}

// TASK-001-006: GDPR erasure log table for SOL-001 compliance framework.
// Append-only audit chain; each entry hashes the previous for tamper evidence.
table! {
    erasure_logs (uuid) {
        uuid         -> Text,
        user_uuid    -> Text,
        requested_at -> Timestamp,
        scheduled_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
        requestor_ip -> Text,
        prev_hash    -> Text,
        entry_hash   -> Text,
    }
}

joinable!(access_review_items -> access_reviews (access_review_id));
joinable!(attachments -> ciphers (cipher_uuid));
joinable!(ciphers -> organizations (organization_uuid));
joinable!(ciphers -> users (user_uuid));
joinable!(ciphers_collections -> ciphers (cipher_uuid));
joinable!(ciphers_collections -> collections (collection_uuid));
joinable!(collections -> organizations (org_uuid));
joinable!(devices -> users (user_uuid));
joinable!(folders -> users (user_uuid));
joinable!(folders_ciphers -> ciphers (cipher_uuid));
joinable!(folders_ciphers -> folders (folder_uuid));
joinable!(org_policies -> organizations (org_uuid));
joinable!(sends -> organizations (organization_uuid));
joinable!(sends -> users (user_uuid));
joinable!(twofactor -> users (user_uuid));
joinable!(users_collections -> collections (collection_uuid));
joinable!(users_collections -> users (user_uuid));
joinable!(users_organizations -> organizations (org_uuid));
joinable!(users_organizations -> users (user_uuid));
joinable!(users_organizations -> ciphers (org_uuid));
joinable!(organization_api_key -> organizations (org_uuid));
joinable!(emergency_access -> users (grantor_uuid));
joinable!(groups -> organizations (organizations_uuid));
joinable!(groups_users -> users_organizations (users_organizations_uuid));
joinable!(groups_users -> groups (groups_uuid));
joinable!(collections_groups -> collections (collections_uuid));
joinable!(collections_groups -> groups (groups_uuid));
joinable!(event -> users_organizations (uuid));
table! {
    custom_roles (uuid) {
        uuid -> Text,
        org_uuid -> Text,
        name -> Text,
        permissions -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    access_schedules (uuid) {
        uuid -> Text,
        org_uuid -> Nullable<Text>,
        user_uuid -> Nullable<Text>,
        timezone -> Text,
        allowed_days -> Integer,
        allowed_time_from -> Nullable<Time>,
        allowed_time_until -> Nullable<Time>,
    }
}

table! {
    ip_allowlists (uuid) {
        uuid -> Text,
        org_uuid -> Nullable<Text>,
        cidr_ranges -> Text,
    }
}

table! {
    backup_runs (id) {
        id -> Text,
        started_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
        status -> Text,
        backup_type -> Text,
        destination -> Text,
        size_bytes -> Nullable<BigInt>,
        sha256 -> Nullable<Text>,
        manifest_json -> Nullable<Text>,
        error_message -> Nullable<Text>,
        verified_at -> Nullable<Timestamp>,
        verification_status -> Nullable<Text>,
        verification_error -> Nullable<Text>,
    }
}

table! {
    privileged_configs (uuid) {
        uuid -> Text,
        cipher_uuid -> Text,
        requires_approval -> Bool,
        max_checkout_duration -> Nullable<Integer>,
        auto_rotate_after_checkout -> Bool,
        rotation_target_type -> Nullable<Text>,
        rotation_target_config -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    checkouts (uuid) {
        uuid -> Text,
        cipher_uuid -> Text,
        user_uuid -> Text,
        justification -> Text,
        itsm_ticket -> Nullable<Text>,
        approval_request_uuid -> Nullable<Text>,
        checked_out_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
        checked_in_at -> Nullable<Timestamp>,
        access_count -> Integer,
        status -> Text,
        rotation_triggered -> Bool,
    }
}

table! {
    rotation_history (uuid) {
        uuid -> Text,
        cipher_uuid -> Text,
        checkout_uuid -> Nullable<Text>,
        started_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
        status -> Text,
        error_message -> Nullable<Text>,
    }
}

table! {
    break_glass_configs (uuid) {
        uuid -> Text,
        user_uuid -> Text,
        witness_uuids -> Text,
        notification_emails -> Text,
        session_duration_hours -> Integer,
    }
}

table! {
    approval_requests (uuid) {
        uuid -> Text,
        requester_user_uuid -> Text,
        resource_uuid -> Text,
        state -> Text,
        created_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
    }
}

table! {
    sod_rules (uuid) {
        uuid -> Text,
        org_uuid -> Text,
        role_a_uuid -> Text,
        role_b_uuid -> Text,
        enforcement -> Text,
    }
}

table! {
    api_keys_v2 (uuid) {
        uuid -> Text,
        org_uuid -> Text,
        client_id -> Text,
        secret_hash -> Text,
        name -> Text,
        scopes -> Text,
        allowed_ips -> Nullable<Text>,
        rate_limit_minute -> Nullable<Integer>,
        expires_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        last_used_at -> Nullable<Timestamp>,
        is_active -> Bool,
    }
}

table! {
    api_key_usage (id) {
        id -> Text,
        api_key_uuid -> Text,
        endpoint -> Text,
        method -> Text,
        status_code -> Integer,
        response_ms -> Integer,
        timestamp -> Timestamp,
    }
}

table! {
    webhooks (uuid) {
        uuid -> Text,
        org_uuid -> Text,
        name -> Text,
        url -> Text,
        secret_hash -> Text,
        events -> Text,
        is_active -> Bool,
        retry_count -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    webhook_deliveries (uuid) {
        uuid -> Text,
        webhook_uuid -> Text,
        event_type -> Text,
        payload -> Text,
        status -> Text,
        attempt_count -> Integer,
        last_attempt_at -> Nullable<Timestamp>,
        next_attempt_at -> Nullable<Timestamp>,
        error_message -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

table! {
    tenants (uuid) {
        uuid -> Text,
        name -> Text,
        slug -> Text,
        domain_restriction -> Nullable<Text>,
        is_active -> Bool,
        max_users -> Nullable<Integer>,
        max_organizations -> Nullable<Integer>,
        max_vault_items -> Nullable<Integer>,
        max_storage_bytes -> Nullable<BigInt>,
        config_overrides -> Nullable<Text>,
        branding -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    tenant_admins (tenant_uuid, user_uuid) {
        tenant_uuid -> Text,
        user_uuid -> Text,
        created_at -> Timestamp,
    }
}

joinable!(auth_requests -> users (user_uuid));
joinable!(sso_users -> users (user_uuid));
joinable!(revoked_tokens -> users (user_uuid));
joinable!(users_organizations -> custom_roles (custom_role_uuid));
joinable!(custom_roles -> organizations (org_uuid));
joinable!(access_schedules -> organizations (org_uuid));
joinable!(access_schedules -> users (user_uuid));
joinable!(ip_allowlists -> organizations (org_uuid));
joinable!(break_glass_configs -> users (user_uuid));
joinable!(approval_requests -> users (requester_user_uuid));
joinable!(sod_rules -> organizations (org_uuid));
joinable!(privileged_configs -> ciphers (cipher_uuid));
joinable!(checkouts -> ciphers (cipher_uuid));
joinable!(checkouts -> users (user_uuid));
joinable!(rotation_history -> ciphers (cipher_uuid));
joinable!(api_keys_v2 -> organizations (org_uuid));
joinable!(api_key_usage -> api_keys_v2 (api_key_uuid));
joinable!(webhooks -> organizations (org_uuid));
joinable!(webhook_deliveries -> webhooks (webhook_uuid));
joinable!(device_trust_policies -> organizations (org_uuid));
joinable!(mdm_compliance_cache -> organizations (org_uuid));
joinable!(tenant_admins -> tenants (tenant_uuid));
joinable!(tenant_admins -> users (user_uuid));
joinable!(users -> tenants (tenant_uuid));
joinable!(organizations -> tenants (tenant_uuid));
joinable!(audit_entries -> tenants (tenant_uuid));

allow_tables_to_appear_in_same_query!(
    attachments,
    ciphers,
    ciphers_collections,
    collections,
    devices,
    folders,
    folders_ciphers,
    invitations,
    org_policies,
    organizations,
    sends,
    sso_users,
    twofactor,
    users,
    users_collections,
    users_organizations,
    organization_api_key,
    emergency_access,
    groups,
    groups_users,
    collections_groups,
    event,
    auth_requests,
    revoked_tokens,
    erasure_logs,
    custom_roles,
    access_schedules,
    ip_allowlists,
    break_glass_configs,
    approval_requests,
    sod_rules,
    backup_runs,
    privileged_configs,
    checkouts,
    rotation_history,
    api_keys_v2,
    api_key_usage,
    webhooks,
    webhook_deliveries,
    device_trust_policies,
    mdm_compliance_cache,
    tenants,
    tenant_admins,
);
