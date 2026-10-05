use crate::prelude::{update_texture, Rect, Vector2, WHITE, Image};
use anyhow::*;
use image::GenericImageView;
use wgpu::*;

/// # FilterType
///
/// Tells whether a texture should be drawn with pixelated (Nearest) or smooth (Linear) edges.
/// Linear is the default filter type.
///
/// ## Examples
///
/// ```
/// let pixelated_texture = load_texture("src/Cupcake.png").with_filter_type(FitlerType::Nearest);
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub enum FilterType {
    /// No antialiasing, sharp pixelated edges
    Nearest,

    /// Antialiasing, smooth interpolated edges
    #[default]
    Linear,
}

impl FilterType {
    pub fn to_wgpu_filter(&self) -> FilterMode {
        match self {
            FilterType::Nearest => FilterMode::Nearest,
            FilterType::Linear => FilterMode::Linear,
        }
    }

    pub fn to_wgpu_mipmap_filter(&self) -> MipmapFilterMode {
        match self {
            FilterType::Nearest => MipmapFilterMode::Nearest,
            FilterType::Linear => MipmapFilterMode::Linear,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Texture {
    pub(crate) id: usize,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Texture {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn update(&self, image: &Image) {
        update_texture(*self, image);
    }
}

pub(crate) struct TextureEntry {
    pub gpu: GpuTexture,
    pub bind_group: BindGroup,
}

pub struct GpuTexture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl GpuTexture {
    #[allow(unused)]
    pub fn empty(
        device: &Device,
        width: u32,
        height: u32,
        filter_type: FilterType,
    ) -> Result<Self> {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let mip_level_count = if matches!(filter_type, FilterType::Nearest) {
            1
        } else {
            width.max(height).ilog2() + 1
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("texture"),
            size,
            mip_level_count,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let filter_mode = filter_type.to_wgpu_filter();
        let mipmap_filter = filter_type.to_wgpu_mipmap_filter();

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: filter_mode,
            mipmap_filter,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    #[allow(unused)]
    pub fn from_bytes(
        device: &Device,
        queue: &Queue,
        bytes: &[u8],
        filter_type: FilterType,
        label: &str,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, filter_type, Some(label))
    }

    #[allow(unused)]
    pub fn from_image(
        device: &Device,
        queue: &Queue,
        img: &image::DynamicImage,
        filter_type: FilterType,
        label: Option<&str>,
    ) -> Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let mip_level_count = if matches!(filter_type, FilterType::Nearest) {
            1 // Disable mipmaps for pixel art
        } else {
            dimensions.0.max(dimensions.1).ilog2() + 1
        };
        let texture = device.create_texture(&TextureDescriptor {
            label,
            size,
            mip_level_count,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                aspect: TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );
        Self::generate_mipmaps(device, queue, &texture, mip_level_count);

        let filter_mode = filter_type.to_wgpu_filter();
        let mipmap_filter = filter_type.to_wgpu_mipmap_filter();

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: filter_mode,
            mipmap_filter,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub fn generate_mipmaps(
        device: &Device,
        queue: &Queue,
        texture: &wgpu::Texture,
        mip_count: u32,
    ) {
        let shader = device.create_shader_module(include_wgsl!("../shaders/blit.wgsl"));

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("mip blit"),
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &[Some(TextureFormat::Rgba8UnormSrgb.into())],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        let layout = pipeline.get_bind_group_layout(0);
        let mut encoder = device.create_command_encoder(&Default::default());

        for i in 1..mip_count {
            let src = texture.create_view(&TextureViewDescriptor {
                base_mip_level: i - 1,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let dst = texture.create_view(&TextureViewDescriptor {
                base_mip_level: i,
                mip_level_count: Some(1),
                ..Default::default()
            });

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&src),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&sampler),
                    },
                ],
            });

            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &dst,
                    resolve_target: None,
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::TRANSPARENT),
                        store: StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    /// # Depth Texture
    /// Stores the depth of each pixel for depth testing.
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn create_depth_texture(
        device: &Device,
        config: &SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let desc = TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 4,
            dimension: TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let texture = device.create_texture(&desc);
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: MipmapFilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

/// # DrawTextureParams
///
/// Parameters used in `draw_texture_ex()` which give more flexibility to drawing textures, such as
/// - How large the texture to be drawn is
/// - The source rect to draw
/// - Rotation
/// - Flip on X
/// - Flip on Y
/// - Pivot
pub struct DrawTextureParams {
    pub dest_size: Option<Vector2>,
    pub source: Option<Rect>, // x, y, w, h in pixels
    pub rotation: f32,        // radians
    pub flip_x: bool,
    pub flip_y: bool,
    pub pivot: Option<Vector2>, // world coords, default = center
    pub color: crate::prelude::Color,
}

impl Default for DrawTextureParams {
    fn default() -> Self {
        Self {
            dest_size: None,
            source: None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None,
            color: WHITE,
        }
    }
}
