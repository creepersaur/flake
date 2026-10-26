#![allow(unused_doc_comments)]
use crate::camera::camera2d::Camera2D;
use crate::camera::camera3d::Camera3D;
use crate::camera::render_target::RenderTarget;
use crate::camera::{Camera, CameraUniform};
use crate::draw_state::draw_state::DrawState;
use crate::global::STATE;
use crate::input::{keyboard::KeyboardState, mouse::MouseState};
use crate::misc::font::Font as FlakeFont;
use crate::misc::timer::Timer;
use crate::model::instance::InstanceRaw;
use crate::model::texture::{GpuTexture, TextureEntry};
use crate::model::vertex::Vertex;
use crate::prelude::{BLACK, FilterType, Image};
use cgmath::Vector2;
use cgmath::prelude::*;
use device_query::DeviceState;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::Arc;
use std::task::{Context, Waker};
use std::time::Instant;
use wgpu::{self, util::DeviceExt, *};
use wgpu_text::glyph_brush::ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use wgpu_text::{BrushBuilder, TextBrush};
use winit::dpi::PhysicalSize;
use winit::event::MouseButton;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowLevel};

struct TextSlot {
    brush: TextBrush<FontVec>,
    size: (u32, u32),
}

pub struct State {
    // Window, Surface & Device
    pub running: bool,
    pub window: Arc<Window>,
    surface: Surface<'static>,
    pub device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    is_surface_configured: bool,

    // Runtime
    pub(crate) task: Option<Pin<Box<dyn Future<Output = ()>>>>,

    // Pipeline
    render_pipeline: RenderPipeline,
    depth_texture: GpuTexture,
    msaa_view: TextureView,
    textures: Vec<TextureEntry>,
    texture_layout: BindGroupLayout,

    // Draw State
    pub clear_color: Option<crate::draw_state::shapes::color::Color>,
    pub draw_state: DrawState,

    // Text
    pub fonts: Vec<FlakeFont>,
    metric_fonts: Vec<FontVec>,
    text_pool: Vec<Option<TextSlot>>,

    // Camera
    camera: Camera2D,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    cam_pool: Vec<(Buffer, BindGroup)>,
    cam_used: usize,

    // Time
    last_frame: Instant,
    fps_cap: bool,
    target_fps: f32,
    raw_deltatime: f32,
    deltatime: f32,
    frames: usize,
    pub timers: Vec<Timer>,

    // Input
    device_state: DeviceState,
    pub keyboard_state: KeyboardState,
    pub mouse_state: MouseState,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let target_fps = window
            .current_monitor()
            .and_then(|m| m.refresh_rate_millihertz())
            .map(|mhz| mhz as f32 / 1000.0)
            .unwrap_or(60.0);

        let (instance, surface) = Self::get_instance_and_surface(window.clone())?;
        let adapter = Self::get_adapter(&surface, &instance).await?;
        let (device, queue) = Self::get_device_and_queue(&adapter).await?;
        let config = Self::get_surface_config(&surface, size, &adapter);

        /// ## Camera
        let camera = Self::get_camera2d(config.width, config.height);
        let (camera_uniform, camera_buffer) = Self::get_camera_uniform_buffer(&camera, &device);
        let (camera_bind_group, camera_bind_group_layout) =
            Self::get_camera_bind_group(&camera_buffer, &device);

        /// ## Textures (and Depth)
        let depth_texture =
            GpuTexture::create_depth_texture(&device, config.width, config.height, "depth_texture");

        let texture_layout = Self::get_texture_layout(&device);

        /// # Render Pipeline
        let render_pipeline =
            Self::get_render_pipeline(&device, &config, &camera_bind_group_layout, &texture_layout);

        /// # ANTI-ALIASING (MSAA)
        let msaa_view = Self::create_msaa_view(&device, &config, config.width, config.height);

        let draw_state = DrawState::new(&device);

        Ok(Self {
            running: true,
            window,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,

            task: None,

            render_pipeline,
            depth_texture,
            msaa_view,
            textures: Vec::with_capacity(128),
            texture_layout,

            clear_color: Some(BLACK),
            draw_state,

            fonts: Vec::with_capacity(10),
            metric_fonts: Vec::new(),
            text_pool: Vec::new(),

            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            cam_pool: Vec::new(),
            cam_used: 0,

            target_fps,
            fps_cap: true,
            last_frame: Instant::now(),
            deltatime: 0.0166,
            raw_deltatime: 0.0166,
            frames: 0,
            timers: Vec::with_capacity(10),

            device_state: DeviceState::new(),
            keyboard_state: KeyboardState::default(),
            mouse_state: MouseState::default(),
        })
    }

    pub fn initialize_state(&mut self) {
        let white = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            1,
            1,
            image::Rgba([255, 255, 255, 255]),
        ));
        self.add_texture(white, FilterType::Nearest);
    }

    fn create_msaa_view(
        device: &Device,
        config: &SurfaceConfiguration,
        width: u32,
        height: u32,
    ) -> TextureView {
        device
            .create_texture(&TextureDescriptor {
                label: Some("msaa"),
                size: Extent3d {
                    width,
                    height,
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
            render_target: None,
        }
    }

    fn get_camera2d(width: u32, height: u32) -> Camera2D {
        Camera2D {
            position: Vector2::zero(),
            width: width as f32,
            height: height as f32,
            zoom: 1.0,
            render_target: None,
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
            .unwrap_or(caps.formats[0])
            .remove_srgb_suffix();

        let alpha_mode = if caps
            .alpha_modes
            .contains(&CompositeAlphaMode::PostMultiplied)
        {
            CompositeAlphaMode::PostMultiplied
        } else if caps
            .alpha_modes
            .contains(&CompositeAlphaMode::PreMultiplied)
        {
            CompositeAlphaMode::PreMultiplied
        } else {
            caps.alpha_modes[0] // fallback, transparency won't work
        };

        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            alpha_mode,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Immediate,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: Default::default(),
        }
    }

    pub fn add_texture(&mut self, img: image::DynamicImage, filter_type: FilterType) -> usize {
        let gpu =
            GpuTexture::from_image(&self.device, &self.queue, &img, filter_type, None).unwrap();
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("texture bind group"),
            layout: &self.texture_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&gpu.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&gpu.sampler),
                },
            ],
        });
        self.textures.push(TextureEntry { gpu, bind_group });
        self.textures.len() - 1
    }

    pub fn add_gpu_texture(&mut self, gpu: GpuTexture) -> usize {
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("texture bind group"),
            layout: &self.texture_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&gpu.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&gpu.sampler),
                },
            ],
        });
        self.textures.push(TextureEntry { gpu, bind_group });
        self.textures.len() - 1
    }

    /// Loads the diffuse texture and builds its bind group layout + bind group.
    fn get_texture_layout(device: &Device) -> BindGroupLayout {
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("texture layout"),
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

        layout
    }

    /// Uploads the current image pixel data to an existing GPU texture.
    pub fn update_texture(&self, texture_id: usize, image: &Image) {
        if let Some(entry) = self.textures.get(texture_id) {
            let texture = &entry.gpu.texture;

            let bytes_per_row = image.width * 4;

            self.queue.write_texture(
                TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                &image.data,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(image.height),
                },
                Extent3d {
                    width: image.width,
                    height: image.height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    fn build_text_brush(
        device: &Device,
        fonts: &[FlakeFont],
        (w, h): (u32, u32),
        format: TextureFormat,
    ) -> TextBrush<FontVec> {
        let fonts: Vec<FontVec> = fonts
            .iter()
            .map(|s| FontVec::try_from_vec(s.data.to_vec()).expect("Invalid font data"))
            .collect();

        BrushBuilder::using_fonts(fonts)
            .with_multisample(MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            })
            .with_depth_stencil(Some(DepthStencilState {
                format: GpuTexture::DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(CompareFunction::Always),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }))
            .build(device, w, h, format)
    }

    fn rebuild_text_brush(&mut self) {
        self.text_pool.clear();
    }

    fn get_render_pipeline(
        device: &Device,
        config: &SurfaceConfiguration,
        camera_bind_group_layout: &BindGroupLayout,
        texture_layout: &BindGroupLayout,
    ) -> RenderPipeline {
        let shader = device.create_shader_module(include_wgsl!("shaders/shader.wgsl"));

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[Some(camera_bind_group_layout), Some(texture_layout)],
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
                format: GpuTexture::DEPTH_FORMAT,
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

        self.msaa_view = Self::create_msaa_view(
            &self.device,
            &self.config,
            self.config.width,
            self.config.height,
        );
        self.depth_texture = GpuTexture::create_depth_texture(
            &self.device,
            self.config.width,
            self.config.height,
            "depth_texture",
        );

        self.camera.width = width as f32;
        self.camera.height = height as f32;
    }

    pub fn handle_key(&mut self, _event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        self.keyboard_state.set_key_pressed(code, is_pressed);
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, is_pressed: bool) {
        self.mouse_state.set_pressed_button(button, is_pressed);
    }

    pub fn handle_mouse_motion(&mut self, x: f32, y: f32) {
        self.mouse_state.set_position(x, y);
    }

    pub fn handle_mouse_wheel(&mut self, x: f32, y: f32) {
        self.mouse_state.accumulate_mouse_wheel(x, y);
    }

    pub fn update(&mut self) {
        self.mouse_state
            .update_position(&self.window, &self.device_state);

        let now = Instant::now();
        self.frames += 1;

        if self.fps_cap {
            self.raw_deltatime = (now - self.last_frame).as_secs_f32();

            // Sleep to cap to target_fps if we're running faster than that
            let target_frame_time = 1.0 / self.target_fps;
            if self.raw_deltatime < target_frame_time {
                let sleep_time = target_frame_time - self.raw_deltatime;
                std::thread::sleep(std::time::Duration::from_secs_f32(sleep_time));
            }

            let paced_now = Instant::now();
            self.deltatime = (paced_now - self.last_frame).as_secs_f32();
            self.last_frame = paced_now;
        } else {
            self.deltatime = (now - self.last_frame).as_secs_f32();
            self.last_frame = now;
        }

        self.timers.iter_mut().for_each(|x| x.tick(self.deltatime));

        let ptr = NonNull::from(&mut *self);
        STATE.set(Some(ptr));
        {
            self.draw_state.clear();

            if let Some(mut task) = self.task.take() {
                let mut cx = Context::from_waker(Waker::noop());
                if task.as_mut().poll(&mut cx).is_pending() {
                    self.task = Some(task);
                } else {
                    self.running = false;
                }
            }
        }
        STATE.set(None);

        self.timers.retain_mut(|x| {
            if x.completed() {
                if x.0.borrow().repeating {
                    x.restart();
                    return true;
                } else {
                    x.0.borrow_mut().is_active = false;
                }
            }
            x.ongoing()
        });

        // Update camera uniform
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        self.keyboard_state.clear_just_pressed();
        self.mouse_state.clear_just_pressed();
        self.mouse_state.clear_mouse_wheel();
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
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
            self.draw_state.upload_shapes(&self.device, &self.queue);

            let segments = self.draw_state.segments();

            for i in 0..self.draw_state.segments().len() {
                if !self.draw_state.segment_has_text(i) {
                    continue;
                }
                let size = match &self.draw_state.segments()[i].target {
                    Some(rt) => (rt.texture.width, rt.texture.height),
                    None => (self.config.width, self.config.height),
                };

                if self.text_pool.len() <= i {
                    self.text_pool.resize_with(i + 1, || None);
                }
                let slot = self.text_pool[i].get_or_insert_with(|| TextSlot {
                    brush: Self::build_text_brush(
                        &self.device,
                        &self.fonts,
                        size,
                        self.config.format,
                    ),
                    size,
                });
                if slot.size != size {
                    slot.brush
                        .resize_view(size.0 as f32, size.1 as f32, &self.queue);
                    slot.size = size;
                }
                self.draw_state
                    .upload_text(&self.device, &self.queue, &mut slot.brush, i)?;
            }

            let mut screen_cleared = false;
            let mut rt_cleared: Vec<usize> = Vec::new();

            for (i, seg) in segments.iter().enumerate() {
                let end = segments
                    .get(i + 1)
                    .map_or(self.draw_state.batch_count(), |n| n.first_batch);

                let cam_bg = match seg.camera {
                    Some(slot) => &self.cam_pool[slot].1,
                    None => &self.camera_bind_group,
                };

                let (msaa, resolve, depth, load) = match &seg.target {
                    Some(rt) => {
                        let load = if rt_cleared.contains(&rt.texture.id) {
                            LoadOp::Load
                        } else {
                            rt_cleared.push(rt.texture.id);
                            LoadOp::Clear(Color::TRANSPARENT)
                        };
                        (
                            &rt.msaa_view,
                            &self.textures[rt.texture.id].gpu.view,
                            &rt.depth.view,
                            load,
                        )
                    }
                    None => {
                        let load = match (screen_cleared, self.clear_color) {
                            (false, Some(c)) => LoadOp::Clear(c.to_wgpu()),
                            _ => LoadOp::Load,
                        };
                        screen_cleared = true;
                        (&self.msaa_view, &view, &self.depth_texture.view, load)
                    }
                };

                let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: msaa,
                        resolve_target: Some(resolve),
                        depth_slice: None,
                        ops: Operations {
                            load,
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                        view: depth,
                        depth_ops: Some(Operations {
                            load: LoadOp::Clear(1.0),
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });

                pass.set_pipeline(&self.render_pipeline);
                pass.set_bind_group(0, cam_bg, &[]);
                self.draw_state
                    .draw_batches(&mut pass, &self.textures, seg.first_batch..end);

                // text isn't segmented yet, so it only goes in the last screen pass
                if self.draw_state.segment_has_text(i) {
                    if let Some(Some(slot)) = self.text_pool.get(i) {
                        slot.brush.draw(&mut pass);
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        self.clear_color = None;

        Ok(self.window.request_redraw())
    }

    pub fn set_camera(&mut self, cam: &Camera2D) {
        let mut cam = cam.clone();
        if let Some(rt) = &cam.render_target {
            cam.width = rt.texture.width as f32;
            cam.height = rt.texture.height as f32;
        }
        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(&cam);

        let slot = self.cam_used;
        self.cam_used += 1;
        if slot < self.cam_pool.len() {
            self.queue
                .write_buffer(&self.cam_pool[slot].0, 0, bytemuck::cast_slice(&[uniform]));
        } else {
            let buffer = self.device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Custom Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });
            let (bg, _) = Self::get_camera_bind_group(&buffer, &self.device);
            self.cam_pool.push((buffer, bg));
        }

        self.draw_state.begin_segment(
            Some(slot),
            cam.render_target.clone(),
            cam.position,
            cam.zoom,
        );
    }

    pub fn set_default_camera(&mut self) {
        self.draw_state
            .begin_segment(None, None, Vector2::zero(), 1.0);
    }
}

impl State {
    pub fn exit(&mut self) {
        self.running = false;
    }

    /// ## Window

    #[inline(always)]
    pub fn get_window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn set_window_passthrough(&self, passthrough: bool) {
        self.window.set_cursor_hittest(!passthrough).unwrap();
    }

    pub fn set_window_level(&self, level: WindowLevel) {
        self.window.set_window_level(level);
    }

    pub fn set_window_title(&mut self, title: &str) {
        self.window.set_title(title);
    }

    pub fn set_window_visible(&mut self, visible: bool) {
        self.window.set_visible(visible);
    }

    pub fn set_window_icon(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
        if let Ok(icon) = winit::window::Icon::from_rgba(rgba, width, height) {
            self.window.set_window_icon(Some(icon));
        }
    }

    pub fn set_window_x(&mut self, x: i32) {
        let current = self.window.outer_position().unwrap_or_default();
        self.window
            .set_outer_position(winit::dpi::PhysicalPosition::new(x, current.y));
    }

    pub fn set_window_y(&mut self, y: i32) {
        let current = self.window.outer_position().unwrap_or_default();
        self.window
            .set_outer_position(winit::dpi::PhysicalPosition::new(current.x, y));
    }

    pub fn set_window_w(&mut self, w: u32) {
        let _ = self
            .window
            .request_inner_size(PhysicalSize::new(w, self.window.outer_size().height));
    }

    pub fn set_window_h(&mut self, h: u32) {
        let _ = self
            .window
            .request_inner_size(PhysicalSize::new(self.window.outer_size().width, h));
    }

    pub fn set_window_size(&mut self, w: u32, h: u32) {
        let _ = self.window.request_inner_size(PhysicalSize::new(w, h));
    }

    pub fn window_width(&self) -> f32 {
        self.window.inner_size().width as f32
    }

    pub fn window_height(&self) -> f32 {
        self.window.inner_size().height as f32
    }

    pub fn window_position(&self) -> (f32, f32) {
        self.window.outer_position().unwrap().into()
    }

    pub fn window_x(&self) -> f32 {
        self.window.outer_position().unwrap().x as f32
    }

    pub fn window_y(&self) -> f32 {
        self.window.outer_position().unwrap().y as f32
    }

    /// ## TIME

    pub fn set_target_fps(&mut self, fps: usize) {
        self.target_fps = fps as f32;
    }

    pub fn set_fps_capped(&mut self, capped: bool) {
        self.fps_cap = capped;
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

    pub fn add_timer(&mut self, timer: Timer) {
        if !self.timers.contains(&timer) {
            timer.0.borrow_mut().is_active = true;
            self.timers.push(timer);
        }
    }

    /// ## FONT

    pub fn load_font_from_path(&mut self, path: &str) {
        let font = FlakeFont::from_path(self.fonts.len(), path);
        self.fonts.push(font);

        self.rebuild_text_brush();
    }

    pub fn load_font_from_bytes(&mut self, bytes: &[u8]) {
        let font = FlakeFont::from_bytes(self.fonts.len(), bytes);
        self.fonts.push(font);

        self.rebuild_text_brush();
    }

    pub fn load_font(&mut self, font: FlakeFont) {
        self.fonts.push(font);

        self.rebuild_text_brush();
    }

    pub fn measure_text(&mut self, text: &str, size: f32, font: Option<&FlakeFont>) -> (f32, f32) {
        // Lazily parse any fonts added since last call (works for every load path)
        while self.metric_fonts.len() < self.fonts.len() {
            let data = &self.fonts[self.metric_fonts.len()].data;
            self.metric_fonts
                .push(FontVec::try_from_vec(data.to_vec()).expect("Invalid font data"));
        }

        let id = font.map_or(0, |f| f.id.0);
        let sf = self.metric_fonts[id].as_scaled(PxScale::from(size));

        let line_h = sf.height() + sf.line_gap();
        let (mut max_w, mut w, mut lines) = (0.0_f32, 0.0_f32, 1);
        let mut prev = None;

        for c in text.chars() {
            if c == '\n' {
                max_w = max_w.max(w);
                w = 0.0;
                lines += 1;
                prev = None;
                continue;
            }
            let g = sf.glyph_id(c);
            if let Some(p) = prev {
                w += sf.kern(p, g);
            }
            w += sf.h_advance(g);
            prev = Some(g);
        }

        (max_w.max(w) / 2.0, (sf.height() + (lines - 1) as f32 * line_h) / 2.0)
    }

    /// # RenderTarget

    pub fn create_render_target(
        &mut self,
        width: u32,
        height: u32,
        filter_type: FilterType,
    ) -> RenderTarget {
        let gpu = GpuTexture::empty(&self.device, width, height, filter_type, self.config.format)
            .expect("Couldn't create empty GpuTexture");

        RenderTarget {
            texture: crate::model::texture::Texture {
                id: self.add_gpu_texture(gpu),
                width,
                height,
            },

            msaa_view: Self::create_msaa_view(&self.device, &self.config, width, height),

            depth: GpuTexture::create_depth_texture(&self.device, width, height, "depth_texture"),
        }
    }
}
