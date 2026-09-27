use crate::prelude::{
    Color, Vector2, draw_rectangle_from_rect, draw_rectangle_lines_from_rect, mouse_position_vec2,
};

/// # Rect
/// Contains the position (x, y) and size (width, height) of a rectangle.
/// Can be used for detecting AABB collisions or drawing rectangles.
///
/// ## Examples
///
/// ```
/// let my_rect = Rect::new(0.0, 0.0, 50.0, 50.0);
/// ```
///
/// Drawing a Rect to the screen:
/// ```
/// my_rect.draw(RED);
/// // or
/// draw_rectangle_from_rect(my_rect, RED);
/// ```
///
/// Check collisions:
/// ```
/// if my_rect.contains_point(mouse_position_vec2()) {
///     println!("Mouse is inside the rect")
/// }
///
/// let aabb_intersection: Option<Rect> = my_rect.intersection(other_rect);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    /// Makes a new `Rect` object with the x, y position and Width x Height
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Build a Rect from two corner points (handles any order)
    pub fn from_corners(a: Vector2, b: Vector2) -> Self {
        let x = a.x.min(b.x);
        let y = a.y.min(b.y);
        Self::new(x, y, (a.x - b.x).abs(), (a.y - b.y).abs())
    }

    /// Build a Rect centered on `center` with the given size
    pub fn from_center(center: Vector2, w: f32, h: f32) -> Self {
        Self::new(center.x - w / 2.0, center.y - h / 2.0, w, h)
    }

    // --- Edges ---
    pub const fn left(&self) -> f32 {
        self.x
    }
    pub const fn right(&self) -> f32 {
        self.x + self.w
    }
    pub const fn top(&self) -> f32 {
        self.y
    }
    pub const fn bottom(&self) -> f32 {
        self.y + self.h
    }

    // --- Points ---
    pub const fn top_left(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }
    pub const fn top_right(&self) -> Vector2 {
        Vector2::new(self.x + self.w, self.y)
    }
    pub const fn bottom_left(&self) -> Vector2 {
        Vector2::new(self.x, self.y + self.h)
    }
    pub const fn bottom_right(&self) -> Vector2 {
        Vector2::new(self.x + self.w, self.y + self.h)
    }

    pub fn center(&self) -> Vector2 {
        Vector2::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    // --- Measurements ---
    pub const fn area(&self) -> f32 {
        self.w * self.h
    }
    pub const fn perimeter(&self) -> f32 {
        2.0 * (self.w + self.h)
    }
    pub const fn size(&self) -> Vector2 {
        Vector2::new(self.w, self.h)
    }
    pub const fn position(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    // --- Builders (non-mutating, return new Rect) ---
    /// Get a new Rect with the specified x
    pub const fn with_x(&self, x: f32) -> Self {
        Self::new(x, self.y, self.w, self.h)
    }
    /// Get a new Rect with the specified y
    pub const fn with_y(&self, y: f32) -> Self {
        Self::new(self.x, y, self.w, self.h)
    }
    /// Get a new Rect with the specified w
    pub const fn with_w(&self, w: f32) -> Self {
        Self::new(self.x, self.y, w, self.h)
    }
    /// Get a new Rect with the specified h
    pub const fn with_h(&self, h: f32) -> Self {
        Self::new(self.x, self.y, self.w, h)
    }

    /// Get a new Rect with the specified position
    pub fn with_position(&self, pos: Vector2) -> Self {
        Self::new(pos.x, pos.y, self.w, self.h)
    }

    /// Get a new Rect with the specified size
    pub fn with_size(&self, size: Vector2) -> Self {
        Self::new(self.x, self.y, size.x, size.y)
    }

    /// Get a moved Rect by an offset
    pub fn translated(&self, offset: Vector2) -> Self {
        Self::new(self.x + offset.x, self.y + offset.y, self.w, self.h)
    }

    /// Move the rect so its center is at `center`
    pub fn centered_at(&self, center: Vector2) -> Self {
        Self::from_center(center, self.w, self.h)
    }

    /// Grow/shrink the rect on all sides by `amount` (negative shrinks), keeping center fixed
    pub fn inflated(&self, amount: f32) -> Self {
        Self::new(
            self.x - amount,
            self.y - amount,
            self.w + amount * 2.0,
            self.h + amount * 2.0,
        )
    }

    /// Scale width/height around the top-left corner (position unchanged)
    pub const fn scaled(&self, factor: f32) -> Self {
        Self::new(self.x, self.y, self.w * factor, self.h * factor)
    }

    /// Scale width/height around the rect's center
    pub fn scaled_from_center(&self, factor: f32) -> Self {
        let center = self.center();
        Self::from_center(center, self.w * factor, self.h * factor)
    }

    /// Smallest rect that fully contains both `self` and `other`
    pub fn union(&self, other: &Rect) -> Self {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.w).max(other.x + other.w);
        let y2 = (self.y + self.h).max(other.y + other.h);
        Self::new(x1, y1, x2 - x1, y2 - y1)
    }

    /// Clamp a point to lie within this rect
    pub fn clamp_point(&self, point: Vector2) -> Vector2 {
        Vector2::new(
            point.x.clamp(self.left(), self.right()),
            point.y.clamp(self.top(), self.bottom()),
        )
    }

    /// Fix a rect with negative width/height into one with positive dimensions
    pub fn normalized(&self) -> Self {
        let (x, w) = if self.w < 0.0 {
            (self.x + self.w, -self.w)
        } else {
            (self.x, self.w)
        };
        let (y, h) = if self.h < 0.0 {
            (self.y + self.h, -self.h)
        } else {
            (self.y, self.h)
        };
        Self::new(x, y, w, h)
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.x, self.y, self.w, self.h]
    }

    /// Linearly interpolate between two rects
    pub fn lerp(self, other: Rect, t: f32) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.w + (other.w - self.w) * t,
            self.h + (other.h - self.h) * t,
        )
    }

    /// Test if point (x, y) lies inside/on the Rect
    pub const fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x <= self.x + self.w && y <= self.y + self.h
    }

    /// Test if point (`Vector2`) lies inside/on the Rect
    pub const fn contains_point_vec2(&self, point: Vector2) -> bool {
        self.contains_point(point.x, point.y)
    }

    /// Test if mouse lies inside/on the Rect
    pub fn contains_mouse(&self) -> bool {
        self.contains_point_vec2(mouse_position_vec2())
    }

    /// AABB overlap test
    pub const fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }

    /// Returns the overlapping region, or `None` if they don't intersect
    pub const fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.w).min(other.x + other.w);
        let y2 = (self.y + self.h).min(other.y + other.h);

        if x1 < x2 && y1 < y2 {
            Some(Rect::new(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }

    /// True if `self` fully contains `other`
    pub const fn contains_rect_fully(&self, other: &Rect) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.x + self.w >= other.x + other.w
            && self.y + self.h >= other.y + other.h
    }

    /// Draw the Rect to the screen using a color
    pub fn draw(&self, color: Color) {
        draw_rectangle_from_rect(*self, color);
    }

    /// Draw the Rect's outline to the screen using a color and thickness
    pub fn draw_lines(&self, thickness: f32, color: Color) {
        draw_rectangle_lines_from_rect(*self, thickness, color);
    }
}
