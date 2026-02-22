use oxidize_pdf::{Document, Page, TextAlign};

use crate::template::types::{Element, ElementSet, Style};

pub fn render_element_set(
    element_set: ElementSet,
    doc: &mut Document,
    style: Option<Style>,
) -> Result<(), anyhow::Error> {
    let mut page = Page::a4();

    let style = &style.unwrap_or(Style::default());

    let mut previous_y = page.height() - style.margin * 4.0;

    for element in element_set.get_elements() {
        let mut flow = page.text_flow();
        flow.at(style.margin, previous_y);

        match element {
            Element::Header(person, links) => {
                flow.set_font(style.clone().font, style.display_size)
                    .set_alignment(TextAlign::Center)
                    .write_wrapped(&format!("{}\n", person.name))?;

                let links = vec![
                    person.location.clone(),
                    format!("Email: {}", person.email),
                    format!("Phone: {}", person.phone),
                    format!("GitHub: {}", links.github),
                    format!("LinkedIn: {}", links.linkedin),
                    format!("Portfolio: {}", links.portfolio),
                ]
                .join(" | ");

                flow.set_font(style.clone().font, style.body_size)
                    .set_alignment(TextAlign::Center)
                    .write_wrapped(&links)?;
            }
            Element::Body(text) => {
                flow.set_font(style.clone().font, style.body_size)
                    .set_alignment(TextAlign::Left)
                    .write_wrapped(&text)?;
            }
            Element::Title(text) => {
                flow.set_font(style.clone().font, style.title_size)
                    .set_alignment(TextAlign::Left)
                    .write_wrapped(&text)?;
            }
            Element::Tags(tags) => {
                flow.set_font(style.clone().font, style.body_size)
                    .set_alignment(TextAlign::Left)
                    .write_wrapped(&tags.join(" | "))?;
            }
            Element::List(items) => {
                flow.set_font(style.clone().font, style.body_size)
                    .set_alignment(TextAlign::Left)
                    .write_paragraph(&format!("- {}\n", items.join("\n- ")))?;
            }
            Element::Education(education) => {
                flow.set_font(style.clone().font, style.body_size)
                    .set_alignment(TextAlign::Left)
                    .write_wrapped(&format!(
                        "{} - {} in {} | {} | {} to {}\n",
                        education.institution,
                        education.degree,
                        education.field,
                        education.location,
                        education.start_date,
                        education.end_date
                    ))?;
            }
            Element::Experience(experience) => {
                flow.set_font(style.clone().font, style.body_size)
                    .set_alignment(TextAlign::Left)
                    .write_wrapped(&format!(
                        "{} - {} at {}\n{}",
                        experience.start_date,
                        experience.end_date,
                        experience.role,
                        experience.summary
                    ))?;
            }
            Element::Table(rows) => {
                for (key, value) in rows {
                    flow.set_font(style.clone().font, style.body_size)
                        .set_alignment(TextAlign::Left)
                        .write_wrapped(&format!("{}: {}\n", key, value))?;
                }
            }
        }

        flow.write_wrapped("\n")?;

        if (flow.cursor_position().1 + style.element_spacing) < style.margin * 4.0 {
            page.add_text_flow(&flow);
            doc.add_page(page.clone());
            page = Page::a4();
            flow = page.text_flow();
            flow.at(style.margin, page.height() - style.margin * 4.0);
        }

        previous_y = flow.cursor_position().1;

        page.add_text_flow(&flow);
    }

    doc.add_page(page);

    Ok(())
}
