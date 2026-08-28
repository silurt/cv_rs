//! The layout engine.
//!
//! Positions are measured downward from the top of the page: `cursor` is the
//! distance from the top edge to the next unused point. PDF's own origin is
//! bottom-left, so the flip happens once, at the moment of drawing.
//!
//! Working top-down means the numbers here read in the same order the document
//! does — a heading at 120 is above a paragraph at 140 — which makes the layout
//! constants legible and the arithmetic hard to get backwards.

use oxidize_pdf::structure::{StandardStructureType, StructTree, StructureElement};
use oxidize_pdf::text::KernedRun;
use oxidize_pdf::{Document, Page};

use crate::layout::utils::{encode, kern_between, run_boundaries, text_width, wrap};
use crate::style::types::{
    Align, ENTRY_RULE_WIDTH, PAGE_HEIGHT, PAGE_PADDING, PAGE_WIDTH, TextStyle,
};

/// A left rule being drawn beside an entry. Held open across page breaks so a
/// split entry gets a rule segment on each page it touches.
struct RuleSpan {
    x: f64,
    top: f64,
}

pub struct Renderer<'a> {
    doc: &'a mut Document,
    page: Page,
    /// Index of the page being built, needed to attach marked content to the
    /// structure tree.
    page_index: usize,
    cursor: f64,
    rule: Option<RuleSpan>,
    /// The Tagged PDF structure tree. Every line of text is emitted inside a
    /// marked-content sequence whose id is attached to the element on top of
    /// `open`, so extractors get a logical document rather than loose glyphs.
    tree: StructTree,
    open: Vec<usize>,
}

impl<'a> Renderer<'a> {
    pub fn new(doc: &'a mut Document) -> Self {
        let mut tree = StructTree::new();
        let root = tree.set_root(StructureElement::new(StandardStructureType::Document));

        Self {
            doc,
            page: Page::a4(),
            page_index: 0,
            cursor: PAGE_PADDING,
            rule: None,
            tree,
            open: vec![root],
        }
    }

    // ---- document structure ----

    /// Open a structure element. Text drawn until the matching [`Self::pop`] is
    /// attached to it.
    pub fn push(&mut self, structure_type: StandardStructureType) {
        let parent = *self.open.last().expect("the root element is never popped");
        if let Ok(index) = self
            .tree
            .add_child(parent, StructureElement::new(structure_type))
        {
            self.open.push(index);
        }
    }

    /// Close the innermost structure element.
    pub fn pop(&mut self) {
        // The root is pushed at construction and must outlive every section.
        if self.open.len() > 1 {
            self.open.pop();
        }
    }

    /// The PDF tag name for the element currently receiving content.
    fn current_tag(&self) -> String {
        self.open
            .last()
            .and_then(|index| self.tree.get(*index))
            .map_or_else(
                || StandardStructureType::P.as_pdf_name().to_string(),
                |element| element.structure_type.as_pdf_name().to_string(),
            )
    }

    fn attach(&mut self, mcid: u32) {
        let page_index = self.page_index;
        if let Some(index) = self.open.last()
            && let Some(element) = self.tree.get_mut(*index)
        {
            element.add_mcid(page_index, mcid);
        }
    }

    // ---- geometry ----

    pub fn cursor(&self) -> f64 {
        self.cursor
    }

    pub fn page_bottom(&self) -> f64 {
        PAGE_HEIGHT - PAGE_PADDING
    }

    pub fn remaining(&self) -> f64 {
        self.page_bottom() - self.cursor
    }

    pub fn advance(&mut self, height: f64) {
        self.cursor += height;
    }

    /// Move the cursor to an absolute position. Used by multi-column rows, where
    /// each column starts from the same top edge.
    pub fn set_cursor(&mut self, y: f64) {
        self.cursor = y;
    }

    /// Break to a new page unless `needed` fits, or unless nothing has been
    /// placed yet — an item taller than a whole page must not loop forever.
    pub fn ensure(&mut self, needed: f64) {
        if needed > self.remaining() && self.cursor > PAGE_PADDING {
            self.break_page();
        }
    }

    pub fn break_page(&mut self) {
        // Close off any open rule at the foot of the page before swapping it out.
        // A split entry's fragment fills the rest of the page, so its rule runs
        // to the bottom margin rather than stopping at the last line placed.
        if let Some(span) = self.rule.take() {
            let bottom = self.page_bottom();
            self.draw_rule_segment(span.x, span.top, bottom);
            self.rule = Some(RuleSpan {
                x: span.x,
                top: PAGE_PADDING,
            });
        }

        let finished = std::mem::replace(&mut self.page, Page::a4());
        self.doc.add_page(finished);
        self.page_index += 1;
        self.cursor = PAGE_PADDING;
    }

    /// Emit the final page and, when tagging is enabled, install the structure
    /// tree. Consumes the renderer.
    pub fn finish(self, tagged: bool) {
        self.doc.add_page(self.page);
        if tagged {
            self.doc.set_struct_tree(self.tree);
        }
    }

    // ---- text ----

    /// Draw a single pre-wrapped line at the current cursor, without advancing.
    ///
    /// The line is emitted as one or more positioned runs, split wherever a kern
    /// pair applies. oxidize-pdf writes plain `Tj` strings with no kerning of
    /// their own, so placing each run explicitly is what reproduces the
    /// reference's glyph positions rather than merely its line breaks.
    fn draw_line(&mut self, text: &str, x: f64, width: f64, style: &TextStyle, justify: bool) {
        if text.trim().is_empty() {
            return;
        }

        let baseline = self.cursor + style.baseline_offset();
        let y = PAGE_HEIGHT - baseline;
        let natural = text_width(text, style);
        let spaces = text.chars().filter(|c| *c == ' ').count();

        // A line the breaker chose by shrinking its glue is set tight so it ends
        // exactly on the column edge. Justified lines additionally stretch. Both
        // are distributed over the inter-word spaces via Tw.
        let word_spacing = if spaces == 0 {
            0.0
        } else if natural > width || justify {
            (width - natural) / spaces as f64
        } else {
            0.0
        };

        let measured = natural + (word_spacing * spaces as f64);

        let origin = match style.align {
            Align::Center => x + ((width - measured) / 2.0),
            _ => x,
        };

        // Emit the line as one `TJ` array: the runs between kern pairs, with
        // each pair's adjustment carried inline. One text object per line keeps
        // the content stream readable and matches what a PDF producer writes;
        // the alternative is a separately positioned text object per kern pair.
        let bytes = encode(text);
        let boundaries = run_boundaries(&bytes);
        let chars: Vec<char> = text.chars().collect();

        let mut runs: Vec<KernedRun<'_>> = Vec::new();
        let mut segments: Vec<(String, f64)> = Vec::new();
        let mut current = String::new();

        for (index, ch) in chars.iter().enumerate() {
            current.push(*ch);

            let kern = match bytes.get(index + 1) {
                Some(&next) if !boundaries.contains(&(index + 1)) => {
                    kern_between(style, bytes[index], next)
                }
                _ => 0.0,
            };

            if kern != 0.0 {
                // TJ adjustments are in thousandths of an em and move the
                // following glyphs left, so a negative kern is a positive
                // adjustment.
                let adjustment = -kern / style.size * 1000.0;
                segments.push((std::mem::take(&mut current), adjustment));
            }
        }
        if !current.is_empty() {
            segments.push((current, 0.0));
        }
        for (text, adjustment) in &segments {
            runs.push(KernedRun::with_adjustment(text, *adjustment));
        }

        let font = style.font.clone();
        let color = style.color;
        let size = style.size;
        let letter_spacing = style.letter_spacing;

        // Wrap the line in a marked-content sequence so it belongs to a node of
        // the structure tree rather than floating loose on the page.
        let tag = self.current_tag();
        let mcid = self.page.begin_marked_content(&tag).ok();

        let text_ctx = self.page.text();
        text_ctx
            .set_font(font, size)
            .set_fill_color(color)
            .set_character_spacing(letter_spacing)
            .set_word_spacing(word_spacing)
            .at(origin, y);
        let _ = text_ctx.write_kerned(&runs);

        let _ = self.page.end_marked_content();
        if let Some(mcid) = mcid {
            self.attach(mcid);
        }
    }

    /// Draw one pre-composed line verbatim and advance past it.
    ///
    /// Unlike [`Self::paragraph`] this neither re-wraps nor normalises runs of
    /// whitespace, so a line whose spacing is significant — the header's
    /// `  \u{2022}  `-separated contact row — keeps its exact width.
    pub fn line(&mut self, text: &str, x: f64, width: f64, style: &TextStyle) {
        self.ensure(style.line_box() + style.margin_bottom);
        self.draw_line(
            text,
            x + style.padding_left,
            width - style.padding_left,
            style,
            false,
        );
        self.cursor += style.line_box() + style.margin_bottom;
    }

    /// Wrap `text` into `width` and draw it, breaking pages between lines.
    /// Advances the cursor past the block, including its bottom margin.
    pub fn paragraph(&mut self, text: &str, x: f64, width: f64, style: &TextStyle) {
        let inner_x = x + style.padding_left;
        let inner_width = width - style.padding_left;
        let lines = wrap(text, inner_width, style);
        let last = lines.len().saturating_sub(1);

        for (index, line) in lines.iter().enumerate() {
            // The block's bottom margin is part of its box, so a final line that
            // would leave no room for it moves to the next page — as it does in
            // the reference, where the whole element is measured before placing.
            let trailing = if index == last {
                style.margin_bottom
            } else {
                0.0
            };
            self.ensure(style.line_box() + trailing);
            let justify = style.align == Align::Justify && index < last;
            self.draw_line(line, inner_x, inner_width, style, justify);
            self.cursor += style.line_box();
        }

        self.cursor += style.margin_bottom;
    }

    /// Height `paragraph` would occupy, without drawing anything.
    pub fn measure_paragraph(&self, text: &str, width: f64, style: &TextStyle) -> f64 {
        let lines = wrap(text, width - style.padding_left, style);
        (lines.len() as f64 * style.line_box()) + style.margin_bottom
    }

    // ---- graphics ----

    fn draw_rule_segment(&mut self, x: f64, top: f64, bottom: f64) {
        if bottom <= top {
            return;
        }
        // A left border of width w occupies x..x+w, so the stroke centre sits
        // half a width in.
        let centre = x + (ENTRY_RULE_WIDTH / 2.0);
        let y0 = PAGE_HEIGHT - top;
        let y1 = PAGE_HEIGHT - bottom;

        self.page
            .graphics()
            .set_line_width(ENTRY_RULE_WIDTH)
            .set_stroke_color(crate::style::types::rule())
            .move_to(centre, y0)
            .line_to(centre, y1)
            .stroke();
    }

    /// Begin a left rule at the current cursor.
    pub fn begin_rule(&mut self, x: f64) {
        self.rule = Some(RuleSpan {
            x,
            top: self.cursor,
        });
    }

    /// Close the open left rule at the current cursor.
    pub fn end_rule(&mut self) {
        if let Some(span) = self.rule.take() {
            let bottom = self.cursor;
            self.draw_rule_segment(span.x, span.top, bottom);
        }
    }

    /// A full-width horizontal rule at the current cursor, e.g. under the header.
    pub fn horizontal_rule(&mut self, line_width: f64) {
        let y = PAGE_HEIGHT - (self.cursor + (line_width / 2.0));
        self.page
            .graphics()
            .set_line_width(line_width)
            .set_stroke_color(crate::style::types::rule())
            .move_to(PAGE_PADDING, y)
            .line_to(PAGE_WIDTH - PAGE_PADDING, y)
            .stroke();
        self.cursor += line_width;
    }
}
