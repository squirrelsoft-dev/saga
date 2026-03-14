use std::collections::HashMap;
use anyhow::Result;
use crate::models::TimeEntry;

pub fn export_entries_csv(
    entries: &[TimeEntry],
    project_names: &HashMap<i64, String>,
) -> Result<String> {
    let filename = format!("saga-export-{}.csv", chrono::Local::now().format("%Y%m%d-%H%M%S"));
    let mut wtr = csv::Writer::from_path(&filename)?;

    wtr.write_record(&["Date", "Start", "End", "Duration (h)", "Project", "Description", "Billable"])?;

    for entry in entries {
        let date = entry.start_time.format("%Y-%m-%d").to_string();
        let start = entry.start_time.format("%H:%M").to_string();
        let end = entry.end_time
            .map(|e| e.format("%H:%M").to_string())
            .unwrap_or_default();
        let duration = entry.duration_secs
            .map(|d| format!("{:.2}", d as f64 / 3600.0))
            .unwrap_or_default();
        let project = project_names.get(&entry.project_id)
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
        let billable = if entry.billable { "Yes" } else { "No" };

        wtr.write_record(&[
            &date,
            &start,
            &end,
            &duration,
            project,
            &entry.description,
            billable,
        ])?;
    }

    wtr.flush()?;
    Ok(filename)
}

pub fn export_entries_csv_to_writer<W: std::io::Write>(
    writer: W,
    entries: &[TimeEntry],
    project_names: &HashMap<i64, String>,
) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(writer);

    wtr.write_record(&["Date", "Start", "End", "Duration (h)", "Project", "Description", "Billable"])?;

    for entry in entries {
        let date = entry.start_time.format("%Y-%m-%d").to_string();
        let start = entry.start_time.format("%H:%M").to_string();
        let end = entry.end_time
            .map(|e| e.format("%H:%M").to_string())
            .unwrap_or_default();
        let duration = entry.duration_secs
            .map(|d| format!("{:.2}", d as f64 / 3600.0))
            .unwrap_or_default();
        let project = project_names.get(&entry.project_id)
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
        let billable = if entry.billable { "Yes" } else { "No" };

        wtr.write_record(&[
            &date,
            &start,
            &end,
            &duration,
            project,
            &entry.description,
            billable,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}
