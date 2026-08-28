mod block;
mod layout;
mod style;

use core::schema::types::CVSchema;
use oxidize_pdf::Document;

use crate::block::utils::{render_header, render_section};
use crate::layout::types::Renderer;
use core::document::utils::build_sections;

/// How to render the document.
#[derive(Clone, Copy, Debug)]
pub struct RenderOptions {
    /// Emit a Tagged PDF: a structure tree marking headings, paragraphs and
    /// lists, so a parser reads a logical document rather than positioned
    /// glyphs. Costs roughly twice the file size; see the README.
    pub tagged: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self { tagged: true }
    }
}

pub fn render(schema: &CVSchema) -> Result<Document, anyhow::Error> {
    render_with(schema, RenderOptions::default())
}

pub fn render_with(schema: &CVSchema, options: RenderOptions) -> Result<Document, anyhow::Error> {
    let mut doc = Document::new();
    doc.set_title(format!("{}'s CV", schema.person.name));

    {
        let mut renderer = Renderer::new(&mut doc);
        render_header(&mut renderer, &schema.person, &schema.links);

        for section in build_sections(schema) {
            render_section(&mut renderer, &section);
        }

        renderer.finish(options.tagged);
    }

    Ok(doc)
}
