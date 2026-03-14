use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: i64,
    pub uuid: String,
    pub project_id: i64,
    pub description: String,
    pub start_time: NaiveDateTime,
    pub end_time: Option<NaiveDateTime>,
    pub duration_secs: Option<i64>,
    pub billable: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTimeEntry {
    pub project_id: i64,
    pub description: String,
    pub start_time: NaiveDateTime,
    pub end_time: Option<NaiveDateTime>,
    pub billable: bool,
    pub tags: Vec<String>,
}
