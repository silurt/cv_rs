//! Rendering for each block type.

use core::document::types::{Block, Entry, EntryVariant, ProseVariant, Section};
use core::document::utils::contact_items;
use core::schema::types::{CVLinks, CVPerson};

use oxidize_pdf::structure::StandardStructureType as Tag;

use crate::layout::types::Renderer;
use crate::layout::utils::{chunk, text_width, wrap};
use crate::style::types::{
    ACHIEVEMENT_SPACER, ENTRY_PLAIN_MARGIN_BOTTOM, ENTRY_RULE_WIDTH, ENTRY_RULED_MARGIN_BOTTOM,
    ENTRY_RULED_PADDING_LEFT, HEADER_BORDER_WIDTH, HEADER_MARGIN_BOTTOM, HEADER_PADDING_BOTTOM,
    INLINE_LIST_SPACER, LABEL_VALUE_LABEL_WIDTH, LABEL_VALUE_ROW_MARGIN_BOTTOM, LABEL_VALUE_SPACER,
    PAGE_PADDING, PROSE_PARAGRAPH_GAP, SECTION_MARGIN_BOTTOM, SECTION_MIN_PRESENCE_AHEAD,
    TextStyle, content_width,
};

/// The header: name, the wrapping contact row, and the rule beneath.
///
/// The contact row is a centred flex row with wrapping, so items are atomic —
/// a single contact never breaks mid-string, and each resulting line is centred
/// independently.
pub fn render_header(renderer: &mut Renderer, person: &CVPerson, links: &CVLinks) {
    let name_style = TextStyle::name();
    renderer.push(Tag::H1);
    renderer.paragraph(
        &person.name.to_uppercase(),
        PAGE_PADDING,
        content_width(),
        &name_style,
    );
    renderer.pop();

    let contact_style = TextStyle::contact();
    let items = contact_items(person, links);

    // The reference prefixes every item after the first with the separator, so a
    // wrapped line begins with the bullet.
    let pieces: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            if index == 0 {
                item.clone()
            } else {
                format!("  \u{2022}  {item}")
            }
        })
        .collect();

    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for piece in &pieces {
        let candidate = format!("{current}{piece}");
        if text_width(&candidate, &contact_style) <= content_width() || current.is_empty() {
            current = candidate;
        } else {
            // An item that starts a wrapped line loses the separator's leading
            // whitespace, as it does in the reference's flex row.
            lines.push(std::mem::take(&mut current));
            current = piece.trim_start().to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }

    let count = lines.len();
    renderer.push(Tag::P);
    for (index, line) in lines.into_iter().enumerate() {
        // Where the row wraps, the break falls inside the next separator's
        // leading whitespace: one space is left behind as trailing space on this
        // line, and the line's first separator renders a space narrower. The
        // overall advance — and so the centring — is unchanged.
        let rendered = if index + 1 < count {
            line.replacen("  \u{2022}  ", " \u{2022}  ", 1) + " "
        } else {
            line
        };
        renderer.line(&rendered, PAGE_PADDING, content_width(), &contact_style);
    }
    renderer.pop();

    renderer.advance(HEADER_PADDING_BOTTOM);
    renderer.horizontal_rule(HEADER_BORDER_WIDTH);
    renderer.advance(HEADER_MARGIN_BOTTOM);
}

/// Height of one entry, used to keep non-wrapping entries whole.
fn measure_entry(renderer: &Renderer, entry: &Entry, variant: EntryVariant, width: f64) -> f64 {
    let ruled = variant == EntryVariant::Ruled;
    let mut height = 0.0;

    let title_style = if ruled {
        TextStyle::entry_title()
    } else {
        TextStyle::entry_title_small()
    };
    height += renderer.measure_paragraph(&entry.title, width, &title_style);

    for (index, line) in entry.meta.iter().enumerate() {
        let style = if index == 0 {
            if ruled {
                TextStyle::entry_meta()
            } else {
                TextStyle::entry_detail()
            }
        } else {
            TextStyle::entry_meta_muted()
        };
        height += renderer.measure_paragraph(line, width, &style);
    }

    if let Some(summary) = &entry.summary {
        height += renderer.measure_paragraph(summary, width, &TextStyle::entry_summary());
    }

    let bullet_style = TextStyle::bullet();
    for bullet in &entry.bullets {
        height += renderer.measure_paragraph(&format!("\u{2022} {bullet}"), width, &bullet_style);
    }

    height
}

fn render_entry(renderer: &mut Renderer, entry: &Entry, variant: EntryVariant, x: f64, width: f64) {
    let ruled = variant == EntryVariant::Ruled;

    let title_style = if ruled {
        TextStyle::entry_title()
    } else {
        TextStyle::entry_title_small()
    };
    renderer.push(Tag::H3);
    renderer.paragraph(&entry.title, x, width, &title_style);
    renderer.pop();

    for (index, line) in entry.meta.iter().enumerate() {
        let style = if index == 0 {
            if ruled {
                TextStyle::entry_meta()
            } else {
                TextStyle::entry_detail()
            }
        } else {
            TextStyle::entry_meta_muted()
        };
        renderer.push(Tag::P);
        renderer.paragraph(line, x, width, &style);
        renderer.pop();
    }

    if let Some(summary) = &entry.summary {
        renderer.push(Tag::P);
        renderer.paragraph(summary, x, width, &TextStyle::entry_summary());
        renderer.pop();
    }

    if !entry.bullets.is_empty() {
        let bullet_style = TextStyle::bullet();
        renderer.push(Tag::L);
        for bullet in &entry.bullets {
            renderer.push(Tag::LI);
            renderer.push(Tag::LBody);
            renderer.paragraph(&format!("\u{2022} {bullet}"), x, width, &bullet_style);
            renderer.pop();
            renderer.pop();
        }
        renderer.pop();
    }
}

pub fn render_block(renderer: &mut Renderer, block: &Block) {
    match block {
        Block::Prose {
            paragraphs,
            variant,
        } => {
            let base = match variant {
                ProseVariant::Lead => TextStyle::prose_lead(),
                ProseVariant::Body => TextStyle::prose_body(),
            };
            let last = paragraphs.len().saturating_sub(1);
            for (index, paragraph) in paragraphs.iter().enumerate() {
                let mut style = base.clone();
                if index < last {
                    style.margin_bottom = PROSE_PARAGRAPH_GAP;
                }
                renderer.push(Tag::P);
                renderer.paragraph(paragraph, PAGE_PADDING, content_width(), &style);
                renderer.pop();
            }
        }

        Block::InlineList {
            items,
            separator,
            rows,
        } => {
            let style = TextStyle::inline_list();
            let groups = chunk(items, *rows);
            let last = groups.len().saturating_sub(1);
            for (index, group) in groups.iter().enumerate() {
                let line = group.join(separator);
                if line.is_empty() {
                    continue;
                }
                renderer.push(Tag::P);
                renderer.paragraph(&line, PAGE_PADDING, content_width(), &style);
                renderer.pop();
                if index < last {
                    renderer.advance(INLINE_LIST_SPACER);
                }
            }
        }

        Block::BulletList {
            items,
            trailing_spacer,
        } => {
            let style = TextStyle::bullet();
            renderer.push(Tag::L);
            for item in items {
                renderer.push(Tag::LI);
                renderer.push(Tag::LBody);
                renderer.paragraph(
                    &format!("\u{2022} {item}"),
                    PAGE_PADDING,
                    content_width(),
                    &style,
                );
                renderer.pop();
                renderer.pop();
            }
            renderer.pop();
            if *trailing_spacer {
                renderer.advance(ACHIEVEMENT_SPACER);
            }
        }

        Block::EntryList {
            entries,
            variant,
            wrap: may_wrap,
        } => {
            let ruled = *variant == EntryVariant::Ruled;
            let x = if ruled {
                PAGE_PADDING + ENTRY_RULE_WIDTH + ENTRY_RULED_PADDING_LEFT
            } else {
                PAGE_PADDING
            };
            let width = content_width() - (x - PAGE_PADDING);
            let margin_bottom = if ruled {
                ENTRY_RULED_MARGIN_BOTTOM
            } else {
                ENTRY_PLAIN_MARGIN_BOTTOM
            };

            for entry in entries {
                renderer.push(Tag::Sect);
                if !*may_wrap {
                    let height = measure_entry(renderer, entry, *variant, width);
                    renderer.ensure(height);
                }
                if ruled {
                    renderer.begin_rule(PAGE_PADDING);
                }
                render_entry(renderer, entry, *variant, x, width);
                if ruled {
                    renderer.end_rule();
                }
                renderer.advance(margin_bottom);
                renderer.pop();
            }
        }

        Block::LabelValue { rows } => {
            let label_style = TextStyle::label_value_label();
            let value_style = TextStyle::label_value_value();
            let value_x = PAGE_PADDING + LABEL_VALUE_LABEL_WIDTH;
            let value_width = content_width() - LABEL_VALUE_LABEL_WIDTH;
            let last = rows.len().saturating_sub(1);
            renderer.push(Tag::L);

            for (index, row) in rows.iter().enumerate() {
                let label = format!("{}:", row.label);
                let value_lines = wrap(&row.value, value_width, &value_style);
                let height = (value_lines.len().max(1) as f64) * value_style.line_box();

                // The row is a flex row: both columns start on the same line, and
                // the row is as tall as the taller column.
                renderer.ensure(height);
                let top = renderer.cursor();

                renderer.push(Tag::LI);
                renderer.push(Tag::Lbl);
                renderer.paragraph(&label, PAGE_PADDING, LABEL_VALUE_LABEL_WIDTH, &label_style);
                renderer.pop();

                renderer.set_cursor(top);
                renderer.push(Tag::LBody);
                renderer.paragraph(&row.value, value_x, value_width, &value_style);
                renderer.pop();
                renderer.pop();

                renderer.advance(LABEL_VALUE_ROW_MARGIN_BOTTOM);
                if index < last {
                    renderer.advance(LABEL_VALUE_SPACER);
                }
            }
            renderer.pop();
        }
    }
}

/// Section chrome: the uppercase heading plus its block.
pub fn render_section(renderer: &mut Renderer, section: &Section) {
    renderer.advance(section.margin_top);

    let title_style = TextStyle::section_title();
    // react-pdf's minPresenceAhead: keep the heading with its content.
    renderer.ensure(title_style.line_box() + SECTION_MIN_PRESENCE_AHEAD);

    renderer.push(Tag::Sect);
    renderer.push(Tag::H2);
    renderer.paragraph(
        &section.title.to_uppercase(),
        PAGE_PADDING,
        content_width(),
        &title_style,
    );
    renderer.pop();

    render_block(renderer, &section.block);
    renderer.pop();
    renderer.advance(SECTION_MARGIN_BOTTOM);
}
