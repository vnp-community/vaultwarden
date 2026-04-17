use crate::db::schema::backup_runs;
use chrono::NaiveDateTime;

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = backup_runs)]
#[diesel(treat_none_as_null = true)]
#[diesel(primary_key(id))]
pub struct BackupRun {
    pub id: String,
    pub started_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub status: String,
    pub backup_type: String,
    pub destination: String,
    pub size_bytes: Option<i64>,
    pub sha256: Option<String>,
    pub manifest_json: Option<String>,
    pub error_message: Option<String>,
    pub verified_at: Option<NaiveDateTime>,
    pub verification_status: Option<String>,
    pub verification_error: Option<String>,
}
