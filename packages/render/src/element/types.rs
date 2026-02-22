use core::schema::types::{CVEducation, CVExperience, CVLinks, CVPerson};

use oxidize_pdf::{Document, Page};

use crate::{element::utils::render_element, style::types::Style};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Element {
    Header(CVPerson, CVLinks),
    Title(String),
    Body(String),
    Tags(Vec<String>),
    List(Vec<String>),
    Table(Vec<(String, String)>),
    Experience(CVExperience),
    Education(CVEducation),
}

impl Element {
    pub fn requires_spacing(&self, previous_element: &Element) -> bool {
        return !matches!(previous_element, Element::Title(_));
    }

    pub fn render(
        &self,
        style: &Style,
        doc: &mut Document,
        page: &mut Page,
        previous_y: &mut f64,
        previous_element: &mut Option<Element>,
    ) -> Result<(), anyhow::Error> {
        render_element(self, style, doc, page, previous_y, previous_element)
    }
}
