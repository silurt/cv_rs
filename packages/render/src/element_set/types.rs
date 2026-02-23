use oxidize_pdf::Document;

use crate::{element::types::Element, style::types::Style};

pub struct ElementSet {
    elements: Vec<Element>,
}

impl ElementSet {
    pub fn new() -> Self {
        Self { elements: vec![] }
    }

    pub fn push(&mut self, element: Element) {
        self.elements.push(element);
    }

    pub fn get_elements(&self) -> &Vec<Element> {
        &self.elements
    }

    pub fn render(&self, doc: &mut Document, style: Option<Style>) -> Result<(), anyhow::Error> {
        let style = &style.unwrap_or(Style::default());

        let mut page = style.setup_page();

        let (_, y) = style.get_initial_cursor_position(&page);
        let mut previous_y = y - style.display.size - style.spacings.element;
        let mut previous_element: Option<Element> = None;

        for element in self.get_elements() {
            element.render(
                style,
                doc,
                &mut page,
                &mut previous_y,
                &mut previous_element,
            )?;
        }

        doc.add_page(page);

        Ok(())
    }
}
