use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
};
use crate::app::App;
use crate::app::state::Screen;
use crate::ui::theme;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let titles: Vec<Line> = [
        Screen::Dashboard,
        Screen::Timer,
        Screen::Entries,
        Screen::Projects,
        Screen::Clients,
        Screen::Reports,
        Screen::Settings,
    ]
    .iter()
    .enumerate()
    .map(|(i, screen)| {
        Line::from(Span::raw(format!(" {} {} ", i + 1, screen.label())))
    })
    .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(" Saga - Time Tracker ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::PRIMARY)),
        )
        .select(app.screen.index())
        .style(Style::default().fg(theme::FG_DIM))
        .highlight_style(
            Style::default()
                .fg(theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}
