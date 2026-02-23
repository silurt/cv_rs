use core::schema::types::{CVEducation, CVLinks, CVPerson};

use oxidize_pdf::{Document, Page};

use crate::{
    element::utils::render_element,
    style::types::{Spacing, Style},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub title: String,
    // Renders it inset with a line on the left
    pub is_quote: bool,
    pub elements: Vec<Element>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Element {
    Header(CVPerson, CVLinks),
    Title(String),
    Subtitle(String),
    Body(String),
    Tags(Vec<String>),
    List(Vec<String>),
    Table(Vec<(String, String)>),
    Block(Block),
    // Experience(CVExperience),
    Education(CVEducation),
}

impl Element {
    pub fn get_spacing(&self, previous_element: &Option<Element>, style: &Style) -> Option<f64> {
        match previous_element {
            Some(Element::Title(_)) => None,
            Some(Element::Block(_)) => {
                if matches!(self, Element::Block(_)) {
                    Some(Spacing::Element.get_spacing(style))
                } else {
                    Some(Spacing::Block.get_spacing(style))
                }
            }
            _ => Some(Spacing::Element.get_spacing(style)),
        }
    }

    pub fn get_indentation(&self) -> Option<f64> {
        match self {
            Element::Block(block) => {
                if block.is_quote {
                    Some(1.0)
                } else {
                    None
                }
            }
            _ => None,
        }
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
