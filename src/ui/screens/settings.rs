use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    style::Style,
};
use crate::app::App;
use crate::ui::theme;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    let config = &app.config;
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Default Currency:    ", theme::header_style()),
            Span::raw(&config.default_currency),
        ]),
        Line::from(vec![
            Span::styled("  Default Billable:    ", theme::header_style()),
            Span::raw(if config.default_billable { "Yes" } else { "No" }),
        ]),
        Line::from(vec![
            Span::styled("  Daily Goal:          ", theme::header_style()),
            Span::raw(config.daily_goal_hours.map(|h| format!("{:.1}h", h)).unwrap_or("Not set".to_string())),
        ]),
        Line::from(vec![
            Span::styled("  Weekly Goal:         ", theme::header_style()),
            Span::raw(config.weekly_goal_hours.map(|h| format!("{:.1}h", h)).unwrap_or("Not set".to_string())),
        ]),
        Line::from(vec![
            Span::styled("  Tick Rate:           ", theme::header_style()),
            Span::raw(format!("{}ms", config.tick_rate_ms)),
        ]),
        Line::from(vec![
            Span::styled("  Date Format:         ", theme::header_style()),
            Span::raw(&config.date_format),
        ]),
        Line::from(vec![
            Span::styled("  Time Format:         ", theme::header_style()),
            Span::raw(&config.time_format),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Use 'saga config set <key> <value>' to modify settings.", theme::muted_style())),
    ];

    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, area);
}
