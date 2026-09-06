#![allow(unused_doc_comments)]
use crate::input::keyboard::KeyboardState;
use crate::model::camera::{Camera, CameraUniform};
use crate::model::instance::{Instance, InstanceRaw};
use crate::model::texture;
use crate::model::vertex::{INDICES, VERTICES, Vertex};
use cgmath::prelude::*;
use std::sync::Arc;
use wgpu::{self, util::DeviceExt, *};
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

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
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,

    diffuse_bind_group: BindGroup,
    depth_texture: texture::Texture,

    // Camera
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,

    // Input
    keyboard_state: KeyboardState,

    // Instancing
    instances: Vec<Instance>,
    instance_buffer: Buffer,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let (instance, surface) = Self::get_instance_and_surface(window.clone())?;
        let adapter = Self::get_adapter(&surface, &instance).await?;
        let (device, queue) = Self::get_device_and_queue(&adapter).await?;
        let config = Self::get_surface_config(&surface, size, &adapter);

        /// ## Camera
        let camera = Self::get_camera(config.width, config.height);
        let (camera_uniform, camera_buffer) = Self::get_camera_uniform_buffer(&camera, &device);
        let (camera_bind_group, camera_bind_group_layout) =
            Self::get_camera_bind_group(&camera_buffer, &device);

        /// ## Instancing
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    } - INSTANCE_DISPLACEMENT;

                    let rotation = if position.is_zero() {
                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can affect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: BufferUsages::VERTEX,
        });

        /// ## Textures
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");
        let (texture_bind_group_layout, diffuse_bind_group) =
            Self::get_texture_bind_group(&device, &queue);

        /// # Render Pipeline
        let render_pipeline = Self::get_render_pipeline(
            &device,
            &config,
            &texture_bind_group_layout,
            &camera_bind_group_layout,
        );

        /// ## Vertex & Input Buffers
        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        Ok(Self {
            window,

            surface,
            device,
            queue,
            config,
            is_surface_configured: false,

            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: INDICES.len() as u32,

            depth_texture,
            diffuse_bind_group,

            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,

            keyboard_state: KeyboardState::default(),

            instances,
            instance_buffer,
        })
    }

    fn get_camera(width: u32, height: u32) -> Camera {
        Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: width as f32 / height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    fn get_camera_uniform_buffer(camera: &Camera, device: &Device) -> (CameraUniform, Buffer) {
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
    ) -> anyhow::Result<(wgpu::Instance, Surface<'static>)> {
        let instance = wgpu::Instance::new(InstanceDescriptor {
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
        instance: &wgpu::Instance,
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
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: Default::default(),
        }
    }

    /// Loads the diffuse texture and builds its bind group layout + bind group.
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
        texture_bind_group_layout: &BindGroupLayout,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> RenderPipeline {
        let shader = device.create_shader_module(include_wgsl!("shaders/shader.wgsl"));

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                Some(texture_bind_group_layout),
                Some(camera_bind_group_layout),
            ],
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
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),

            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                ..Default::default()
            },

            depth_stencil: Some(DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::Less),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),

            multisample: MultisampleState::default(),
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
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        self.keyboard_state.set_key_pressed(code, is_pressed);

        if let (KeyCode::Escape, true) = (code, is_pressed) {
            event_loop.exit();
        }
    }

    pub(crate) fn update(&mut self) {
        if self.keyboard_state.is_key_pressed(KeyCode::KeyA) {
            self.camera.eye -= cgmath::Vector3::unit_x() * 0.05;
            self.camera.target -= cgmath::Vector3::unit_x() * 0.05;
        }
        if self.keyboard_state.is_key_pressed(KeyCode::KeyD) {
            self.camera.eye += cgmath::Vector3::unit_x() * 0.05;
            self.camera.target += cgmath::Vector3::unit_x() * 0.05;
        }
        if self.keyboard_state.is_key_pressed(KeyCode::KeyW) {
            self.camera.eye -= cgmath::Vector3::unit_z() * 0.05;
            self.camera.target -= cgmath::Vector3::unit_z() * 0.05;
        }
        if self.keyboard_state.is_key_pressed(KeyCode::KeyS) {
            self.camera.eye += cgmath::Vector3::unit_z() * 0.05;
            self.camera.target += cgmath::Vector3::unit_z() * 0.05;
        }
        if self.keyboard_state.is_key_pressed(KeyCode::KeyQ) {
            self.camera.eye -= cgmath::Vector3::unit_y() * 0.05;
            self.camera.target -= cgmath::Vector3::unit_y() * 0.05;
        }
        if self.keyboard_state.is_key_pressed(KeyCode::KeyE) {
            self.camera.eye += cgmath::Vector3::unit_y() * 0.05;
            self.camera.target += cgmath::Vector3::unit_y() * 0.05;
        }

        // Update camera uniform
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
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
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
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

            // Set the pipeline
            pass.set_pipeline(&self.render_pipeline);

            // Bind Groups
            pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            pass.set_bind_group(1, &self.camera_bind_group, &[]);

            // Buffers
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

            // Draw
            pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);
        Ok(())
    }
}
