use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use crate::app::App;
use crate::ui::{layout::centered_rect, theme};

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 60, area);
    frame.render_widget(Clear, popup);

    let title = if app.entry_form.editing_id.is_some() {
        " Edit Entry "
    } else {
        " New Entry "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(inner);

    let form = &app.entry_form;
    render_field("Project", &form.project_name.value, form.focused_field == 0, frame, fields[0]);
    render_field("Description", &form.description.value, form.focused_field == 1, frame, fields[1]);
    render_field("Date", &form.date.value, form.focused_field == 2, frame, fields[2]);
    render_field("Start (HH:MM)", &form.start_time.value, form.focused_field == 3, frame, fields[3]);
    render_field("End (HH:MM)", &form.end_time.value, form.focused_field == 4, frame, fields[4]);

    // Billable toggle
    let billable_style = if form.focused_field == 5 {
        Style::default().fg(theme::PRIMARY).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::FG_DIM)
    };
    let billable_text = if form.billable { "[x] Billable" } else { "[ ] Billable" };
    let billable = Paragraph::new(Line::from(Span::styled(format!(" {}", billable_text), billable_style)))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(theme::MUTED)));
    frame.render_widget(billable, fields[5]);

    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled(" Tab", Style::default().fg(theme::ACCENT)),
        Span::raw(":next  "),
        Span::styled("Enter", Style::default().fg(theme::ACCENT)),
        Span::raw(":save  "),
        Span::styled("Esc", Style::default().fg(theme::ACCENT)),
        Span::raw(":cancel"),
    ]));
    frame.render_widget(help, fields[6]);
}

fn render_field(label: &str, value: &str, focused: bool, frame: &mut Frame, area: Rect) {
    let style = if focused {
        Style::default().fg(theme::PRIMARY)
    } else {
        Style::default().fg(theme::FG_DIM)
    };

    let block = Block::default()
        .title(format!(" {} ", label))
        .borders(Borders::ALL)
        .border_style(if focused {
            Style::default().fg(theme::PRIMARY)
        } else {
            Style::default().fg(theme::MUTED)
        });

    let display = if value.is_empty() && !focused {
        format!(" {}", label)
    } else {
        format!(" {}", value)
    };

    let p = Paragraph::new(Line::from(Span::styled(display, style))).block(block);
    frame.render_widget(p, area);
}
