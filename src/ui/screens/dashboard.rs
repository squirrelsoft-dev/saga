use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BarChart, Paragraph, Row, Table},
};
use crate::app::App;
use crate::ui::theme;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Active timer summary
            Constraint::Min(8),    // Two panels: today entries + weekly chart
        ])
        .split(area);

    // Active timer section
    render_timer_summary(app, frame, chunks[0]);

    // Bottom section: today's entries and weekly chart
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);

    render_today_entries(app, frame, bottom[0]);
    render_weekly_chart(app, frame, bottom[1]);
}

fn render_timer_summary(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Active Timer ")
        .borders(Borders::ALL)
        .border_style(if app.active_entry.is_some() {
            Style::default().fg(theme::SUCCESS)
        } else {
            Style::default().fg(theme::MUTED)
        });

    if let Some(ref _entry) = app.active_entry {
        let project = app.active_project_name.as_deref().unwrap_or("Unknown");
        let hours = app.timer_seconds / 3600;
        let mins = (app.timer_seconds % 3600) / 60;
        let secs = app.timer_seconds % 60;

        let text = vec![
            Line::from(vec![
                Span::styled(format!(" {} ", project), theme::active_timer_style()),
                Span::raw("  "),
                Span::styled(
                    format!("{:02}:{:02}:{:02}", hours, mins, secs),
                    Style::default().fg(theme::FG).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                " Press 's' to stop",
                theme::muted_style(),
            )),
        ];
        let p = Paragraph::new(text).block(block);
        frame.render_widget(p, area);
    } else {
        let text = vec![
            Line::from(Span::styled(" No timer running", theme::muted_style())),
            Line::from(Span::styled(" Press 's' to start", theme::muted_style())),
        ];
        let p = Paragraph::new(text).block(block);
        frame.render_widget(p, area);
    }
}

fn render_today_entries(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(format!(" Today ({}) ", format_duration_short(app.today_total_secs)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    if app.today_entries.is_empty() {
        let p = Paragraph::new(" No entries today")
            .style(theme::muted_style())
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let rows: Vec<Row> = app.today_entries.iter().map(|entry| {
        let proj = app.entry_project_names.get(&entry.project_id)
            .map(|s| s.as_str())
            .unwrap_or("?");
        let start = entry.start_time.format("%H:%M").to_string();
        let end = entry.end_time.map(|e| e.format("%H:%M").to_string()).unwrap_or("...".to_string());
        let dur = if let Some(d) = entry.duration_secs {
            format_duration_short(d)
        } else {
            let elapsed = chrono::Local::now().naive_local()
                .signed_duration_since(entry.start_time).num_seconds();
            format!("{}*", format_duration_short(elapsed))
        };
        Row::new(vec![
            start,
            end,
            dur,
            proj.to_string(),
            entry.description.clone(),
        ])
    }).collect();

    let header = Row::new(vec!["Start", "End", "Dur", "Project", "Description"])
        .style(theme::header_style());

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(7),
            Constraint::Length(12),
            Constraint::Min(10),
        ]
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}

fn render_weekly_chart(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" This Week ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    if app.week_daily_totals.is_empty() {
        let p = Paragraph::new(" No data").style(theme::muted_style()).block(block);
        frame.render_widget(p, area);
        return;
    }

    let data: Vec<(&str, u64)> = app.week_daily_totals.iter()
        .map(|(label, secs)| (label.as_str(), (*secs as u64) / 60)) // Show in minutes
        .collect();

    let chart = BarChart::default()
        .block(block)
        .data(&data)
        .bar_width(5)
        .bar_gap(2)
        .bar_style(Style::default().fg(theme::PRIMARY))
        .value_style(Style::default().fg(theme::FG));

    frame.render_widget(chart, area);
}

fn format_duration_short(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{}h{:02}m", h, m)
    } else {
        format!("{}m", m)
    }
}
