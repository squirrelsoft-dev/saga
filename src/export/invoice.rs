use anyhow::Result;
use genpdf::{elements, style, Element, Document, SimplePageDecorator};
use crate::models::{Client, TimeEntry, Project};

pub struct InvoiceData {
    pub invoice_number: String,
    pub client: Client,
    pub entries: Vec<(TimeEntry, Project)>,
    pub hourly_rate: f64,
    pub currency: String,
    pub period_start: String,
    pub period_end: String,
    pub notes: Option<String>,
}

pub fn generate_invoice(data: &InvoiceData) -> Result<String> {
    let filename = format!("invoice-{}.pdf", data.invoice_number);

    let font_family = load_font_family()?;

    let mut doc = Document::new(font_family);
    doc.set_title(format!("Invoice {}", data.invoice_number));

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(20);
    doc.set_page_decorator(decorator);

    // Header
    doc.push(elements::Paragraph::new("INVOICE")
        .styled(style::Style::new().bold().with_font_size(24)));
    doc.push(elements::Break::new(1));

    // Invoice details
    doc.push(elements::Paragraph::new(format!("Invoice #: {}", data.invoice_number))
        .styled(style::Style::new().bold()));
    doc.push(elements::Paragraph::new(format!("Date: {}", chrono::Local::now().format("%Y-%m-%d"))));
    doc.push(elements::Paragraph::new(format!("Period: {} to {}", data.period_start, data.period_end)));
    doc.push(elements::Break::new(1));

    // Client info
    doc.push(elements::Paragraph::new("Bill To:")
        .styled(style::Style::new().bold().with_font_size(12)));
    doc.push(elements::Paragraph::new(&data.client.name));
    if let Some(ref email) = data.client.email {
        doc.push(elements::Paragraph::new(email));
    }
    if let Some(ref contact) = data.client.contact {
        doc.push(elements::Paragraph::new(contact));
    }
    doc.push(elements::Break::new(1));

    // Line items table
    let mut table = elements::TableLayout::new(vec![2, 3, 1, 1, 1]);
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));

    let header_style = style::Style::new().bold();
    let mut header = table.row();
    header.push_element(elements::Paragraph::new("Date").styled(header_style));
    header.push_element(elements::Paragraph::new("Description").styled(header_style));
    header.push_element(elements::Paragraph::new("Hours").styled(header_style));
    header.push_element(elements::Paragraph::new("Rate").styled(header_style));
    header.push_element(elements::Paragraph::new("Amount").styled(header_style));
    header.push().expect("Failed to push header");

    let mut total_hours = 0.0_f64;
    let mut total_amount = 0.0_f64;

    for (entry, project) in &data.entries {
        if !entry.billable {
            continue;
        }
        let hours = entry.duration_secs.unwrap_or(0) as f64 / 3600.0;
        let amount = hours * data.hourly_rate;
        total_hours += hours;
        total_amount += amount;

        let desc = if entry.description.is_empty() {
            project.name.clone()
        } else {
            format!("{}: {}", project.name, entry.description)
        };

        let mut row = table.row();
        row.push_element(elements::Paragraph::new(entry.start_time.format("%Y-%m-%d").to_string()));
        row.push_element(elements::Paragraph::new(desc));
        row.push_element(elements::Paragraph::new(format!("{:.2}", hours)));
        row.push_element(elements::Paragraph::new(format!("{:.2}", data.hourly_rate)));
        row.push_element(elements::Paragraph::new(format!("{:.2}", amount)));
        row.push().expect("Failed to push row");
    }

    doc.push(table);
    doc.push(elements::Break::new(1));

    // Totals
    doc.push(elements::Paragraph::new(format!("Total Hours: {:.2}", total_hours))
        .styled(style::Style::new().bold()));
    doc.push(elements::Paragraph::new(
        format!("Total Amount: {} {:.2}", data.currency, total_amount))
        .styled(style::Style::new().bold().with_font_size(14)));

    // Notes
    if let Some(ref notes) = data.notes {
        doc.push(elements::Break::new(1));
        doc.push(elements::Paragraph::new("Notes:")
            .styled(style::Style::new().bold()));
        doc.push(elements::Paragraph::new(notes));
    }

    doc.render_to_file(&filename)?;
    Ok(filename)
}

fn load_font_family() -> Result<genpdf::fonts::FontFamily<genpdf::fonts::FontData>> {
    let font_dir = std::path::Path::new("fonts");
    if font_dir.exists() {
        if let Ok(family) = genpdf::fonts::from_files(font_dir, "LiberationSans", None) {
            return Ok(family);
        }
    }

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
