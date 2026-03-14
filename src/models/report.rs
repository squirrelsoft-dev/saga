use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: String,
    pub total_seconds: i64,
    pub entry_count: i64,
    pub billable_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBreakdown {
    pub project_id: i64,
    pub project_name: String,
    pub project_color: String,
    pub total_seconds: i64,
    pub billable_seconds: i64,
    pub entry_count: i64,
    pub amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyRetrospective {
    pub week_start: String,
    pub week_end: String,
    pub daily_totals: Vec<DailySummary>,
    pub project_breakdowns: Vec<ProjectBreakdown>,
    pub total_seconds: i64,
    pub billable_seconds: i64,
}
