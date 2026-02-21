use oxidize_pdf::{Document, Font, Page, TextAlign};

use crate::template::types::{Element, ElementSet};

pub fn render_element_set(
    element_set: ElementSet,
    doc: &mut Document,
) -> Result<(), anyhow::Error> {
    let mut page = Page::a4();
    let mut flow = page.text_flow();

    flow.at(50.0, 750.0)
        .set_font(Font::Helvetica, 10.0)
        .set_alignment(TextAlign::Left);

    for element in element_set.get_elements() {
        match element {
            Element::Body(text) => {
                flow.write_wrapped(&text)?;
            }
            Element::Newline => {
                flow.write_wrapped("\n")?;
            }
        }

        // Check if the current flow position is near the bottom of the page, and if so, add a new page
        if flow.cursor_position().1 < 50.0 {
            page.add_text_flow(&flow);
            doc.add_page(page);

            page = Page::a4();
            flow = page.text_flow();

            flow.at(50.0, 750.0)
                .set_font(Font::Helvetica, 10.0)
                .set_alignment(TextAlign::Left);
        }
    }

    page.add_text_flow(&flow);
    doc.add_page(page);

    Ok(())
}
