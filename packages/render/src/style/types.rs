use oxidize_pdf::{Font, Margins, Page, TextAlign, TextFlowContext};

#[derive(Clone)]
pub struct FontConfig {
    pub font: Font,
    pub size: f64,
}

pub enum StyleFont {
    Display,
    Title,
    Body,
}

impl StyleFont {
    pub fn get_font_config(&self, style: &Style) -> FontConfig {
        match self {
            StyleFont::Display => style.display.clone(),
            StyleFont::Title => style.title.clone(),
            StyleFont::Body => style.body.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Spacings {
    pub element: f64,
    pub block: f64,
}

pub enum Spacing {
    Element,
    Block,
}
impl Spacing {
    pub fn get_spacing(&self, style: &Style) -> f64 {
        match self {
            Spacing::Element => style.spacings.element,
            Spacing::Block => style.spacings.block,
        }
    }
}

#[derive(Clone)]
pub struct Style {
    pub display: FontConfig,
    pub title: FontConfig,
    pub body: FontConfig,
    pub margin: f64,
    pub indentation: f64,
    pub spacings: Spacings,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            display: FontConfig {
                font: Font::HelveticaBold,
                size: 30.0,
            },
            title: FontConfig {
                font: Font::HelveticaBold,
                size: 14.0,
            },
            body: FontConfig {
                font: Font::Helvetica,
                size: 10.0,
            },
            margin: 32.0,
            indentation: 8.0,
            spacings: Spacings {
                element: 16.0,
                block: 8.0,
            },
        }
    }
}

impl Style {
    pub fn get_initial_cursor_position(&self, page: &Page) -> (f64, f64) {
        (0.0, page.height() - self.margin)
    }

    pub fn setup_page(&self) -> Page {
        let mut page = Page::a4();
        page.set_margins(self.margin, self.margin, self.margin, self.margin);
        page
    }

    pub fn style_flow(&self, font: StyleFont, flow: &mut TextFlowContext, alignment: TextAlign) {
        let font_config = font.get_font_config(self);
        flow.set_font(font_config.font.clone(), font_config.size)
            .set_alignment(alignment);
    }

    pub fn write_to_flow(
        &self,
        flow: &mut TextFlowContext,
        font: StyleFont,
        alignment: TextAlign,
        text: &str,
    ) -> Result<(), anyhow::Error> {
        self.style_flow(font, flow, alignment);
        flow.write_wrapped(text)?;
        Ok(())
    }

    pub fn create_flow(&self, page: &Page, indent_level: Option<f64>) -> TextFlowContext {
        let indentation = indent_level.unwrap_or(0.0) * self.indentation;

        let mut flow = TextFlowContext::new(
            page.effective_width(),
            page.effective_height(),
            Margins {
                left: self.margin + indentation,
                right: self.margin,
                top: self.margin,
                bottom: self.margin,
            },
        );

        let (x, y) = self.get_initial_cursor_position(page);
        flow.at(x + indentation, y);
        flow
    }
}
