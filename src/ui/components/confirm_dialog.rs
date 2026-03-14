use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use crate::app::App;
use crate::ui::{layout::centered_rect_fixed, theme};

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let popup = centered_rect_fixed(40, 7, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::WARNING));

    let text = vec![
        Line::from(""),
        Line::from(Span::raw(format!("  {}", app.confirm_message))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [y]es  ", Style::default().fg(theme::SUCCESS)),
            Span::styled("[n]o", Style::default().fg(theme::DANGER)),
        ]),
    ];

    let p = Paragraph::new(text).block(block);
    frame.render_widget(p, popup);
}
