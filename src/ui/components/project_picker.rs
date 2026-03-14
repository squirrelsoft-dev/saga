use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use crate::app::App;
use crate::ui::{layout::centered_rect, theme};

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let popup = centered_rect(40, 50, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Select Project ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
        ])
        .split(inner);

    // Search input
    let search_block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ACCENT));
    let search = Paragraph::new(format!(" {}", app.picker_filter.value))
        .block(search_block);
    frame.render_widget(search, chunks[0]);

    // Project list
    let items: Vec<ListItem> = app.picker_projects.iter().map(|p| {
        ListItem::new(Line::from(Span::raw(format!("  {}", p.name))))
    }).collect();

    let list = List::new(items)
        .highlight_style(theme::selected_style())
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.picker_selected));
    frame.render_stateful_widget(list, chunks[1], &mut state);
}
