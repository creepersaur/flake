use crate::model::vertex::Vertex;

pub const CIRCLE_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 0.0] }, // top-left
    Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 0.0] }, // top-right
    Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 1.0] }, // bottom-right
    Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 1.0] }, // bottom-left
];

pub const CIRCLE_INDICES: &[u16] = &[
    0, 1, 2,
    0, 2, 3,
];