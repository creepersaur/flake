#![allow(unused_doc_comments)]

use std::ptr::NonNull;
use crate::camera::camera2d::Camera2D;
use crate::camera::camera3d::Camera3D;
use crate::camera::{Camera, CameraUniform};
use crate::draw_state::draw_state::DrawState;
use crate::input::{keyboard::KeyboardState, mouse::MouseState};
use crate::model::instance::InstanceRaw;
use crate::model::texture;
use crate::model::vertex::Vertex;
use cgmath::Vector2;
use cgmath::prelude::*;
use device_query::DeviceState;
use std::sync::Arc;
use std::time::Instant;
use wgpu::{self, util::DeviceExt, *};
use wgpu_text::glyph_brush::ab_glyph::FontRef;
use wgpu_text::{BrushBuilder, TextBrush};
use winit::dpi::PhysicalSize;
use winit::event::MouseButton;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;
use crate::global::STATE;

pub struct State {
    // Window, Surface & Device
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    is_surface_configured: bool,

    // Pipeline
    render_pipeline: RenderPipeline,
    depth_texture: texture::Texture,
    msaa_view: TextureView,

    // Draw State
    pub clear_color: crate::shapes::color::Color,
    pub draw_state: DrawState,
    text_brush: TextBrush<FontRef<'static>>,

    // Camera
    camera: Camera2D,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,

    // Time
    last_frame: Instant,
    deltatime: f32,
    frames: usize,

    // Input
    device_state: DeviceState,
    pub keyboard_state: KeyboardState,
    pub mouse_state: MouseState,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let (instance, surface) = Self::get_instance_and_surface(window.clone())?;
        let adapter = Self::get_adapter(&surface, &instance).await?;
        let (device, queue) = Self::get_device_and_queue(&adapter).await?;
        let config = Self::get_surface_config(&surface, size, &adapter);

        /// ## Camera
        let camera = Self::get_camera2d(config.width, config.height);
        let (camera_uniform, camera_buffer) = Self::get_camera_uniform_buffer(&camera, &device);
        let (camera_bind_group, camera_bind_group_layout) =
            Self::get_camera_bind_group(&camera_buffer, &device);

        /// ## Depth Texture
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        /// # Render Pipeline
        let render_pipeline =
            Self::get_render_pipeline(&device, &config, &camera_bind_group_layout);

        /// # ANTI-ALIASING (MSAA)
        let msaa_view = Self::create_msaa_view(&device, &config);

        let draw_state = DrawState::new(&device);
        let text_brush = BrushBuilder::using_font_bytes(include_bytes!("Jetbrains.ttf"))?
            .with_multisample(MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            })
            .with_depth_stencil(Some(DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(CompareFunction::Always),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }))
            .build(&device, config.width, config.height, config.format);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,

            render_pipeline,
            depth_texture,
            msaa_view,

            clear_color: crate::shapes::color::Color::BLACK,
            draw_state,
            text_brush,

            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,

            last_frame: Instant::now(),
            deltatime: 0.0166,
            frames: 0,

            device_state: DeviceState::new(),
            keyboard_state: KeyboardState::default(),
            mouse_state: MouseState::default(),
        })
    }

    fn create_msaa_view(device: &Device, config: &SurfaceConfiguration) -> TextureView {
        device
            .create_texture(&TextureDescriptor {
                label: Some("msaa"),
                size: Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 4,
                dimension: TextureDimension::D2,
                format: config.format,
                usage: TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default())
    }

    #[allow(unused)]
    fn get_camera3d(width: u32, height: u32) -> Camera3D {
        Camera3D {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: width as f32 / height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    fn get_camera2d(width: u32, height: u32) -> Camera2D {
        Camera2D {
            position: Vector2::zero(),
            width: width as f32,
            height: height as f32,
            zoom: 1.0,
        }
    }

    fn get_camera_uniform_buffer(camera: &impl Camera, device: &Device) -> (CameraUniform, Buffer) {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(camera);

        let camera_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        (camera_uniform, camera_buffer)
    }

    fn get_camera_bind_group(
        camera_buffer: &Buffer,
        device: &Device,
    ) -> (BindGroup, BindGroupLayout) {
        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        (camera_bind_group, camera_bind_group_layout)
    }

    fn get_instance_and_surface(
        window: Arc<Window>,
    ) -> anyhow::Result<(Instance, Surface<'static>)> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });
        let surface = instance.create_surface(window)?;

        Ok((instance, surface))
    }

    async fn get_adapter(
        surface: &Surface<'_>,
        instance: &Instance,
    ) -> Result<Adapter, RequestAdapterError> {
        instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(surface),
                ..Default::default()
            })
            .await
    }

    async fn get_device_and_queue(
        adapter: &Adapter,
    ) -> Result<(Device, Queue), RequestDeviceError> {
        adapter
            .request_device(&DeviceDescriptor {
                trace: Trace::Off,
                ..Default::default()
            })
            .await
    }

    fn get_surface_config(
        surface: &Surface,
        size: PhysicalSize<u32>,
        adapter: &Adapter,
    ) -> SurfaceConfiguration {
        let caps = surface.get_capabilities(adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Immediate,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: Default::default(),
        }
    }

    /// Loads the diffuse texture and builds its bind group layout + bind group.
    #[allow(unused)]
    fn get_texture_bind_group(device: &Device, queue: &Queue) -> (BindGroupLayout, BindGroup) {
        let diffuse_texture = texture::Texture::from_bytes(
            device,
            queue,
            include_bytes!("Cupcake.png"),
            "Cupcake.png",
        )
        .expect("failed to load texture");

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: &layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        (layout, bind_group)
    }

    fn get_render_pipeline(
        device: &Device,
        config: &SurfaceConfiguration,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> RenderPipeline {
        let shader = device.create_shader_module(include_wgsl!("shaders/shader.wgsl"));

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[Some(camera_bind_group_layout)],
            immediate_size: 0,
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),

            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::desc()), Some(InstanceRaw::desc())],
                compilation_options: Default::default(),
            },

            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),

            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: FrontFace::Ccw,
                //cull_mode: Some(Face::Back),
                ..Default::default()
            },

            depth_stencil: Some(DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::LessEqual),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),

            multisample: MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.is_surface_configured = true;

        self.text_brush.resize_view(
            self.config.width as f32,
            self.config.height as f32,
            &self.queue,
        );
        self.msaa_view = Self::create_msaa_view(&self.device, &self.config);
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

        self.camera.width = width as f32;
        self.camera.height = height as f32;
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        self.keyboard_state.set_key_pressed(code, is_pressed);

        if let (KeyCode::Escape, true) = (code, is_pressed) {
            event_loop.exit();
        }
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, is_pressed: bool) {
        self.mouse_state.set_pressed_button(button, is_pressed);
    }

    pub fn handle_mouse_motion(&mut self, x: f32, y: f32) {
        self.mouse_state.set_position(x, y);
    }

    pub fn get_fps(&self) -> f32 {
        1.0 / self.deltatime
    }

    pub fn get_frame_time(&self) -> f32 {
        self.deltatime
    }

    pub fn get_frames(&self) -> usize {
        self.frames
    }

    pub fn update(&mut self, update_fn: &mut impl FnMut()) {
        self.mouse_state
            .update_position(&self.window, &self.device_state);

        let now = Instant::now();
        self.deltatime = (now - self.last_frame).as_secs_f32();
        self.frames += 1;

        let ptr = NonNull::from(&mut *self);
        STATE.set(Some(ptr));
        self.draw_state.clear();
        update_fn();
        STATE.set(None);

        // Update camera uniform
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        self.last_frame = now;
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t) => t,
            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Occluded
            | CurrentSurfaceTexture::Validation => {
                return Ok(()); // skip frame
            }
            CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            CurrentSurfaceTexture::Lost => anyhow::bail!("Lost device"),
        };

        let view = output.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            self.draw_state
                .upload(&self.device, &self.queue, &mut self.text_brush)?;

            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.msaa_view,
                    resolve_target: Some(&view),
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.clear_color.to_wgpu()),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            
            pass.set_pipeline(&self.render_pipeline);
            pass.set_bind_group(0, &self.camera_bind_group, &[]);

            self.draw_state.draw(&mut pass, &self.text_brush);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())
    }
}
