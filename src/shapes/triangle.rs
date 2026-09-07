use crate::model::vertex::Vertex;

pub const TRI_VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0] }, // bottom-left
    Vertex { position: [0.5, 0.0, 0.0], tex_coords: [1.0, 0.0] }, // top-middle
    Vertex { position: [1.0, 1.0, 0.0], tex_coords: [1.0, 1.0] }, // bottom-right
];

pub const TRI_INDICES: &[u16] = &[
    0, 1, 2,
];