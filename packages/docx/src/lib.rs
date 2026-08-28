//! A `.docx` backend for the same document model the PDF renderer uses.
//!
//! Why this exists: several widely deployed applicant tracking systems were
//! built around Word documents and parse `.docx` more reliably than PDF. The
//! two backends share `core::document`, so the section list, the ordering and
//! the text of every line are identical — only the container differs.
//!
//! The output is deliberately plain. Headings carry the built-in `Heading1`…
//! `Heading3` style ids rather than being ad-hoc bold text, because that is the
//! signal a parser reads to find section boundaries. There are no tables, text
//! boxes, headers or footers: everything lives in one linear body, in reading
//! order.

mod xml;

use std::io::{Cursor, Write};

use core::document::types::{Block, EntryVariant, Section};
use core::document::utils::{build_sections, contact_items};
use core::schema::types::CVSchema;
use zip::write::SimpleFileOptions;

use crate::xml::escape;

/// Render `schema` as the bytes of a `.docx` file.
pub fn render(schema: &CVSchema) -> Result<Vec<u8>, anyhow::Error> {
    let body = build_body(schema);

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            // A fixed timestamp keeps the output byte-reproducible.
            .last_modified_time(zip::DateTime::default());

        for (name, contents) in [
            ("[Content_Types].xml", xml::CONTENT_TYPES.to_string()),
            ("_rels/.rels", xml::ROOT_RELS.to_string()),
            (
                "word/_rels/document.xml.rels",
                xml::DOCUMENT_RELS.to_string(),
            ),
            ("word/styles.xml", xml::styles()),
            ("word/document.xml", body),
        ] {
            zip.start_file(name, options)?;
            zip.write_all(contents.as_bytes())?;
        }

        zip.finish()?;
    }

    Ok(buffer.into_inner())
}

/// A paragraph carrying a named style.
fn paragraph(style: &str, text: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    format!(
        "<w:p><w:pPr><w:pStyle w:val=\"{style}\"/></w:pPr><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
        escape(text)
    )
}

fn build_body(schema: &CVSchema) -> String {
    let mut out = String::from(xml::DOCUMENT_OPEN);

    // Header: the name is the document title, contacts are one plain line.
    out.push_str(&paragraph("Title", &schema.person.name));
    let contacts = contact_items(&schema.person, &schema.links);
    if !contacts.is_empty() {
        out.push_str(&paragraph("Contact", &contacts.join(" | ")));
    }

    for section in build_sections(schema) {
        out.push_str(&render_section(&section));
    }

    out.push_str(xml::DOCUMENT_CLOSE);
    out
}

fn render_section(section: &Section) -> String {
    let mut out = paragraph("Heading1", &section.title.to_uppercase());

    match &section.block {
        Block::Prose { paragraphs, .. } => {
            for text in paragraphs {
                out.push_str(&paragraph("Body", text));
            }
        }

        Block::InlineList {
            items, separator, ..
        } => {
            // One line, not one per item: an ATS reads this as a comma-style
            // list of skills rather than a column of fragments.
            out.push_str(&paragraph("Body", &items.join(separator)));
        }

        Block::BulletList { items, .. } => {
            for item in items {
                out.push_str(&paragraph("Bullet", &format!("\u{2022} {item}")));
            }
        }

        Block::EntryList {
            entries, variant, ..
        } => {
            let heading = if *variant == EntryVariant::Ruled {
                "Heading2"
            } else {
                "Heading3"
            };
            for entry in entries {
                out.push_str(&paragraph(heading, &entry.title));
                for line in &entry.meta {
                    out.push_str(&paragraph("Meta", line));
                }
                if let Some(summary) = &entry.summary {
                    out.push_str(&paragraph("Body", summary));
                }
                for bullet in &entry.bullets {
                    out.push_str(&paragraph("Bullet", &format!("\u{2022} {bullet}")));
                }
            }
        }

        Block::LabelValue { rows } => {
            // Rendered as "Label: value" lines rather than a table. Table cells
            // are a common source of parser confusion, and the label/value
            // relationship survives fine as running text.
            for row in rows {
                out.push_str(&paragraph("Body", &format!("{}: {}", row.label, row.value)));
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> CVSchema {
        let json = include_str!("../../../examples/example.json");
        serde_json::from_str(json).expect("example schema should parse")
    }

    #[test]
    fn produces_a_zip_container() {
        let bytes = render(&example()).expect("render should succeed");
        assert_eq!(&bytes[..2], b"PK", "a .docx is a zip archive");
        assert!(bytes.len() > 1_000);
    }

    #[test]
    fn body_carries_headings_and_content() {
        let body = build_body(&example());
        assert!(body.contains("Ada Lovelace"));
        assert!(body.contains(r#"<w:pStyle w:val="Heading1"/>"#));
        assert!(body.contains("PROFILE"));
        assert!(body.contains("EDUCATION"));
    }

    #[test]
    fn escapes_xml_metacharacters() {
        let mut schema = example();
        schema.person.name = "A & B <script>".into();
        let body = build_body(&schema);
        assert!(body.contains("A &amp; B &lt;script&gt;"));
        assert!(!body.contains("<script>"));
    }

    #[test]
    fn an_empty_schema_still_produces_a_valid_document() {
        let bytes = render(&CVSchema::default()).expect("render should succeed");
        assert_eq!(&bytes[..2], b"PK");
    }
}
