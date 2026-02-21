pub enum Element {
    Body(String),
    Newline,
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
