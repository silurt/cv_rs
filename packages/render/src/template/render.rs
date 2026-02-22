use oxidize_pdf::{Document, Page, TextAlign, TextFlowContext};

use crate::template::types::{Element, ElementSet, FontConfig, Style};

fn get_initial_cursor_position(page: &Page, style: &Style) -> (f64, f64) {
    (0.0, page.height() - style.margin)
}

fn setup_page(style: &Style) -> Page {
    let mut page = Page::a4();
    page.set_margins(style.margin, style.margin, style.margin, style.margin);
    page
}

fn style_flow(flow: &mut TextFlowContext, font_config: &FontConfig, alignment: TextAlign) {
    flow.set_font(font_config.font.clone(), font_config.size)
        .set_alignment(alignment);
}

fn write_to_flow(
    flow: &mut TextFlowContext,
    font_config: &FontConfig,
    alignment: TextAlign,
    text: &str,
) -> Result<(), anyhow::Error> {
    style_flow(flow, font_config, alignment);
    flow.write_wrapped(text)?;
    Ok(())
}

fn render_element(
    element: &Element,
    style: &Style,
    doc: &mut Document,
    page: &mut Page,
    previous_y: &mut f64,
) -> Result<(), anyhow::Error> {
    let mut flow = page.text_flow();
    flow.at(style.margin, *previous_y);

    match element {
        Element::Header(person, links) => {
            write_to_flow(
                &mut flow,
                &style.display,
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

            write_to_flow(&mut flow, &style.body, TextAlign::Center, &links)?;

            style_flow(&mut flow, &style.body, TextAlign::Center);

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
            write_to_flow(&mut flow, &style.body, TextAlign::Left, text)?;
        }
        Element::Title(text) => {
            write_to_flow(&mut flow, &style.title, TextAlign::Left, text)?;
        }
        Element::Tags(tags) => {
            write_to_flow(&mut flow, &style.body, TextAlign::Left, &tags.join(" | "))?;
        }
        Element::List(items) => {
            write_to_flow(
                &mut flow,
                &style.body,
                TextAlign::Left,
                &format!("- {}\n", items.join("\n- ")),
            )?;
        }
        Element::Education(education) => {
            write_to_flow(
                &mut flow,
                &style.body,
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
            write_to_flow(
                &mut flow,
                &style.body,
                TextAlign::Left,
                &format!(
                    "{} - {} at {}\n{}",
                    experience.start_date, experience.end_date, experience.role, experience.summary
                ),
            )?;
        }
        Element::Table(rows) => {
            for (key, value) in rows {
                write_to_flow(
                    &mut flow,
                    &style.body,
                    TextAlign::Left,
                    &format!("{}: {}\n", key, value),
                )?;
            }
        }
    }


    write_to_flow(&mut flow, &style.body, TextAlign::Left, "\n")?;

    if (flow.cursor_position().1 + style.element_spacing) < style.margin {
        page.add_text_flow(&flow);
        doc.add_page(page.clone());
        *page = setup_page(style);

        flow = page.text_flow();

        let (x, y) = get_initial_cursor_position(page, style);
        flow.at(x, y);
    }

    *previous_y = flow.cursor_position().1;

    page.add_text_flow(&flow);

    Ok(())
}

pub fn render_element_set(
    element_set: ElementSet,
    doc: &mut Document,
    style: Option<Style>,
) -> Result<(), anyhow::Error> {
    let style = &style.unwrap_or(Style::default());

    let mut page = setup_page(style);

    let (_, y) = get_initial_cursor_position(&page, style);
    let mut previous_y = y - style.display.size - style.element_spacing;

    for element in element_set.get_elements() {
        render_element(element, style, doc, &mut page, &mut previous_y)?;
    }

    doc.add_page(page);

    Ok(())
}
