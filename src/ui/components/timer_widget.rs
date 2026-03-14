use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::ui::theme;

pub fn render(seconds: u64, running: bool, frame: &mut Frame, area: Rect) {
    let hours = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let time_str = format!("{:02}:{:02}:{:02}", hours, mins, secs);
    let color = if running { theme::SUCCESS } else { theme::FG_DIM };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if running { theme::SUCCESS } else { theme::MUTED }));

    let p = Paragraph::new(Line::from(Span::styled(
        time_str,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(block);

    frame.render_widget(p, area);
}
