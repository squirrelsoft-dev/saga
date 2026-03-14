use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Borders, Row, Table, TableState},
};
use crate::app::App;
use crate::ui::theme;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(format!(" Time Entries ({}) ", app.entries.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    if app.entries.is_empty() {
        let p = ratatui::widgets::Paragraph::new(" No entries. Press 'n' to add one.")
            .style(theme::muted_style())
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let rows: Vec<Row> = app.entries.iter().enumerate().map(|(_i, entry)| {
        let proj = app.entry_project_names.get(&entry.project_id)
            .map(|s| s.as_str())
            .unwrap_or("?");
        let date = entry.start_time.format("%Y-%m-%d").to_string();
        let start = entry.start_time.format("%H:%M").to_string();
        let end = entry.end_time.map(|e| e.format("%H:%M").to_string()).unwrap_or("...".to_string());
        let dur = if let Some(d) = entry.duration_secs {
            format_duration(d)
        } else {
            let elapsed = chrono::Local::now().naive_local()
                .signed_duration_since(entry.start_time).num_seconds();
            format!("{}*", format_duration(elapsed))
        };
        let billable = if entry.billable { "$" } else { "" };
        Row::new(vec![
            date,
            start,
            end,
            dur,
            proj.to_string(),
            entry.description.clone(),
            billable.to_string(),
        ])
    }).collect();

    let header = Row::new(vec!["Date", "Start", "End", "Duration", "Project", "Description", "Bill"])
        .style(theme::header_style())
        .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(11),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Min(15),
            Constraint::Length(5),
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(theme::selected_style());

    let mut state = TableState::default();
    state.select(Some(app.entries_selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn format_duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    format!("{}:{:02}", h, m)
}
