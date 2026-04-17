use crate::util;
use crate::db::schema::approval_requests;
use crate::db::DbConn;
use crate::api::EmptyResult;
use crate::error::MapResult;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = approval_requests)]
#[diesel(primary_key(uuid))]
pub struct ApprovalRequest {
    pub uuid: String,
    pub requester_user_uuid: String,
    pub resource_uuid: String,
    pub state: String, // "pending", "approved", "rejected", "expired"
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
}

impl ApprovalRequest {
    pub fn new(requester_user_uuid: String, resource_uuid: String, state: String) -> Self {
        Self {
            uuid: util::get_uuid(),
            requester_user_uuid,
            resource_uuid,
            state,
            created_at: chrono::Utc::now().naive_utc(),
            expires_at: None,
        }
    }
}

impl ApprovalRequest {
    pub async fn save(&self, conn: &DbConn) -> EmptyResult {
        db_run! { conn:
            sqlite, mysql {
                match diesel::replace_into(approval_requests::table)
                    .values(self)
                    .execute(conn)
                {
                    Ok(_) => Ok(()),
                    Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)) => {
                        diesel::update(approval_requests::table)
                            .filter(approval_requests::uuid.eq(&self.uuid))
                            .set(self)
                            .execute(conn)
                            .map_res("Error saving approval request")
                    }
                    Err(e) => Err(e.into()),
                }.map_res("Error saving approval request")
            }
            postgresql {
                diesel::insert_into(approval_requests::table)
                    .values(self)
                    .on_conflict(approval_requests::uuid)
                    .do_update()
                    .set(self)
                    .execute(conn)
                    .map_res("Error saving approval request")
            }
        }
    }

    pub async fn delete(self, conn: &DbConn) -> EmptyResult {
        db_run! { conn: {
            diesel::delete(approval_requests::table.filter(approval_requests::uuid.eq(self.uuid)))
                .execute(conn)
                .map_res("Error deleting approval request")
        }}
    }

    pub async fn find_by_uuid(uuid: &str, conn: &DbConn) -> Option<Self> {
        db_run! { conn: {
            approval_requests::table
                .filter(approval_requests::uuid.eq(uuid))
                .first::<Self>(conn)
                .ok()
        }}
    }

    pub async fn find_pending_for_resource(resource_uuid: &str, conn: &DbConn) -> Vec<Self> {
        db_run! { conn: {
            approval_requests::table
                .filter(approval_requests::resource_uuid.eq(resource_uuid))
                .filter(approval_requests::state.eq("pending"))
                .load::<Self>(conn)
                .expect("Error loading pending approval requests")
        }}
    }

    pub async fn find_by_requester(requester_user_uuid: &str, conn: &DbConn) -> Vec<Self> {
        db_run! { conn: {
            approval_requests::table
                .filter(approval_requests::requester_user_uuid.eq(requester_user_uuid))
                .load::<Self>(conn)
                .expect("Error loading approval requests for user")
        }}
    }
}
