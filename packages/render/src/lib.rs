mod element;
mod element_set;
mod style;

use core::schema::types::CVSchema;
use oxidize_pdf::Document;

pub fn render(schema: &CVSchema) -> Result<Document, anyhow::Error> {
    let mut doc = Document::new();
    doc.set_title(&format!("{}'s CV", &schema.person.name));

    let element_set = element_set::utils::build_element_set(schema);

    element_set.render(&mut doc, None)?;

    Ok(doc)
}
