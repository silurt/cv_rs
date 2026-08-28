//! Rendering for each block type.

use core::schema::types::{CVLinks, CVPerson};

use crate::block::types::{Block, Entry, EntryVariant, ProseVariant, Section};
use crate::layout::types::Renderer;
use crate::layout::utils::{chunk, text_width, wrap};
use crate::style::types::{
    ACHIEVEMENT_SPACER, ENTRY_PLAIN_MARGIN_BOTTOM, ENTRY_RULE_WIDTH, ENTRY_RULED_MARGIN_BOTTOM,
    ENTRY_RULED_PADDING_LEFT, HEADER_BORDER_WIDTH, HEADER_MARGIN_BOTTOM, HEADER_PADDING_BOTTOM,
    INLINE_LIST_SPACER, LABEL_VALUE_LABEL_WIDTH, LABEL_VALUE_ROW_MARGIN_BOTTOM, LABEL_VALUE_SPACER,
    PAGE_PADDING, PROSE_PARAGRAPH_GAP, SECTION_MARGIN_BOTTOM, SECTION_MIN_PRESENCE_AHEAD,
    TextStyle, content_width,
};

/// Strip the scheme and `www.` from a link, matching the reference's
/// `normalizeLink`.
fn normalize_link(value: &str) -> String {
    let trimmed = value
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    trimmed.trim_start_matches("www.").to_string()
}

fn format_github(value: &str) -> String {
    let normalized = normalize_link(value);
    if normalized.starts_with("github.com/") {
        format!("GitHub: {normalized}")
    } else {
        format!("GitHub: github.com/{normalized}")
    }
}

fn format_linkedin(value: &str) -> String {
    let normalized = normalize_link(value);
    if normalized.starts_with("linkedin.com/") {
        format!("LinkedIn: {normalized}")
    } else {
        let handle = normalized.trim_start_matches("in/");
        format!("LinkedIn: linkedin.com/{handle}")
    }
}

fn format_portfolio(value: &str) -> String {
    format!("Portfolio: {}", normalize_link(value))
}

/// The contact items, in reference order, with empties dropped.
fn contact_items(person: &CVPerson, links: &CVLinks) -> Vec<String> {
    let mut items = vec![person.location.clone()];

    if !person.email.trim().is_empty() {
        items.push(format!("Email: {}", person.email));
    }
    // The public schema carries no phone at all; an unconditional line here
    // would render "Phone: " with nothing after it.
    if let Some(phone) = person.phone.as_ref().filter(|p| !p.trim().is_empty()) {
        items.push(format!("Phone: {phone}"));
    }
    if !links.github.trim().is_empty() {
        items.push(format_github(&links.github));
    }
    if !links.linkedin.trim().is_empty() {
        items.push(format_linkedin(&links.linkedin));
    }
    if !links.portfolio.trim().is_empty() {
        items.push(format_portfolio(&links.portfolio));
    }

    items.retain(|item| !item.trim().is_empty());
    items
}

/// The header: name, the wrapping contact row, and the rule beneath.
///
/// The contact row is a centred flex row with wrapping, so items are atomic —
/// a single contact never breaks mid-string, and each resulting line is centred
/// independently.
pub fn render_header(renderer: &mut Renderer, person: &CVPerson, links: &CVLinks) {
    let name_style = TextStyle::name();
    renderer.paragraph(
        &person.name.to_uppercase(),
        PAGE_PADDING,
        content_width(),
        &name_style,
    );

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
    renderer.paragraph(&entry.title, x, width, &title_style);

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
        renderer.paragraph(line, x, width, &style);
    }

    if let Some(summary) = &entry.summary {
        renderer.paragraph(summary, x, width, &TextStyle::entry_summary());
    }

    let bullet_style = TextStyle::bullet();
    for bullet in &entry.bullets {
        renderer.paragraph(&format!("\u{2022} {bullet}"), x, width, &bullet_style);
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
                renderer.paragraph(paragraph, PAGE_PADDING, content_width(), &style);
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
                renderer.paragraph(&line, PAGE_PADDING, content_width(), &style);
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
            for item in items {
                renderer.paragraph(
                    &format!("\u{2022} {item}"),
                    PAGE_PADDING,
                    content_width(),
                    &style,
                );
            }
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
            }
        }

        Block::LabelValue { rows } => {
            let label_style = TextStyle::label_value_label();
            let value_style = TextStyle::label_value_value();
            let value_x = PAGE_PADDING + LABEL_VALUE_LABEL_WIDTH;
            let value_width = content_width() - LABEL_VALUE_LABEL_WIDTH;
            let last = rows.len().saturating_sub(1);

            for (index, row) in rows.iter().enumerate() {
                let label = format!("{}:", row.label);
                let value_lines = wrap(&row.value, value_width, &value_style);
                let height = (value_lines.len().max(1) as f64) * value_style.line_box();

                // The row is a flex row: both columns start on the same line, and
                // the row is as tall as the taller column.
                renderer.ensure(height);
                let top = renderer.cursor();

                renderer.paragraph(&label, PAGE_PADDING, LABEL_VALUE_LABEL_WIDTH, &label_style);

                renderer.set_cursor(top);
                renderer.paragraph(&row.value, value_x, value_width, &value_style);

                renderer.advance(LABEL_VALUE_ROW_MARGIN_BOTTOM);
                if index < last {
                    renderer.advance(LABEL_VALUE_SPACER);
                }
            }
        }
    }
}

/// Section chrome: the uppercase heading plus its block.
pub fn render_section(renderer: &mut Renderer, section: &Section) {
    renderer.advance(section.margin_top);

    let title_style = TextStyle::section_title();
    // react-pdf's minPresenceAhead: keep the heading with its content.
    renderer.ensure(title_style.line_box() + SECTION_MIN_PRESENCE_AHEAD);
    renderer.paragraph(
        &section.title.to_uppercase(),
        PAGE_PADDING,
        content_width(),
        &title_style,
    );

    render_block(renderer, &section.block);
    renderer.advance(SECTION_MARGIN_BOTTOM);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person() -> CVPerson {
        CVPerson {
            name: "Ada Lovelace".into(),
            location: "London, UK".into(),
            email: "ada@example.com".into(),
            phone: Some("+44 20 7946 0000".into()),
        }
    }

    #[test]
    fn normalises_link_schemes() {
        assert_eq!(normalize_link("https://github.com/x"), "github.com/x");
        assert_eq!(normalize_link("http://www.example.com"), "example.com");
        assert_eq!(normalize_link("www.example.com"), "example.com");
    }

    #[test]
    fn formats_links_without_doubling_the_host() {
        assert_eq!(
            format_github("https://github.com/x"),
            "GitHub: github.com/x"
        );
        assert_eq!(format_github("x"), "GitHub: github.com/x");
        assert_eq!(
            format_linkedin("www.linkedin.com/in/x"),
            "LinkedIn: linkedin.com/in/x"
        );
        assert_eq!(format_linkedin("in/x"), "LinkedIn: linkedin.com/x");
        assert_eq!(format_portfolio("https://x.dev"), "Portfolio: x.dev");
    }

    #[test]
    fn drops_empty_contact_fields() {
        let links = CVLinks {
            github: "https://github.com/x".into(),
            linkedin: String::new(),
            portfolio: "   ".into(),
        };
        let items = contact_items(&person(), &links);
        assert_eq!(
            items,
            vec![
                "London, UK",
                "Email: ada@example.com",
                "Phone: +44 20 7946 0000",
                "GitHub: github.com/x",
            ]
        );
    }

    #[test]
    fn omits_the_phone_line_when_there_is_no_phone() {
        let mut without = person();
        without.phone = None;
        let items = contact_items(&without, &CVLinks::default());
        assert!(
            !items.iter().any(|item| item.starts_with("Phone")),
            "a public CV must not render an empty phone line: {items:?}"
        );
    }
}
