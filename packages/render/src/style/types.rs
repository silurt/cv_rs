//! A direct port of the reference `pdfStyles.ts`.
//!
//! Every number here is the CSS value from that stylesheet. Keeping them literal
//! (rather than derived from a spacing scale) is deliberate: when the reference
//! stylesheet changes, the diff against this file should be obvious.

use oxidize_pdf::{Color, Font};

// ---- Page ----
pub const PAGE_WIDTH: f64 = 595.28;
pub const PAGE_HEIGHT: f64 = 841.89;
pub const PAGE_PADDING: f64 = 26.0;
pub const BASE_LINE_HEIGHT: f64 = 1.4;

/// Distance from the top of a line box down to the baseline, as a fraction of
/// the font size.
///
/// Note this does not depend on line height: the reference puts a line's ascent
/// at the top of its box and lets the extra leading fall below the baseline, so
/// a taller line pushes the *following* line down rather than shifting its own
/// glyphs. Calibrated against the reference render.
pub const ASCENT: f64 = 0.887;

/// react-pdf renders a `minPresenceAhead` of 50pt on every section heading, so a
/// heading never strands at the foot of a page.
pub const SECTION_MIN_PRESENCE_AHEAD: f64 = 50.0;

pub fn content_width() -> f64 {
    PAGE_WIDTH - (PAGE_PADDING * 2.0)
}

// ---- Colours ----
pub fn ink() -> Color {
    Color::Rgb(0.0, 0.0, 0.0)
}
/// `#333`
pub fn ink_soft() -> Color {
    Color::Rgb(0.2, 0.2, 0.2)
}
/// `#555`
pub fn ink_muted() -> Color {
    Color::Rgb(0.333_333, 0.333_333, 0.333_333)
}
/// `#ccc`
pub fn rule() -> Color {
    Color::Rgb(0.8, 0.8, 0.8)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    Left,
    Center,
    Justify,
}

/// One resolved text style — the Rust equivalent of a `StyleSheet.create` entry.
#[derive(Clone, Debug)]
pub struct TextStyle {
    pub font: Font,
    pub size: f64,
    pub line_height: f64,
    pub color: Color,
    pub letter_spacing: f64,
    pub align: Align,
    /// Left inset applied to every line, in points.
    pub padding_left: f64,
    /// Space added after the last line of the block.
    pub margin_bottom: f64,
}

impl TextStyle {
    fn base(font: Font, size: f64) -> Self {
        Self {
            font,
            size,
            line_height: BASE_LINE_HEIGHT,
            color: ink(),
            letter_spacing: 0.0,
            align: Align::Left,
            padding_left: 0.0,
            margin_bottom: 0.0,
        }
    }

    /// Height of one line box.
    pub fn line_box(&self) -> f64 {
        self.size * self.line_height
    }

    /// Offset from the top of the line box down to the baseline.
    pub fn baseline_offset(&self) -> f64 {
        self.size * ASCENT
    }

    // ---- header ----
    pub fn name() -> Self {
        Self {
            align: Align::Center,
            margin_bottom: 8.0,
            ..Self::base(Font::HelveticaBold, 24.0)
        }
    }
    pub fn contact() -> Self {
        Self {
            align: Align::Center,
            color: ink_muted(),
            ..Self::base(Font::Helvetica, 10.0)
        }
    }

    // ---- section chrome ----
    pub fn section_title() -> Self {
        Self {
            letter_spacing: 0.5,
            margin_bottom: 6.0,
            ..Self::base(Font::HelveticaBold, 13.0)
        }
    }

    // ---- ProseBlock ----
    /// `profileText` — the larger, justified lead treatment.
    pub fn prose_lead() -> Self {
        Self {
            line_height: 1.35,
            align: Align::Justify,
            ..Self::base(Font::Helvetica, 11.0)
        }
    }
    /// `bodyText`
    pub fn prose_body() -> Self {
        Self::base(Font::Helvetica, 10.0)
    }

    // ---- InlineListBlock ----
    pub fn inline_list() -> Self {
        Self::base(Font::Helvetica, 10.0)
    }

    // ---- BulletListBlock ----
    pub fn bullet() -> Self {
        Self {
            line_height: 1.3,
            margin_bottom: 2.0,
            padding_left: 12.0,
            ..Self::base(Font::Helvetica, 10.0)
        }
    }

    // ---- EntryListBlock ----
    pub fn entry_title() -> Self {
        Self::base(Font::HelveticaBold, 12.0)
    }
    pub fn entry_title_small() -> Self {
        Self::base(Font::HelveticaBold, 11.0)
    }
    pub fn entry_meta() -> Self {
        Self {
            color: ink_muted(),
            margin_bottom: 4.0,
            ..Self::base(Font::Helvetica, 10.0)
        }
    }
    pub fn entry_detail() -> Self {
        Self {
            color: ink_soft(),
            ..Self::base(Font::Helvetica, 10.0)
        }
    }
    pub fn entry_meta_muted() -> Self {
        Self {
            color: ink_muted(),
            ..Self::base(Font::Helvetica, 10.0)
        }
    }
    pub fn entry_summary() -> Self {
        Self {
            color: ink_soft(),
            margin_bottom: 4.0,
            ..Self::base(Font::Helvetica, 10.0)
        }
    }

    // ---- LabelValueBlock ----
    pub fn label_value_label() -> Self {
        Self::base(Font::HelveticaBold, 10.0)
    }
    pub fn label_value_value() -> Self {
        Self::base(Font::Helvetica, 10.0)
    }
}

// ---- Box metrics that are not text styles ----

/// `header`
pub const HEADER_MARGIN_BOTTOM: f64 = 14.0;
pub const HEADER_PADDING_BOTTOM: f64 = 10.0;
pub const HEADER_BORDER_WIDTH: f64 = 1.0;

/// `section`
pub const SECTION_MARGIN_BOTTOM: f64 = 11.0;

/// `proseParagraphGap`
pub const PROSE_PARAGRAPH_GAP: f64 = 4.0;

/// `inlineListSpacer`
pub const INLINE_LIST_SPACER: f64 = 7.0;

/// `achievementSpacer`
pub const ACHIEVEMENT_SPACER: f64 = 5.0;

/// `experienceItem`
pub const ENTRY_RULED_MARGIN_BOTTOM: f64 = 9.0;
pub const ENTRY_RULED_PADDING_LEFT: f64 = 8.0;
pub const ENTRY_RULE_WIDTH: f64 = 2.0;

/// `educationItem`
pub const ENTRY_PLAIN_MARGIN_BOTTOM: f64 = 8.0;

/// `labelValueRow` / `labelValueLabel` / `labelValueSpacer`
pub const LABEL_VALUE_ROW_MARGIN_BOTTOM: f64 = 4.0;
pub const LABEL_VALUE_LABEL_WIDTH: f64 = 130.0;
pub const LABEL_VALUE_SPACER: f64 = 6.0;
