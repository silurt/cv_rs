use core::schema::types::{CVEducation, CVExperience, CVLinks, CVPerson};

use oxidize_pdf::Font;

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
pub struct FontConfig {
    pub font: Font,
    pub size: f64,
}

#[derive(Clone)]
pub struct Style {
    pub display: FontConfig,
    pub title: FontConfig,
    pub body: FontConfig,
    pub margin: f64,
    pub element_spacing: f64,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            display: FontConfig {
                font: Font::HelveticaBold,
                size: 30.0,
            },
            title: FontConfig {
                font: Font::HelveticaBold,
                size: 14.0,
            },
            body: FontConfig {
                font: Font::Helvetica,
                size: 10.0,
            },
            margin: 32.0,
            element_spacing: 4.0,
        }
    }
}
