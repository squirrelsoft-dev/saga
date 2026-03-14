use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BarChart, Paragraph, Row, Table},
};
use crate::app::App;
use crate::app::state::ReportPeriod;
use crate::ui::theme;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Period selector + summary
            Constraint::Min(8),    // Charts area
            Constraint::Length(8), // Project breakdown table
        ])
        .split(area);

    render_period_header(app, frame, chunks[0]);

    let chart_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(chunks[1]);

    render_daily_chart(app, frame, chart_area[0]);
    render_summary(app, frame, chart_area[1]);
    render_project_table(app, frame, chunks[2]);
}

fn render_period_header(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    let total_h = app.report_total_secs as f64 / 3600.0;
    let billable_h = app.report_billable_secs as f64 / 3600.0;

    let text = Line::from(vec![
        Span::styled(" < ", Style::default().fg(theme::ACCENT)),
        Span::styled(app.report_period.label(), theme::title_style()),
        Span::styled(" > ", Style::default().fg(theme::ACCENT)),
        Span::raw(format!("  Total: {:.1}h  Billable: {:.1}h", total_h, billable_h)),
        Span::styled("  [c]sv [p]df", theme::muted_style()),
    ]);

    let p = Paragraph::new(text).block(block);
    frame.render_widget(p, area);
}

fn render_daily_chart(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Hours by Day ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    if app.report_daily_summaries.is_empty() {
        let p = Paragraph::new(" No data").style(theme::muted_style()).block(block);
        frame.render_widget(p, area);
        return;
    }

    let data: Vec<(String, u64)> = app.report_daily_summaries.iter().map(|s| {
        let label = if s.date.len() >= 10 {
            s.date[5..10].to_string()
        } else {
            s.date.clone()
        };
        (label, (s.total_seconds / 60) as u64)
    }).collect();

    let data_refs: Vec<(&str, u64)> = data.iter().map(|(l, v)| (l.as_str(), *v)).collect();

    let chart = BarChart::default()
        .block(block)
        .data(&data_refs)
        .bar_width(5)
        .bar_gap(1)
        .bar_style(Style::default().fg(theme::PRIMARY))
        .value_style(Style::default().fg(theme::FG));

    frame.render_widget(chart, area);
}

fn render_summary(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Summary ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::MUTED));

    let total_h = app.report_total_secs as f64 / 3600.0;
    let billable_h = app.report_billable_secs as f64 / 3600.0;
    let non_billable = total_h - billable_h;
    let num_projects = app.report_project_breakdown.len();
    let total_entries: i64 = app.report_project_breakdown.iter().map(|b| b.entry_count).sum();

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Total Hours:     ", theme::header_style()),
            Span::styled(format!("{:.1}", total_h), Style::default().fg(theme::FG).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" Billable:        ", theme::header_style()),
            Span::styled(format!("{:.1}", billable_h), Style::default().fg(theme::SUCCESS)),
        ]),
        Line::from(vec![
            Span::styled(" Non-billable:    ", theme::header_style()),
            Span::raw(format!("{:.1}", non_billable)),
        ]),
        Line::from(vec![
            Span::styled(" Projects:        ", theme::header_style()),
            Span::raw(format!("{}", num_projects)),
        ]),
        Line::from(vec![
            Span::styled(" Entries:         ", theme::header_style()),
            Span::raw(format!("{}", total_entries)),
        ]),
    ];

    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, area);
}

fn render_project_table(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" By Project ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    if app.report_project_breakdown.is_empty() {
        let p = Paragraph::new(" No data").style(theme::muted_style()).block(block);
        frame.render_widget(p, area);
        return;
    }

    let rows: Vec<Row> = app.report_project_breakdown.iter().map(|b| {
        let hours = b.total_seconds as f64 / 3600.0;
        let billable_hours = b.billable_seconds as f64 / 3600.0;
        Row::new(vec![
            b.project_name.clone(),
            format!("{:.1}h", hours),
            format!("{:.1}h", billable_hours),
            format!("{}", b.entry_count),
        ])
    }).collect();

    let header = Row::new(vec!["Project", "Total", "Billable", "Entries"])
        .style(theme::header_style());

    let table = Table::new(
        rows,
        [
            Constraint::Min(15),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(8),
        ]
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}
