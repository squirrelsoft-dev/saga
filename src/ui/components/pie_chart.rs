use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, canvas::{Canvas, Circle}},
};
use crate::ui::theme;

pub struct PieSlice {
    pub label: String,
    pub value: f64,
    pub color: ratatui::style::Color,
}

pub fn render(slices: &[PieSlice], title: &str, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::PRIMARY));

    // Simple representation using a Canvas with colored circles
    // For a real pie chart, we'd use more complex canvas drawing
    let total: f64 = slices.iter().map(|s| s.value).sum();
    if total == 0.0 {
        let p = ratatui::widgets::Paragraph::new(" No data").block(block);
        frame.render_widget(p, area);
        return;
    }

    let canvas = Canvas::default()
        .block(block)
        .x_bounds([-1.0, 1.0])
        .y_bounds([-1.0, 1.0])
        .paint(|ctx| {
            // Draw concentric circles as a simple representation
            for (i, slice) in slices.iter().enumerate() {
                let radius = 0.8 - (i as f64 * 0.15);
                if radius > 0.0 {
                    ctx.draw(&Circle {
                        x: 0.0,
                        y: 0.0,
                        radius,
                        color: slice.color,
                    });
                }
            }
        });

    frame.render_widget(canvas, area);
}
