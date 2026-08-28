//! Text measurement and line breaking.
//!
//! Measurement uses the Adobe AFM tables in [`crate::style::metrics`], kern pairs
//! included, because the reference renderer does. Widths drive line breaking, so
//! a systematic error here shifts every wrap point in the document.

use oxidize_pdf::Font;

use crate::layout::linebreak;

use crate::style::metrics::{
    HELVETICA_BOLD_KERN, HELVETICA_BOLD_WIDTHS, HELVETICA_KERN, HELVETICA_WIDTHS,
};
use crate::style::types::TextStyle;

/// Map a character to its WinAnsi byte, which is the encoding used for the
/// standard PDF fonts. Unmappable characters fall back to `?`, matching what a
/// viewer would show.
pub fn winansi_byte(ch: char) -> u8 {
    let cp = ch as u32;
    match cp {
        0x20..=0x7e | 0xa0..=0xff => cp as u8,
        0x20ac => 0x80,
        0x201a => 0x82,
        0x0192 => 0x83,
        0x201e => 0x84,
        0x2026 => 0x85,
        0x2020 => 0x86,
        0x2021 => 0x87,
        0x02c6 => 0x88,
        0x2030 => 0x89,
        0x0160 => 0x8a,
        0x2039 => 0x8b,
        0x0152 => 0x8c,
        0x017d => 0x8e,
        0x2018 => 0x91,
        0x2019 => 0x92,
        0x201c => 0x93,
        0x201d => 0x94,
        0x2022 => 0x95,
        0x2013 => 0x96,
        0x2014 => 0x97,
        0x02dc => 0x98,
        0x2122 => 0x99,
        0x0161 => 0x9a,
        0x203a => 0x9b,
        0x0153 => 0x9c,
        0x017e => 0x9e,
        0x0178 => 0x9f,
        _ => b'?',
    }
}

fn tables(font: &Font) -> (&'static [u16; 256], &'static [(u8, u8, i16)]) {
    match font {
        Font::HelveticaBold => (&HELVETICA_BOLD_WIDTHS, HELVETICA_BOLD_KERN),
        _ => (&HELVETICA_WIDTHS, HELVETICA_KERN),
    }
}

fn kern_units(table: &'static [(u8, u8, i16)], left: u8, right: u8) -> i32 {
    table
        .binary_search_by(|probe| (probe.0, probe.1).cmp(&(left, right)))
        .map(|index| table[index].2 as i32)
        .unwrap_or(0)
}

pub fn encode(text: &str) -> Vec<u8> {
    text.chars().map(winansi_byte).collect()
}

/// The list marker the reference emits as its own text run.
const BULLET_PREFIX: [u8; 2] = [0x95, b' '];

/// Byte offsets at which a new shaping run begins.
///
/// The reference builds a bullet line from two JSX children — the literal
/// `"\u{2022} "` and the item text — which become two runs in the attributed
/// string. Kerning does not cross that seam, so a line like "• Work" keeps the
/// full space width instead of applying the `space`+`W` pair.
pub fn run_boundaries(bytes: &[u8]) -> Vec<usize> {
    if bytes.starts_with(&BULLET_PREFIX) {
        vec![BULLET_PREFIX.len()]
    } else {
        Vec::new()
    }
}

/// Per-character advances for an encoded line, in points.
///
/// Each entry is the distance from one character's origin to the next: its own
/// width, plus letter spacing, plus word spacing on a space, plus the kern
/// against the following character.
pub fn advances(bytes: &[u8], style: &TextStyle, word_spacing: f64) -> Vec<f64> {
    let boundaries = run_boundaries(bytes);
    let (widths, kern) = tables(&style.font);
    let scale = style.size / 1000.0;

    bytes
        .iter()
        .enumerate()
        .map(|(index, &byte)| {
            let mut advance = (widths[byte as usize] as f64) * scale + style.letter_spacing;
            if byte == b' ' {
                advance += word_spacing;
            }
            // Kerning never crosses a run boundary.
            if let Some(&next) = bytes.get(index + 1)
                && !boundaries.contains(&(index + 1))
            {
                advance += (kern_units(kern, byte, next) as f64) * scale;
            }
            advance
        })
        .collect()
}

/// Kerning adjustment between two encoded characters, in points.
pub fn kern_between(style: &TextStyle, left: u8, right: u8) -> f64 {
    let (_, kern) = tables(&style.font);
    (kern_units(kern, left, right) as f64) * (style.size / 1000.0)
}

/// Width of `text` when set in `style`, kerning and letter spacing included.
pub fn text_width(text: &str, style: &TextStyle) -> f64 {
    advances(&encode(text), style, 0.0).iter().sum()
}

/// react-pdf starts at tolerance 4 and relaxes in steps of 5 up to 50 before
/// giving up on Knuth & Plass and falling back to best fit.
const TOLERANCE_START: f64 = 4.0;
const TOLERANCE_STEP: f64 = 5.0;
const TOLERANCE_LIMIT: f64 = 50.0;

/// Best-fit breaking, used when no feasible Knuth & Plass solution exists.
/// A port of the reference `applyBestFit` / `getNextBreakpoint`.
fn best_fit(nodes: &[linebreak::Node], line_length: f64) -> Vec<usize> {
    use linebreak::Kind;

    let mut breaks = Vec::new();
    let mut offset = 0usize;

    while offset < nodes.len() {
        let remaining = &nodes[offset..];
        let mut position: Option<usize> = None;
        let mut minimum_badness = f64::INFINITY;
        let (mut width, mut stretch, mut shrink) = (0.0f64, 0.0f64, 0.0f64);
        let mut overfull = false;

        for (index, node) in remaining.iter().enumerate() {
            match node.kind {
                Kind::Box => width += node.width,
                Kind::Glue => {
                    width += node.width;
                    stretch += node.stretch;
                    shrink += node.shrink;
                }
                Kind::Penalty => {}
            }

            if width - shrink > line_length {
                if position.is_none() {
                    let mut j = if index == 0 { index + 1 } else { index };
                    while j < remaining.len()
                        && matches!(remaining[j].kind, Kind::Glue | Kind::Penalty)
                    {
                        j += 1;
                    }
                    position = Some(j - 1);
                }
                overfull = true;
                break;
            }

            if matches!(node.kind, Kind::Penalty | Kind::Glue) {
                let ratio = if width < line_length {
                    if stretch - node.stretch > 0.0 {
                        (line_length - width) / stretch
                    } else {
                        f64::INFINITY
                    }
                } else if width > line_length {
                    if shrink - node.shrink > 0.0 {
                        (line_length - width) / shrink
                    } else {
                        f64::INFINITY
                    }
                } else {
                    0.0
                };

                let badness = 100.0 * ratio.abs().powi(3)
                    + if node.kind == Kind::Penalty {
                        node.penalty
                    } else {
                        0.0
                    };
                if minimum_badness >= badness {
                    position = Some(index);
                    minimum_badness = badness;
                }
            }
        }

        match position {
            Some(index) if overfull => {
                breaks.push(offset + index);
                offset += index + 1;
            }
            _ => break,
        }
    }

    breaks
}

/// Break `text` into lines that fit `width`, using the reference's algorithm.
///
/// Hyphenation is disabled in the reference, so every word is an unbreakable box
/// and no hyphen penalties enter the node list.
pub fn wrap(text: &str, width: f64, style: &TextStyle) -> Vec<String> {
    let mut lines = Vec::new();

    for hard_line in text.split('\n') {
        let words: Vec<&str> = hard_line.split_whitespace().collect();
        if words.is_empty() {
            lines.push(String::new());
            continue;
        }

        let space = text_width(" ", style);
        let mut nodes = Vec::with_capacity(words.len() * 2 + 2);
        for (index, word) in words.iter().enumerate() {
            nodes.push(linebreak::Node::text_box(text_width(word, style)));
            if index + 1 < words.len() {
                nodes.push(linebreak::Node::glue(space));
            }
        }
        linebreak::terminate(&mut nodes);

        let mut tolerance = TOLERANCE_START;
        let mut breaks = linebreak::break_lines(&nodes, width, tolerance);
        while breaks.as_ref().is_none_or(Vec::is_empty) && tolerance < TOLERANCE_LIMIT {
            tolerance += TOLERANCE_STEP;
            breaks = linebreak::break_lines(&nodes, width, tolerance);
        }
        let breaks = match breaks {
            Some(found) if !found.is_empty() => found,
            _ => best_fit(&nodes, width),
        };

        let last_node = nodes.len() - 1;
        let mut start = 0usize;
        for position in breaks {
            if position >= last_node {
                continue;
            }
            // Glue between word i and i+1 sits at node index 2i + 1.
            let word = position / 2;
            if word < start {
                continue;
            }
            lines.push(words[start..=word].join(" "));
            start = word + 1;
        }
        if start < words.len() {
            lines.push(words[start..].join(" "));
        }
    }

    lines
}

/// Split `items` into `rows` groups of roughly equal length, preserving order.
/// Mirrors the reference `chunk()` in `InlineListBlock`.
pub fn chunk(items: &[String], rows: usize) -> Vec<Vec<String>> {
    let rows = rows.max(1);
    if items.is_empty() {
        return Vec::new();
    }
    let per_row = items.len().div_ceil(rows);
    items.chunks(per_row).map(<[String]>::to_vec).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxidize_pdf::Font;

    fn body() -> TextStyle {
        TextStyle::prose_body()
    }

    #[test]
    fn maps_typographic_characters_to_winansi() {
        assert_eq!(winansi_byte('A'), b'A');
        assert_eq!(winansi_byte('\u{2022}'), 0x95, "bullet");
        assert_eq!(winansi_byte('\u{2013}'), 0x96, "en dash");
        assert_eq!(winansi_byte('\u{2014}'), 0x97, "em dash");
        assert_eq!(winansi_byte('\u{00b7}'), 0xb7, "middot");
        assert_eq!(winansi_byte('\u{4e2d}'), b'?', "unmappable");
    }

    #[test]
    fn measures_against_reference_widths() {
        // Widths taken from the reference renderer for Helvetica at 10pt.
        let width = text_width(
            "Environments (GDPR) \u{b7} Rust Smart Contracts (Canisters) \u{b7} Vector Search",
            &body(),
        );
        assert!((width - 325.55).abs() < 0.01, "got {width}");
    }

    #[test]
    fn applies_kerning() {
        let style = body();
        // "Wa" kerns; measuring the pair must be narrower than the two glyphs.
        let pair = text_width("Wa", &style);
        let apart = text_width("W", &style) + text_width("a", &style);
        assert!(pair < apart, "expected kerning: {pair} vs {apart}");
    }

    #[test]
    fn does_not_kern_across_the_bullet_run_boundary() {
        let style = body();
        let bytes = encode("\u{2022} Work");
        let total: f64 = advances(&bytes, &style, 0.0).iter().sum();
        let unkerned =
            text_width("\u{2022}", &style) + text_width(" ", &style) + text_width("Work", &style);
        assert!(
            (total - unkerned).abs() < 0.01,
            "space+W must not kern across the run seam: {total} vs {unkerned}"
        );
    }

    #[test]
    fn bold_and_regular_use_different_tables() {
        let mut bold = body();
        bold.font = Font::HelveticaBold;
        assert!(text_width("Bold", &bold) > text_width("Bold", &body()));
    }

    #[test]
    fn wraps_without_exceeding_the_column() {
        let style = body();
        let text = "The quick brown fox jumps over the lazy dog and keeps on running \
                    well past the end of any reasonable column width";
        for line in wrap(text, 200.0, &style) {
            // Knuth & Plass may set a line tight, but never by more than its glue
            // can shrink.
            assert!(text_width(&line, &style) < 215.0, "runaway line: {line}");
        }
    }

    #[test]
    fn wrapping_preserves_every_word() {
        let style = body();
        let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu";
        let joined = wrap(text, 90.0, &style).join(" ");
        assert_eq!(joined, text);
    }

    #[test]
    fn an_overlong_word_overflows_rather_than_splitting() {
        let style = body();
        let lines = wrap("supercalifragilisticexpialidocious", 20.0, &style);
        assert_eq!(lines, vec!["supercalifragilisticexpialidocious"]);
    }

    #[test]
    fn chunks_items_into_balanced_rows() {
        let items: Vec<String> = (1..=7).map(|n| n.to_string()).collect();
        let rows = chunk(&items, 2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].len(), 4, "the first row takes the extra item");
        assert_eq!(rows[1].len(), 3);
        assert!(chunk(&[], 2).is_empty());
    }
}
