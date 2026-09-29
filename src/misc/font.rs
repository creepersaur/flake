use std::sync::Arc;
use wgpu_text::glyph_brush::FontId;

#[derive(Clone, PartialEq)]
pub struct Font {
    pub id: FontId,
    pub data: Arc<[u8]>,
}

impl Font {
    pub fn from_bytes_vec(id: usize, bytes: Vec<u8>) -> Self {
        Self {
            id: FontId(id),
            data: bytes.into(),
        }
    }

    pub fn from_bytes(id: usize, bytes: &[u8]) -> Self {
        Self {
            id: FontId(id),
            data: bytes.into(),
        }
    }

    pub fn from_path(id: usize, path: &str) -> Self {
        Self::from_bytes_vec(id, std::fs::read(path).expect("Could not read/find font file"))
    }
}
