mod camera;
mod boid;

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{WindowBuilder, Window},
};

use wgpu::util::DeviceExt;

use rand::prelude::*;

use camera::{Camera, CameraUniform};
use boid::Boid;

struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    size: winit::dpi::PhysicalSize<u32>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    frame_count: usize,

    boids_buffers: Vec<wgpu::Buffer>,
    boids_bind_groups: Vec<wgpu::BindGroup>,
    
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    staging_buffer: wgpu::util::StagingBelt,

    vertex_buffer: wgpu::Buffer,

    render_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,

    window: &'a Window,
}

const VERTICES: &[[f32; 3]] = &[
    [ 1.00000,  0.00000, 1.0],
    [-0.83147,  0.55557,  1.0],
    [-0.83147, -0.55557,  1.0],
];

const N_BOIDS: usize = 10000;

impl<'a> Renderer<'a> {
    async fn new(window: &'a Window) -> Renderer<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|r| r.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };


        let frame_count = 0;


        let wd = 1024.0;
        let ht = 1024.0;
        let mut rng = rand::rng();
        let mut boids = Vec::new();
        for _ in 0..N_BOIDS {
            let x = wd * rng.random::<f32>() - (wd / 2.0);
            let y = ht * rng.random::<f32>() - (ht / 2.0);
            let a = rng.random::<f32>() * 6.28318;
            let (vy, vx) = f32::sin_cos(a);
            let boid = Boid::new(x, y, vx, vy);
            boids.push(boid);
        }
        

        let mut boids_buffers = Vec::new();
        for i in 0..2 {
            let buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(format!("Boids Buffer {}", i).as_str()),
                    contents: bytemuck::cast_slice(&boids),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                }
            );
            boids_buffers.push(buffer);
        }

        let boids_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Boid Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
            }
        );

        let mut boids_bind_groups = Vec::new();
        for i in 0..2 {
            let bind_group = device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some(format!("Bind Group {}", i).as_str()),
                    layout: &boids_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: boids_buffers[i % 2].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: boids_buffers[(i + 1) % 2].as_entire_binding(),
                        },
                    ],
                }
            );
            boids_bind_groups.push(bind_group);
        }


        let camera = Camera::new(size);
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera.into_matrix()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ]
            }
        );
        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }
                ]
            }
        );


        let staging_buffer = wgpu::util::StagingBelt::new((std::mem::size_of::<CameraUniform>() + 4) as wgpu::BufferAddress);


        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let render_shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &render_shader,
                    entry_point: "vs_main",
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Boid>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32x2],
                        },
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &render_shader,
                    entry_point: "fs_main",
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            }
        );

        let compute_shader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));

        let compute_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[
                    &boids_bind_group_layout
                ],
                push_constant_ranges: &[],
            }
        );

        let compute_pipeline = device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: Some("Compute Pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &compute_shader,
                entry_point: "cs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            }
        );


        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        Self {
            surface,
            size,
            device,
            queue,
            config,

            frame_count,

            boids_buffers,
            boids_bind_groups,

            camera,
            camera_buffer,
            camera_bind_group,

            staging_buffer,

            vertex_buffer,

            render_pipeline,
            compute_pipeline, 

            window,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;

            self.surface.configure(&self.device, &self.config);
            self.camera.update_scale(new_size);
        }
    }

    fn update(&mut self) {
        let mut update_encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Staging Buffer Encoder"),
            }
        );

        use std::num::NonZero;
        let size = NonZero::new(std::mem::size_of::<CameraUniform>() as u64).unwrap();
        self.staging_buffer.write_buffer(&mut update_encoder, &self.camera_buffer, 0, size, &self.device)
            .copy_from_slice(bytemuck::cast_slice(&[self.camera.into_matrix()]));

        self.staging_buffer.finish();
        self.queue.submit(std::iter::once(update_encoder.finish()));
        self.staging_buffer.recall();

        let mut compute_encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Compute Pass Encoder")
            }
        );

        let mut compute_pass = compute_encoder.begin_compute_pass(
            &wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            }
        );

        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.boids_bind_groups[self.frame_count % 2], &[]);
        compute_pass.dispatch_workgroups(1024, 1, 1);

        drop(compute_pass);

        self.queue.submit(std::iter::once(compute_encoder.finish()));
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera.process_events(event)
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder")
            }
        );

        let mut render_pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.009021491898012131,
                                g: 0.009021491898012131,
                                b: 0.023103556157921437,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        }
                    })
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            }
        );

        let instance_buffer = &self.boids_buffers[(self.frame_count + 1) % 2];

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);


        render_pass.draw(0..VERTICES.len() as u32, 0..N_BOIDS as u32);

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        self.frame_count = (self.frame_count + 1) % usize::MAX;
        Ok(())

    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_decorations(false)
        //.with_inner_size(winit::dpi::PhysicalSize{width: 1280*2, height: 720*2})
        .build(&event_loop).unwrap();
    window.set_cursor_visible(false);

    let mut renderer = Renderer::new(&window).await;
    let mut surface_configured = false;

    let _ = event_loop.run(move |event, control_flow| {
        if let Event::WindowEvent { window_id, ref event } = event {
            if window_id != renderer.window().id() { return }
            if renderer.input(event) { return }

            if renderer.frame_count != 0 {
                renderer.update();
            }
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput { 
                    event: KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                    ..
                } => control_flow.exit(),

                WindowEvent::Resized(physical_size) => {
                    log::info!("physical_size: {physical_size:?}");
                    surface_configured = true;
                    renderer.resize(*physical_size);
                },

                WindowEvent::RedrawRequested => {
                /*WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Enter),
                        ..
                    },
                    ..
                } => { */
                    renderer.window().request_redraw();

                    if !surface_configured { return; }

                    renderer.update();
                    match renderer.render() {
                        Ok(_) => {}
                                                    // reconfigure the surface if it's lost or outdated
                        Err(
                            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                        ) => renderer.resize(renderer.size),

                        // system is out of memory so quit
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("OutOfMemory");
                            control_flow.exit();
                        }

                        // happens when frame takes too long to present
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface Timeout");
                        }
                    }
                }
                _ => {}

            }
        }
    });
}
