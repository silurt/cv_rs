//! The document model: the shapes a CV section can take.
//!
//! Deliberately format-independent — the PDF and DOCX backends both consume
//! this, so a change to what a section *is* happens in one place.
//!
//! A section renders exactly one block. Adding a rendering shape means adding a
//! variant here and a match arm in `utils::render_block` — nothing else changes.

/// `lead` is the larger, justified profile treatment; `body` is running text.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProseVariant {
    Lead,
    Body,
}

/// `Ruled` draws the left rule used by Experience; `Plain` is the Education
/// treatment.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntryVariant {
    Ruled,
    Plain,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Entry {
    pub title: String,
    /// Rendered one per line under the title.
    pub meta: Vec<String>,
    pub summary: Option<String>,
    pub bullets: Vec<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LabelValueRow {
    pub label: String,
    pub value: String,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Block {
    /// Paragraphs of running text.
    Prose {
        paragraphs: Vec<String>,
        variant: ProseVariant,
    },
    /// Items joined onto one or more balanced lines, e.g. "A · B · C".
    InlineList {
        items: Vec<String>,
        separator: String,
        rows: usize,
    },
    /// A flat list of bulleted lines.
    BulletList {
        items: Vec<String>,
        trailing_spacer: bool,
    },
    /// Repeated title/meta/summary/bullets records.
    EntryList {
        entries: Vec<Entry>,
        variant: EntryVariant,
        /// Whether an entry may split across a page boundary.
        wrap: bool,
    },
    /// Two-column label/value rows, e.g. the skills table.
    LabelValue { rows: Vec<LabelValueRow> },
}

/// A heading plus its block. `margin_top` carries the per-section nudges.
#[derive(Clone, PartialEq, Debug)]
pub struct Section {
    pub title: String,
    pub margin_top: f64,
    pub block: Block,
}

impl Section {
    pub fn new(title: impl Into<String>, block: Block) -> Self {
        Self {
            title: title.into(),
            margin_top: 0.0,
            block,
        }
    }

    pub fn with_margin_top(mut self, margin_top: f64) -> Self {
        self.margin_top = margin_top;
        self
    }
}
