use std::borrow::Borrow;
use crate::draw_state::shapes::circle::{CIRCLE_INDICES, CIRCLE_VERTICES};
use crate::draw_state::shapes::color::Color;
use crate::draw_state::shapes::polygon::{cross, point_in_tri};
use crate::draw_state::shapes::rectangle::{RECT_INDICES, RECT_VERTICES};
use crate::draw_state::shapes::triangle::{TRI_INDICES, TRI_VERTICES};
use crate::draw_state::shapes::{Shape, ShapeBuffer};
use crate::misc::font::Font;
use crate::misc::span::{RichtextSection, SpanRef};
use crate::model::instance::{Instance, InstanceRaw};
use cgmath::{InnerSpace, MetricSpace, One, Quaternion, Rotation3, Vector2, Vector3, Zero};
use wgpu::{Device, Queue, RenderPass};
use wgpu_text::TextBrush;
use wgpu_text::glyph_brush::ab_glyph::FontVec;
use wgpu_text::glyph_brush::{FontId, Section, Text};
use crate::prelude::Span;

#[derive(Clone, Debug)]
pub struct TextItem {
    pub text: String,
    pub font_id: FontId,
    pub pos: (f32, f32),
    pub z: f32,
    pub size: f32,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct QueueItem {
    shape: Shape,
    instance: Instance,
    z_offset: f32,
}

#[derive(Clone, Debug)]
pub struct DrawState {
    z_offset: f32,
    rect_buffer: ShapeBuffer,
    circle_buffer: ShapeBuffer,
    triangle_buffer: ShapeBuffer,

    queue: Vec<QueueItem>,
    scratch: Vec<InstanceRaw>,

    text_queue: Vec<TextItem>,
    text_len: usize,

    richtext_text: String,
    span_refs: Vec<SpanRef>,
    richtext_sections: Vec<RichtextSection>,
}

impl DrawState {
    pub fn new(device: &Device) -> Self {
        let rect_buffer = ShapeBuffer::new(
            device,
            Shape::Rectangle,
            "Rectangle Buffer",
            RECT_VERTICES,
            RECT_INDICES,
        );
        let circle_buffer = ShapeBuffer::new(
            device,
            Shape::Circle,
            "Circle Buffer",
            CIRCLE_VERTICES,
            CIRCLE_INDICES,
        );
        let triangle_buffer = ShapeBuffer::new(
            device,
            Shape::Triangle,
            "Triangle Buffer",
            TRI_VERTICES,
            TRI_INDICES,
        );

        Self {
            z_offset: 0.0,
            rect_buffer,
            circle_buffer,
            triangle_buffer,

            queue: Vec::with_capacity(1024),
            scratch: Vec::with_capacity(1024),

            text_queue: Vec::with_capacity(128),
            text_len: 0,

            richtext_sections: Vec::with_capacity(128),
            span_refs: Vec::with_capacity(128),
            richtext_text: String::new(),
        }
    }

    pub fn upload(
        &mut self,
        device: &Device,
        queue: &Queue,
        text_brush: &mut TextBrush<FontVec>,
    ) -> anyhow::Result<()> {
        for buf in [
            &mut self.rect_buffer,
            &mut self.circle_buffer,
            &mut self.triangle_buffer,
        ] {
            self.scratch.clear();
            self.scratch.extend(
                self.queue
                    .iter_mut()
                    .filter(|queue_item| queue_item.shape == buf.shape)
                    .map(|queue_item| {
                        queue_item
                            .instance
                            .update_with_z_offset(queue_item.z_offset)
                            .to_raw()
                    }),
            );
            buf.upload(device, queue, &self.scratch);
        }

        let plain = self.text_queue[..self.text_len].iter().map(|item| {
            Section::default()
                .add_text(
                    Text::new(&item.text)
                        .with_scale(item.size)
                        .with_color(item.color.to_array())
                        .with_z(item.z)
                        .with_font_id(item.font_id),
                )
                .with_screen_position((item.pos.0, item.pos.1))
        });

        let rich = self.richtext_sections.iter().map(|sec| {
            let mut section = Section::default()
                .with_screen_position((sec.position.x, sec.position.y));
            for r in &self.span_refs[sec.spans.clone()] {
                section = section.add_text(
                    Text::new(&self.richtext_text[r.range.clone()])
                        .with_scale(r.size)
                        .with_color(r.color.to_array())
                        .with_z(sec.z)
                        .with_font_id(r.font_id),
                );
            }
            section
        });

        text_brush.queue(device, queue, plain.chain(rich))?;

        Ok(())
    }

    pub fn draw<'a>(&'a self, pass: &mut RenderPass<'a>, text_brush: &TextBrush<FontVec>) {
        self.rect_buffer.draw(pass);
        self.circle_buffer.draw(pass);
        self.triangle_buffer.draw(pass);

        text_brush.draw(pass);
    }

    pub fn increment_z(&mut self) {
        const EPSILON: f32 = 0.000016;
        self.z_offset += EPSILON;
    }

    pub fn push_shape(&mut self, shape: Shape, instance: Instance) {
        self.increment_z();
        self.queue.push(QueueItem {
            shape,
            instance,
            z_offset: self.z_offset,
        })
    }
}

impl DrawState {
    pub fn clear(&mut self) {
        self.queue.clear();
        self.text_len = 0;

        self.richtext_text.clear();
        self.span_refs.clear();
        self.richtext_sections.clear();

        self.z_offset = 0.0;
    }

    pub fn draw_rectangle(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.push_shape(
            Shape::Rectangle,
            Instance {
                position: Vector3::new(x, y, 0.0),
                size: Vector2::new(w, h),
                rotation: Quaternion::one(),
                color,
                shape: Shape::Rectangle,
                tri_points: [Vector2::zero(), Vector2::zero(), Vector2::zero()],
            },
        );
    }

    pub fn draw_rectangle_lines(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        thickness: f32,
        color: Color,
    ) {
        self.draw_line(
            x - thickness / 2.0,
            y,
            x + w + thickness / 2.0,
            y,
            thickness,
            color,
        );
        self.draw_line(x + w, y, x + w, y + h + thickness / 2.0, thickness, color);
        self.draw_line(x, y + h, x + w + thickness / 2.0, y + h, thickness, color);
        self.draw_line(x, y, x, y + h + thickness / 2.0, thickness, color);
    }

    pub fn draw_rectangle_rotated(
        &mut self,
        mut x: f32,
        mut y: f32,
        w: f32,
        h: f32,
        rotation: f32,
        color: Color,
    ) {
        x -= w / 2.0;
        y -= h / 2.0;
        let (s, c) = rotation.sin_cos();
        let cx = x + w / 2.0;
        let cy = y + h / 2.0;

        // offset from center to top-left, rotated
        let (hx, hy) = (-w / 2.0, -h / 2.0);
        let offset_x = hx * c - hy * s;
        let offset_y = hx * s + hy * c;

        self.push_shape(
            Shape::Rectangle,
            Instance {
                position: Vector3::new(cx + offset_x, cy + offset_y, 0.0),
                size: Vector2::new(w, h),
                rotation: Quaternion::from_angle_z(cgmath::Rad(rotation)),
                color,
                shape: Shape::Rectangle,
                tri_points: [Vector2::zero(); 3],
            },
        );
    }
    pub fn draw_rectangle_lines_rotated(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rotation: f32,
        thickness: f32,
        color: Color,
    ) {
        let (s, c) = rotation.sin_cos();
        let (cx, cy) = (x + w * 0.5, y + h * 0.5);
        let rot = |dx: f32, dy: f32| {
            let (dx, dy) = (dx - w * 0.5, dy - h * 0.5);
            Vector2::new(cx + dx * c - dy * s, cy + dx * s + dy * c)
        };
        let points = [rot(0.0, 0.0), rot(w, 0.0), rot(w, h), rot(0.0, h)];
        self.draw_poly_line_miter(&points, thickness, color, true);
    }

    pub fn draw_circle(&mut self, x: f32, y: f32, r: f32, color: Color) {
        self.push_shape(
            Shape::Circle,
            Instance {
                position: Vector3::new(x, y, 0.0),
                size: Vector2::new(r, r),
                rotation: Quaternion::one(),
                color,
                shape: Shape::Circle,
                tri_points: [Vector2::zero(), Vector2::zero(), Vector2::zero()],
            },
        );
    }

    pub fn draw_circle_lines(&mut self, x: f32, y: f32, r: f32, thickness: f32, color: Color) {
        let segments = ((r.sqrt() * 8.0) as usize).clamp(16, 128);
        let step = std::f32::consts::TAU / segments as f32;

        let points: Vec<Vector2<f32>> = (0..segments)
            .map(|i| {
                let a = i as f32 * step;
                Vector2::new(x + r / 2.0 * a.cos(), y + r / 2.0 * a.sin())
            })
            .collect();

        self.draw_poly_line_miter(&points, thickness, color, true);
    }

    pub fn draw_triangle(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        color: Color,
    ) {
        self.push_shape(
            Shape::Triangle,
            Instance {
                position: Vector3::new(0.0, 0.0, 0.0),
                size: Vector2::new(1.0, 1.0),
                rotation: Quaternion::one(),
                color,
                shape: Shape::Triangle,
                tri_points: [
                    Vector2::new(x1, y1),
                    Vector2::new(x2, y2),
                    Vector2::new(x3, y3),
                ],
            },
        );
    }

    pub fn draw_triangle_lines(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        thickness: f32,
        color: Color,
    ) {
        self.draw_poly_line_miter(
            &[
                Vector2::new(x1, y1),
                Vector2::new(x2, y2),
                Vector2::new(x3, y3),
            ],
            thickness,
            color,
            true,
        );
    }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        let a = Vector2::new(x1, y1);
        let b = Vector2::new(x2, y2);
        let delta = b - a;
        let angle_radians = delta.y.atan2(delta.x);

        self.draw_rectangle_rotated(
            (x1 + x2) / 2.0,
            (y1 + y2) / 2.0,
            a.distance(b),
            thickness,
            angle_radians,
            color,
        );
    }

    pub fn draw_poly_line(
        &mut self,
        points: &[Vector2<f32>],
        thickness: f32,
        color: Color,
        closed: bool,
        rounded: bool,
    ) {
        for win in points.windows(2) {
            let (a, b) = (win[0], win[1]);

            if rounded {
                self.draw_circle(a.x, a.y, thickness, color);
            }
            self.draw_line(a.x, a.y, b.x, b.y, thickness, color);
        }

        if closed
            && let Some(a) = points.first()
            && let Some(b) = points.last()
        {
            self.draw_line(a.x, a.y, b.x, b.y, thickness, color);
        }

        if rounded && let Some(a) = points.last() {
            self.draw_circle(a.x, a.y, thickness, color);
        }
    }

    pub fn draw_polygon(&mut self, points: &[Vector2<f32>], color: Color) {
        if points.len() < 3 {
            return;
        }
        let a = points[0];
        for win in points[1..].windows(2) {
            let (b, c) = (win[0], win[1]);
            self.draw_triangle(a.x, a.y, b.x, b.y, c.x, c.y, color);
        }
    }

    #[allow(unused)]
    pub fn draw_polygon_concave(&mut self, points: &[Vector2<f32>], color: Color) {
        if points.len() < 3 {
            return;
        }

        // Signed area: >0 means counter-clockwise
        let area: f32 = (0..points.len())
            .map(|i| {
                let (a, b) = (points[i], points[(i + 1) % points.len()]);
                a.x * b.y - b.x * a.y
            })
            .sum();
        let sign = if area >= 0.0 { 1.0 } else { -1.0 };

        let mut idx: Vec<usize> = (0..points.len()).collect();

        while idx.len() > 3 {
            let n = idx.len();
            let mut clipped = false;

            for i in 0..n {
                let (ia, ib, ic) = (idx[(i + n - 1) % n], idx[i], idx[(i + 1) % n]);
                let (a, b, c) = (points[ia], points[ib], points[ic]);

                // Reflex vertex: not an ear
                if cross(a, b, c) * sign <= 0.0 {
                    continue;
                }
                // Another vertex inside the triangle: not an ear
                if idx
                    .iter()
                    .any(|&j| j != ia && j != ib && j != ic && point_in_tri(points[j], a, b, c))
                {
                    continue;
                }

                self.draw_triangle(a.x, a.y, b.x, b.y, c.x, c.y, color);
                idx.remove(i);
                clipped = true;
                break;
            }

            if !clipped {
                break; // degenerate or self-intersecting polygon
            }
        }

        if idx.len() == 3 {
            let (a, b, c) = (points[idx[0]], points[idx[1]], points[idx[2]]);
            self.draw_triangle(a.x, a.y, b.x, b.y, c.x, c.y, color);
        }
    }

    #[allow(unused)]
    pub fn draw_poly_line_miter(
        &mut self,
        points: &[Vector2<f32>],
        thickness: f32,
        color: Color,
        closed: bool,
    ) {
        // Drop consecutive duplicates (they break direction math)
        let mut pts: Vec<Vector2<f32>> = Vec::with_capacity(points.len());
        for &p in points {
            if pts.last().map_or(true, |&q| q.distance2(p) > 1e-12) {
                pts.push(p);
            }
        }
        if closed && pts.len() > 1 && pts[0].distance2(*pts.last().unwrap()) < 1e-12 {
            pts.pop();
        }

        let n = pts.len();
        if n < 2 {
            return;
        }

        let half = thickness / 2.0;
        const MITER_LIMIT: f32 = 4.0; // max miter length / half-thickness

        // Left-hand normal of a segment direction
        let normal = |d: Vector2<f32>| Vector2::new(-d.y, d.x);
        let dir = |a: Vector2<f32>, b: Vector2<f32>| (b - a).normalize();

        // For each point, compute the left and right offset vertices
        let mut left: Vec<Vector2<f32>> = Vec::with_capacity(n);
        let mut right: Vec<Vector2<f32>> = Vec::with_capacity(n);

        for i in 0..n {
            let has_prev = closed || i > 0;
            let has_next = closed || i < n - 1;

            let d_prev = if has_prev {
                Some(dir(pts[(i + n - 1) % n], pts[i]))
            } else {
                None
            };
            let d_next = if has_next {
                Some(dir(pts[i], pts[(i + 1) % n]))
            } else {
                None
            };

            let offset = match (d_prev, d_next) {
                (Some(p), Some(q)) => {
                    let n1 = normal(p);
                    let n2 = normal(q);
                    let m = n1 + n2;
                    let m_len2 = m.magnitude2();
                    // Nearly 180° reversal: fall back to bevel-ish normal
                    if m_len2 < 1e-6 {
                        n2 * half
                    } else {
                        let m = m / m_len2.sqrt();
                        let cos_half = m.dot(n1); // cos of half the turn angle
                        let miter_len = half / cos_half.max(1e-4);
                        if miter_len / half > MITER_LIMIT {
                            // Too sharp: clamp the miter length
                            m * (half * MITER_LIMIT)
                        } else {
                            m * miter_len
                        }
                    }
                }
                (Some(d), None) | (None, Some(d)) => normal(d) * half,
                (None, None) => return,
            };

            left.push(pts[i] + offset);
            right.push(pts[i] - offset);
        }

        let segs = if closed { n } else { n - 1 };
        for i in 0..segs {
            let j = (i + 1) % n;
            let (l0, r0, l1, r1) = (left[i], right[i], left[j], right[j]);
            self.draw_triangle(l0.x, l0.y, r0.x, r0.y, l1.x, l1.y, color);
            self.draw_triangle(r0.x, r0.y, r1.x, r1.y, l1.x, l1.y, color);
        }
    }

    pub fn draw_text(
        &mut self,
        fonts: &[Font],
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        font: Option<&Font>,
        color: Color,
    ) {
        self.increment_z();

        if self.text_len < self.text_queue.len() {
            let item = &mut self.text_queue[self.text_len];
            item.text.clear();
            item.text.push_str(text);
            item.pos.0 = x;
            item.pos.1 = y;
            item.z = self.z_offset;
            item.color = color;
            item.size = size;
        } else {
            self.text_queue.push(TextItem {
                text: text.to_owned(),
                font_id: font.unwrap_or(&fonts[0]).id,
                pos: (x, y),
                z: self.z_offset,
                size,
                color,
            });
        }

        self.text_len += 1;
    }

    pub fn draw_richtext<'a, I>(&mut self, position: Vector2<f32>, spans: I)
    where
        I: IntoIterator,
        I::Item: Borrow<Span<'a>>,
    {
        self.increment_z();

        let first = self.span_refs.len();
        for s in spans {
            let s: &Span = s.borrow();
            let start = self.richtext_text.len();
            self.richtext_text.push_str(s.text);
            self.span_refs.push(SpanRef {
                range: start..self.richtext_text.len(),
                color: s.color,
                size: s.size,
                font_id: s.font_id,
            });
        }
        self.richtext_sections.push(RichtextSection {
            position,
            z: self.z_offset,
            spans: first..self.span_refs.len(),
        });
    }

    pub fn draw_arrow(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        thickness: f32,
        head_size: f32,
        color: Color,
    ) {
        self.draw_line(x1, y1, x2, y2, thickness, color);

        let a = Vector2::new(x1, y1);
        let b = Vector2::new(x2, y2);
        let diff = (b - a).normalize() * head_size;
        let diff_perp = Vector2::new(-diff.y, diff.x) / 1.5;

        self.draw_triangle(
            x2 - diff_perp.x,
            y2 - diff_perp.y,
            x2 + diff.x,
            y2 + diff.y,
            x2 + diff_perp.x,
            y2 + diff_perp.y,
            color,
        );
    }
}
