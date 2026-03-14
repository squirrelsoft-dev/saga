use anyhow::Result;
use genpdf::{elements, style, Element, Document, SimplePageDecorator};
use crate::models::{ProjectBreakdown, DailySummary};
use crate::app::state::ReportPeriod;

pub fn export_report_pdf(
    breakdowns: &[ProjectBreakdown],
    daily_summaries: &[DailySummary],
    period: ReportPeriod,
) -> Result<String> {
    let filename = format!("saga-report-{}.pdf", chrono::Local::now().format("%Y%m%d-%H%M%S"));

    // Try to load font from bundled fonts, fall back to built-in
    let font_family = load_font_family()?;

    let mut doc = Document::new(font_family);
    doc.set_title("Saga Time Report");

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(20);
    doc.set_page_decorator(decorator);

    // Title
    doc.push(elements::Paragraph::new(format!("Time Report - {}", period.label()))
        .styled(style::Style::new().bold().with_font_size(18)));
    doc.push(elements::Paragraph::new(format!("Generated: {}", chrono::Local::now().format("%Y-%m-%d %H:%M")))
        .styled(style::Style::new().with_font_size(10)));
    doc.push(elements::Break::new(1));

    // Summary
    let total_secs: i64 = breakdowns.iter().map(|b| b.total_seconds).sum();
    let billable_secs: i64 = breakdowns.iter().map(|b| b.billable_seconds).sum();
    let total_hours = total_secs as f64 / 3600.0;
    let billable_hours = billable_secs as f64 / 3600.0;

    doc.push(elements::Paragraph::new("Summary")
        .styled(style::Style::new().bold().with_font_size(14)));
    doc.push(elements::Paragraph::new(format!("Total Hours: {:.1}", total_hours)));
    doc.push(elements::Paragraph::new(format!("Billable Hours: {:.1}", billable_hours)));
    doc.push(elements::Paragraph::new(format!("Projects: {}", breakdowns.len())));
    doc.push(elements::Break::new(1));

    // Project breakdown table
    if !breakdowns.is_empty() {
        doc.push(elements::Paragraph::new("By Project")
            .styled(style::Style::new().bold().with_font_size(14)));

        let mut table = elements::TableLayout::new(vec![1, 1, 1, 1]);
        table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));

        // Header
        let header_style = style::Style::new().bold();
        let mut header = table.row();
        header.push_element(elements::Paragraph::new("Project").styled(header_style));
        header.push_element(elements::Paragraph::new("Total").styled(header_style));
        header.push_element(elements::Paragraph::new("Billable").styled(header_style));
        header.push_element(elements::Paragraph::new("Entries").styled(header_style));
        header.push().expect("Failed to push header row");

        for b in breakdowns {
            let mut row = table.row();
            row.push_element(elements::Paragraph::new(&b.project_name));
            row.push_element(elements::Paragraph::new(format!("{:.1}h", b.total_seconds as f64 / 3600.0)));
            row.push_element(elements::Paragraph::new(format!("{:.1}h", b.billable_seconds as f64 / 3600.0)));
            row.push_element(elements::Paragraph::new(format!("{}", b.entry_count)));
            row.push().expect("Failed to push row");
        }

        doc.push(table);
        doc.push(elements::Break::new(1));
    }

    // Daily summaries
    if !daily_summaries.is_empty() {
        doc.push(elements::Paragraph::new("Daily Breakdown")
            .styled(style::Style::new().bold().with_font_size(14)));

        let mut table = elements::TableLayout::new(vec![1, 1, 1]);
        table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));

        let header_style = style::Style::new().bold();
        let mut header = table.row();
        header.push_element(elements::Paragraph::new("Date").styled(header_style));
        header.push_element(elements::Paragraph::new("Hours").styled(header_style));
        header.push_element(elements::Paragraph::new("Entries").styled(header_style));
        header.push().expect("Failed to push header row");

        for s in daily_summaries {
            let mut row = table.row();
            row.push_element(elements::Paragraph::new(&s.date));
            row.push_element(elements::Paragraph::new(format!("{:.1}h", s.total_seconds as f64 / 3600.0)));
            row.push_element(elements::Paragraph::new(format!("{}", s.entry_count)));
            row.push().expect("Failed to push row");
        }

        doc.push(table);
    }

    doc.render_to_file(&filename)?;
    Ok(filename)
}

fn load_font_family() -> Result<genpdf::fonts::FontFamily<genpdf::fonts::FontData>> {
    // Try bundled fonts first
    let font_dir = std::path::Path::new("fonts");
    if font_dir.exists() {
        if let Ok(family) = genpdf::fonts::from_files(font_dir, "LiberationSans", None) {
            return Ok(family);
        }
    }

    // Try system fonts
    for path in &[
        "/usr/share/fonts/truetype/liberation",
        "/usr/share/fonts/liberation-sans",
        "/usr/share/fonts/TTF",
    ] {
        let p = std::path::Path::new(path);
        if p.exists() {
            if let Ok(family) = genpdf::fonts::from_files(p, "LiberationSans", None) {
                return Ok(family);
            }
        }
    }

    Err(anyhow::anyhow!(
        "Could not find fonts. Place LiberationSans-Regular.ttf, LiberationSans-Bold.ttf, \
         and LiberationSans-Italic.ttf in the 'fonts/' directory."
    ))
}
