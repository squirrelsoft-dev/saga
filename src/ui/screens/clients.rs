use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
};
use crate::app::App;
use crate::ui::theme;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(format!(" Clients ({}) ", app.clients.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    if app.clients.is_empty() {
        let p = Paragraph::new(" No clients. Use 'saga clients add <name>' to create one.")
            .style(theme::muted_style())
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let rows: Vec<Row> = app.clients.iter().map(|client| {
        Row::new(vec![
            client.name.clone(),
            client.email.clone().unwrap_or_default(),
            client.contact.clone().unwrap_or_default(),
        ])
    }).collect();

    let header = Row::new(vec!["Name", "Email", "Contact"])
        .style(theme::header_style())
        .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(35),
            Constraint::Percentage(35),
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(theme::selected_style());

    let mut state = TableState::default();
    state.select(Some(app.clients_selected));
    frame.render_stateful_widget(table, area, &mut state);
}
