use std::fmt;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RateType {
    Project,
    Client,
    Default,
}

impl fmt::Display for RateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RateType::Project => write!(f, "project"),
            RateType::Client => write!(f, "client"),
            RateType::Default => write!(f, "default"),
        }
    }
}

impl RateType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "project" => Some(RateType::Project),
            "client" => Some(RateType::Client),
            "default" => Some(RateType::Default),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rate {
    pub id: i64,
    pub rate_type: RateType,
    pub project_id: Option<i64>,
    pub client_id: Option<i64>,
    pub hourly_rate: f64,
    pub currency: String,
    pub effective_from: String,
    pub created_at: NaiveDateTime,
}
