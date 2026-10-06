use std::sync::LazyLock;
use crate::model::vertex::Vertex;

const CIRCLE_VERTEX_COUNT: usize = 10;

fn circle_mesh(n: usize) -> (Vec<Vertex>, Vec<u16>) {
    let r = 0.5 / (std::f32::consts::PI / n as f32).cos();
    let verts = (0..n)
        .map(|i| {
            let a = (i as f32 + 0.5) * std::f32::consts::TAU / n as f32;
            let (s, c) = a.sin_cos();
            Vertex {
                position: [c * r, s * r, 0.0],
                tex_coords: [c * r + 0.5, s * r + 0.5],
            }
        })
        .collect();
    let idx = (1..n as u16 - 1).flat_map(|i| [0, i, i + 1]).collect();
    (verts, idx)
}

static CIRCLE_MESH: LazyLock<(Vec<Vertex>, Vec<u16>)> =
    LazyLock::new(|| circle_mesh(CIRCLE_VERTEX_COUNT));

pub fn circle_vertices() -> &'static [Vertex] { &CIRCLE_MESH.0 }
pub fn circle_indices() -> &'static [u16] { &CIRCLE_MESH.1 }