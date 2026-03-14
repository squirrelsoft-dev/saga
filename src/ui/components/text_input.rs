use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::ui::theme;

pub fn render(label: &str, value: &str, focused: bool, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(format!(" {} ", label))
        .borders(Borders::ALL)
        .border_style(if focused {
            Style::default().fg(theme::PRIMARY)
        } else {
            Style::default().fg(theme::MUTED)
        });

    let text = if value.is_empty() {
        Line::from(Span::styled(format!(" {}", label), theme::muted_style()))
    } else {
        Line::from(Span::raw(format!(" {}", value)))
    };

    let p = Paragraph::new(text).block(block);
    frame.render_widget(p, area);
}
