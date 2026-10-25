use crate::model::texture::{GpuTexture, Texture};
use wgpu::TextureView;

#[derive(Clone, Debug)]
pub struct RenderTarget {
    pub texture: Texture,
    pub depth: GpuTexture,
    pub msaa_view: TextureView,
}
