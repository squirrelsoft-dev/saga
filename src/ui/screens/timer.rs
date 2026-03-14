use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::App;
use crate::ui::theme;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(5),
        ])
        .split(area);

    // Project name
    let project_name = app.active_project_name.as_deref().unwrap_or("No project selected");
    let project_block = Block::default()
        .title(" Project ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));
    let project_text = Paragraph::new(format!(" {}", project_name))
        .style(if app.active_entry.is_some() { theme::active_timer_style() } else { theme::muted_style() })
        .block(project_block);
    frame.render_widget(project_text, chunks[0]);

    // Big clock
    let hours = app.timer_seconds / 3600;
    let mins = (app.timer_seconds % 3600) / 60;
    let secs = app.timer_seconds % 60;
    let time_str = format!("{:02}:{:02}:{:02}", hours, mins, secs);

    let clock_block = Block::default()
        .title(" Timer ")
        .borders(Borders::ALL)
        .border_style(if app.active_entry.is_some() {
            Style::default().fg(theme::SUCCESS)
        } else {
            Style::default().fg(theme::MUTED)
        });

    let clock = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            time_str,
            Style::default()
                .fg(if app.active_entry.is_some() { theme::SUCCESS } else { theme::FG })
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .alignment(Alignment::Center)
    .block(clock_block);
    frame.render_widget(clock, chunks[1]);

    // Controls
    let controls_block = Block::default()
        .title(" Controls ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::MUTED));

    let controls = if app.active_entry.is_some() {
        vec![
            Line::from(vec![
                Span::styled(" Enter/s ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
                Span::raw("Stop  "),
                Span::styled(" c ", Style::default().fg(theme::DANGER).add_modifier(Modifier::BOLD)),
                Span::raw("Cancel"),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled(" Enter/s ", Style::default().fg(theme::SUCCESS).add_modifier(Modifier::BOLD)),
                Span::raw("Start  "),
                Span::styled(" p ", Style::default().fg(theme::PRIMARY).add_modifier(Modifier::BOLD)),
                Span::raw("Pick project"),
            ]),
        ]
    };

    let controls_widget = Paragraph::new(controls).block(controls_block);
    frame.render_widget(controls_widget, chunks[2]);
}
