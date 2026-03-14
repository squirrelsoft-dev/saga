use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaConfig {
    /// Default currency for billing
    pub default_currency: String,
    /// Default hourly rate
    pub default_hourly_rate: Option<f64>,
    /// Whether new entries are billable by default
    pub default_billable: bool,
    /// Daily goal in hours
    pub daily_goal_hours: Option<f64>,
    /// Weekly goal in hours
    pub weekly_goal_hours: Option<f64>,
    /// Tick rate for TUI refresh in milliseconds
    pub tick_rate_ms: u64,
    /// Theme preference (not used yet, placeholder)
    pub theme: String,
    /// Date format for display
    pub date_format: String,
    /// Time format for display (12h or 24h)
    pub time_format: String,
    /// Reminder interval in minutes (0 = disabled)
    pub reminder_interval_mins: u64,
}

impl Default for SagaConfig {
    fn default() -> Self {
        Self {
            default_currency: "USD".to_string(),
            default_hourly_rate: None,
            default_billable: true,
            daily_goal_hours: Some(8.0),
            weekly_goal_hours: Some(40.0),
            tick_rate_ms: 250,
            theme: "default".to_string(),
            date_format: "%Y-%m-%d".to_string(),
            time_format: "24h".to_string(),
            reminder_interval_mins: 0,
        }
    }
}

impl SagaConfig {
    pub fn load() -> Result<Self> {
        let config: SagaConfig = confy::load("saga", "config")?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        confy::store("saga", "config", self)?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        let path = confy::get_configuration_file_path("saga", "config")?;
        Ok(path)
    }

    pub fn data_dir() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("com", "saga", "saga")
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
        let data_dir = dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;
        Ok(data_dir)
    }

    pub fn db_path() -> Result<PathBuf> {
        let mut path = Self::data_dir()?;
        path.push("saga.db");
        Ok(path)
    }
}
