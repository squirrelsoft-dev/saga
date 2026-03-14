use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use crate::app::App;
use crate::ui::{layout::centered_rect, theme};

pub fn render(_app: &App, frame: &mut Frame, area: Rect) {
    let popup = centered_rect(50, 70, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    let shortcuts = vec![
        ("", "Global"),
        ("1-7", "Switch tabs"),
        ("Tab/Shift+Tab", "Next/prev tab"),
        ("s", "Start/stop timer"),
        ("Ctrl+p", "Project picker"),
        ("?", "Toggle help"),
        ("q", "Quit"),
        ("", ""),
        ("", "Entries"),
        ("j/k or arrows", "Navigate"),
        ("n/a", "New entry"),
        ("e/Enter", "Edit entry"),
        ("d/Delete", "Delete entry"),
        ("", ""),
        ("", "Projects"),
        ("j/k or arrows", "Navigate"),
        ("x", "Archive/activate"),
        ("h", "Toggle archived"),
        ("", ""),
        ("", "Reports"),
        ("h/l or arrows", "Change period"),
        ("c", "Export CSV"),
        ("p", "Export PDF"),
    ];

    let lines: Vec<Line> = shortcuts.iter().map(|(key, desc)| {
        if key.is_empty() {
            Line::from(Span::styled(
                format!("  {}", desc),
                Style::default().fg(theme::PRIMARY).add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(vec![
                Span::styled(format!("  {:20}", key), Style::default().fg(theme::ACCENT)),
                Span::raw(*desc),
            ])
        }
    }).collect();

    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, popup);
}
