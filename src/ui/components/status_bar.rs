use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::App;
use crate::ui::theme;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(30),
            Constraint::Length(30),
        ])
        .split(area);

    // Left: status message or shortcuts
    let left_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::MUTED));

    let left_text = if let Some((ref msg, _)) = app.status_message {
        Line::from(Span::styled(format!(" {}", msg), Style::default().fg(theme::ACCENT)))
    } else {
        Line::from(vec![
            Span::styled(" s", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(":timer "),
            Span::styled("?", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(":help "),
            Span::styled("q", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(":quit "),
            Span::styled("1-7", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(":tabs"),
        ])
    };

    let left = Paragraph::new(left_text).block(left_block);
    frame.render_widget(left, chunks[0]);

    // Right: active timer indicator
    let right_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::MUTED));

    let right_text = if let Some(ref _entry) = app.active_entry {
        let hours = app.timer_seconds / 3600;
        let mins = (app.timer_seconds % 3600) / 60;
        let secs = app.timer_seconds % 60;
        let proj = app.active_project_name.as_deref().unwrap_or("...");
        Line::from(vec![
            Span::styled("● ", Style::default().fg(theme::SUCCESS)),
            Span::raw(format!("{} {:02}:{:02}:{:02}", proj, hours, mins, secs)),
        ])
    } else {
        Line::from(Span::styled(" No timer", theme::muted_style()))
    };

    let right = Paragraph::new(right_text).block(right_block);
    frame.render_widget(right, chunks[1]);
}
