use crate::prelude::font::Font;
use crate::prelude::{Color, Vector2};
use wgpu_text::glyph_brush::FontId;

#[derive(Clone, Copy, Debug)]
pub struct Span<'a> {
    pub text: &'a str,
    pub color: Color,
    pub size: f32,
    pub font_id: FontId,
}

impl<'a> Span<'a> {
    pub const fn new(text: &'a str, color: Color) -> Self {
        Self {
            text,
            color,
            size: 16.0,
            font_id: FontId(0),
        }
    }

    pub const fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub const fn with_font(mut self, font: Option<&Font>) -> Self {
        self.font_id = if let Some(font) = font {
            font.id
        } else {
            FontId(0)
        };
        self
    }
}

// internal: no lifetimes, no strings
#[derive(Clone, Debug)]
pub(crate) struct SpanRef {
    pub range: std::ops::Range<usize>, // into the frame text arena
    pub color: Color,
    pub size: f32,
    pub font_id: FontId,
}

#[derive(Clone, Debug)]
pub(crate) struct RichtextSection {
    pub position: Vector2,
    pub z: f32,
    pub spans: std::ops::Range<usize>, // into span_refs
}
