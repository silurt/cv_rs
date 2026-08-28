//! The fixed OOXML parts, and the style sheet the body refers to.
//!
//! Everything here is deliberately minimal: the parts a `.docx` must have to be
//! a valid package, plus named paragraph styles. Nothing decorative.

/// Escape the five XML metacharacters.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

pub const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#;

pub const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

pub const DOCUMENT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;

pub const DOCUMENT_OPEN: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#
);

/// A4 with the same 26pt margins the PDF uses, expressed in twentieths of a
/// point.
pub const DOCUMENT_CLOSE: &str = r#"<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="520" w:right="520" w:bottom="520" w:left="520"/></w:sectPr></w:body></w:document>"#;

/// One style definition. Sizes are in half-points.
fn style(id: &str, name: &str, half_points: u32, bold: bool, outline: Option<u32>) -> String {
    let bold = if bold { "<w:b/>" } else { "" };
    let outline = outline.map_or_else(String::new, |level| {
        format!("<w:outlineLvl w:val=\"{level}\"/>")
    });
    format!(
        "<w:style w:type=\"paragraph\" w:styleId=\"{id}\">\
<w:name w:val=\"{name}\"/>\
<w:pPr><w:spacing w:before=\"60\" w:after=\"60\"/>{outline}</w:pPr>\
<w:rPr><w:rFonts w:ascii=\"Helvetica\" w:hAnsi=\"Helvetica\"/><w:sz w:val=\"{half_points}\"/>{bold}</w:rPr>\
</w:style>"
    )
}

/// The style sheet.
///
/// `Heading1`–`Heading3` keep Word's built-in names and outline levels, because
/// that is what a parser looks for when it tries to find section boundaries. A
/// heading that is merely bold body text carries no such signal.
pub fn styles() -> String {
    let mut out = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
    );
    out.push_str(&style("Normal", "Normal", 20, false, None));
    out.push_str(&style("Title", "Title", 48, true, None));
    out.push_str(&style("Contact", "Contact", 20, false, None));
    out.push_str(&style("Heading1", "heading 1", 26, true, Some(0)));
    out.push_str(&style("Heading2", "heading 2", 24, true, Some(1)));
    out.push_str(&style("Heading3", "heading 3", 22, true, Some(2)));
    out.push_str(&style("Body", "Body Text", 20, false, None));
    out.push_str(&style("Meta", "Meta", 20, false, None));
    out.push_str(&style("Bullet", "List Paragraph", 20, false, None));
    out.push_str("</w:styles>");
    out
}
