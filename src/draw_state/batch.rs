use crate::draw_state::shapes::Shape;
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct Batch {
    pub shape: Shape,
    pub texture: usize,
    pub range: Range<u32>,
}
