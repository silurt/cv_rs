mod build_element_set;
mod template;

use core::schema::types::CVSchema;
use oxidize_pdf::Document;

use crate::{build_element_set::build_element_set, template::render::render_element_set};

pub fn render(schema: &CVSchema) -> Result<Document, anyhow::Error> {
    let mut doc = Document::new();
    doc.set_title(&format!("{}'s CV", &schema.person.name));

    render_element_set(build_element_set(schema), &mut doc)?;

    Ok(doc)
}
