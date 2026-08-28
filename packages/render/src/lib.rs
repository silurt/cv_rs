mod block;
mod layout;
mod sections;
mod style;

use core::schema::types::CVSchema;
use oxidize_pdf::Document;

use crate::block::utils::{render_header, render_section};
use crate::layout::types::Renderer;
use crate::sections::utils::build_sections;

pub fn render(schema: &CVSchema) -> Result<Document, anyhow::Error> {
    let mut doc = Document::new();
    doc.set_title(format!("{}'s CV", schema.person.name));

    {
        let mut renderer = Renderer::new(&mut doc);
        render_header(&mut renderer, &schema.person, &schema.links);

        for section in build_sections(schema) {
            render_section(&mut renderer, &section);
        }

        renderer.finish();
    }

    Ok(doc)
}
