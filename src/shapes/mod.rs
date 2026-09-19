pub mod rectangle;
pub mod color;
pub mod circle;
pub mod triangle;
pub mod polygon;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Rectangle = 0,
    Circle = 1,
    Triangle = 2,
}

impl PartialEq<Shape> for &Shape {
    fn eq(&self, other: &Shape) -> bool {
        other == *self
    }
}