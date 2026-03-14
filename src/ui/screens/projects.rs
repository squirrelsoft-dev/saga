use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState, Gauge},
};
use crate::app::App;
use crate::ui::theme;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

    render_project_list(app, frame, chunks[0]);
    render_project_detail(app, frame, chunks[1]);
}

fn render_project_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let title = if app.show_archived_projects {
        " Projects (all) "
    } else {
        " Projects (active) "
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    if app.projects.is_empty() {
        let p = Paragraph::new(" No projects. Use 'saga projects add <name>' to create one.")
            .style(theme::muted_style())
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let rows: Vec<Row> = app.projects.iter().map(|project| {
        let status = match project.status {
            crate::models::ProjectStatus::Active => "●",
            crate::models::ProjectStatus::Archived => "○",
        };
        Row::new(vec![
            status.to_string(),
            project.name.clone(),
            project.budget_hours.map(|b| format!("{:.0}h", b)).unwrap_or_default(),
        ])
    }).collect();

    let header = Row::new(vec!["", "Name", "Budget"])
        .style(theme::header_style())
        .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Min(15),
            Constraint::Length(8),
        ]
    )
    .header(header)
    .block(block)
    .row_highlight_style(theme::selected_style());

    let mut state = TableState::default();
    state.select(Some(app.projects_selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn render_project_detail(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::MUTED));

    if app.projects.is_empty() || app.projects_selected >= app.projects.len() {
        let p = Paragraph::new("").block(block);
        frame.render_widget(p, area);
        return;
    }

    let project = &app.projects[app.projects_selected];

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(block.inner(area));

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Name: ", theme::header_style()),
            Span::raw(&project.name),
        ]),
        Line::from(vec![
            Span::styled("Status: ", theme::header_style()),
            Span::raw(project.status.as_str()),
        ]),
        Line::from(vec![
            Span::styled("Color: ", theme::header_style()),
            Span::raw(&project.color),
        ]),
    ];

    if let Some(budget) = project.budget_hours {
        lines.push(Line::from(vec![
            Span::styled("Budget: ", theme::header_style()),
            Span::raw(format!("{:.1} hours", budget)),
        ]));
    }

    if let Some(ref notes) = project.notes {
        lines.push(Line::from(vec![
            Span::styled("Notes: ", theme::header_style()),
            Span::raw(notes),
        ]));
    }

    let detail = Paragraph::new(lines);
    frame.render_widget(block, area);
    frame.render_widget(detail, inner[0]);

    // Budget gauge if applicable
    if let Some(budget) = project.budget_hours {
        let ratio = 0.0_f64; // TODO: calculate from actual hours
        let gauge = Gauge::default()
            .block(Block::default().title(" Budget ").borders(Borders::ALL))
            .gauge_style(Style::default().fg(theme::PRIMARY))
            .ratio(ratio.min(1.0))
            .label(format!("{:.1}/{:.1}h", ratio * budget, budget));
        frame.render_widget(gauge, inner[1]);
    }
}
