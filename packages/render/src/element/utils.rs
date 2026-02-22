use oxidize_pdf::{Document, Page, TextAlign};

use crate::{
    element::types::Element,
    style::types::{Style, StyleFont},
};

pub fn render_element(
    element: &Element,
    style: &Style,
    doc: &mut Document,
    page: &mut Page,
    previous_y: &mut f64,
    previous_element: &mut Option<Element>,
) -> Result<(), anyhow::Error> {
    let mut flow = page.text_flow();
    flow.at(style.margin, *previous_y);

    if let Some(previous_element) = previous_element {
        if element.requires_spacing(previous_element) {
            style.write_to_flow(&mut flow, StyleFont::Body, TextAlign::Left, "\n")?;
        }
    }
    *previous_element = Some(element.clone());

    match element {
        Element::Header(person, links) => {
            style.write_to_flow(
                &mut flow,
                StyleFont::Display,
                TextAlign::Center,
                &person.name.to_uppercase(),
            )?;

            let links = vec![
                person.location.clone(),
                format!("Email: {}", person.email),
                format!("Phone: {}", person.phone),
                format!("GitHub: {}", links.github),
                format!("LinkedIn: {}", links.linkedin),
                format!("Portfolio: {}", links.portfolio),
            ]
            .join(" | ");

            style.write_to_flow(&mut flow, StyleFont::Body, TextAlign::Center, &links)?;

            style.style_flow(StyleFont::Body, &mut flow, TextAlign::Center);

            // Add a horizontal line after the header
            let current_y = flow.cursor_position().1;
            let _ = flow.write_paragraph("\n\n");
            let _ = flow.write_paragraph("\n\n");
            let new_y = flow.cursor_position().1;

            let page_width = page.width();

            let line_y = (current_y + new_y) / 2.0;

            // Draw a line from (margin, current_y - 4) to (page.width() - margin, current_y - 4)
            page.graphics()
                .set_line_width(1.0)
                .move_to(style.margin, line_y)
                .line_to(page_width - style.margin, line_y)
                .stroke();
        }
        Element::Body(text) => {
            style.write_to_flow(&mut flow, StyleFont::Body, TextAlign::Left, text)?;
        }
        Element::Title(text) => {
            style.write_to_flow(&mut flow, StyleFont::Title, TextAlign::Left, text)?;
        }
        Element::Tags(tags) => {
            style.write_to_flow(
                &mut flow,
                StyleFont::Body,
                TextAlign::Left,
                &tags.join(" | "),
            )?;
        }
        Element::List(items) => {
            style.write_to_flow(
                &mut flow,
                StyleFont::Body,
                TextAlign::Left,
                &format!("- {}\n", items.join("\n- ")),
            )?;
        }
        Element::Education(education) => {
            style.write_to_flow(
                &mut flow,
                StyleFont::Body,
                TextAlign::Left,
                &format!(
                    "{} - {} in {} | {} | {} to {}\n",
                    education.institution,
                    education.degree,
                    education.field,
                    education.location,
                    education.start_date,
                    education.end_date
                ),
            )?;
        }
        Element::Experience(experience) => {
            style.write_to_flow(
                &mut flow,
                StyleFont::Body,
                TextAlign::Left,
                &format!(
                    "{} - {} at {}\n{}",
                    experience.start_date, experience.end_date, experience.role, experience.summary
                ),
            )?;
        }
        Element::Table(rows) => {
            for (key, value) in rows {
                style.write_to_flow(
                    &mut flow,
                    StyleFont::Body,
                    TextAlign::Left,
                    &format!("{}: {}\n", key, value),
                )?;
            }
        }
    }

    if (flow.cursor_position().1 + style.element_spacing) < style.margin {
        page.add_text_flow(&flow);
        doc.add_page(page.clone());
        *page = style.setup_page();

        flow = page.text_flow();

        let (x, y) = style.get_initial_cursor_position(page);
        flow.at(x, y);
    }

    *previous_y = flow.cursor_position().1;

    page.add_text_flow(&flow);

    Ok(())
}
