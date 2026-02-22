use core::schema::types::{CVEducation, CVExperience, CVLinks, CVPerson};

use oxidize_pdf::Font;

#[derive(Clone, Debug)]
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
}

#[derive(Clone)]
pub struct Style {
    pub font: Font,
    pub display_size: f64,
    pub title_size: f64,
    pub body_size: f64,
    pub margin: f64,
    pub element_spacing: f64,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            font: Font::Helvetica,
            display_size: 30.0,
            title_size: 14.0,
            body_size: 10.0,
            margin: 16.0,
            element_spacing: 4.0,
        }
    }
}
