use std::{f32, time::SystemTime};

use wgpu::util::DeviceExt;
use winit::event::{Event, WindowEvent};
pub mod camera;
mod chunk_gen;
pub mod display_handler;
pub mod instances;
pub mod texture;
use camera::Camera;
use cgmath::prelude::*;
use instances::*;
use texture::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct Storrage {
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    vertex_list: Vec<Vertex>,
    indecies: Vec<u16>,
    instances: Vec<instances::CFrame>,
    diffuse_bind_group: wgpu::BindGroup,
}

impl Storrage {
    fn update_instance_buffer(&mut self, queue: &wgpu::Queue) {
        let instance_data = self
            .instances
            .iter()
            .map(|v| v.to_raw())
            .collect::<Vec<_>>();

        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instance_data));
    }
}

struct RenderScene<'a> {
    render_pipeline: &'a wgpu::RenderPipeline,
    camera_bind_group: &'a wgpu::BindGroup,
    camera_uniform: CameraUniform,
    camera: &'a Camera,
    queue: &'a wgpu::Queue,
    device: &'a wgpu::Device,
    surface: &'a wgpu::Surface,
    window: &'a winit::window::Window,
    buffers: &'a Storrage,
    depth_texture: &'a Texture,
    start_time: SystemTime,
}

fn render_scene(scene: &mut RenderScene) {
    let time = scene.start_time.elapsed().unwrap().as_secs_f32();

    scene.queue.write_buffer(
        &scene.buffers.uniform_buffer,
        0,
        &bytemuck::cast_slice(&[time]),
    );

    let frame = scene
        .surface
        .get_current_texture()
        .expect("Failed to get Current texture");

    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoer = scene
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut render_pass = encoer.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                    store: wgpu::StoreOp::Store,
                },
            })],

            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &scene.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        scene.camera_uniform.update_view_proj(&scene.camera);
        scene.queue.write_buffer(
            &scene.buffers.camera_buffer,
            0,
            bytemuck::cast_slice(&[scene.camera_uniform]),
        );
        render_pass.set_pipeline(&scene.render_pipeline);
        render_pass.set_vertex_buffer(0, scene.buffers.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, scene.buffers.instance_buffer.slice(..));
        render_pass.set_bind_group(0, scene.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &scene.buffers.diffuse_bind_group, &[]);
        render_pass.set_index_buffer(
            scene.buffers.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(
            0..scene.buffers.indecies.len() as u32,
            0,
            0..scene.buffers.instances.len() as _,
        );
    }
    scene.queue.submit(Some(encoer.finish()));
    frame.present();
    scene.window.request_redraw();
}

pub async fn run(game_window: display_handler::GameWindow) {

    let device = &game_window.device;
    let adapter = &game_window.adapter;
    let surface = &game_window.surface;
    let size = game_window.window.inner_size();

    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    let mut cam = Camera::default(&config);

    let mut camera_uniform = CameraUniform::new();
    camera_uniform.update_view_proj(&cam);

    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&Vec::<u8>::new()),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&Vec::<u8>::new()),
        usage: wgpu::BufferUsages::INDEX,
    });

    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&Vec::<u8>::new()),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: std::mem::size_of::<f32>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let diffuse_texture = chunk_gen::generate_chunk(&device, &game_window.queue);

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D3,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    });
    let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            },
        ],
        label: Some("diffuse_bind_group"),
    });

    let camera_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
        label: Some("camera_bind_group"),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pipeline_layout"),
        bind_group_layouts: &[&camera_bind_group_layout, &bind_group_layout],
        push_constant_ranges: &[],
    });

    let depth_texture = texture::Texture::cretate_depth_texture(device, &config);

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),

        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc(), InstanceRaw::desc()],
        },

        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState{
                format: swapchain_format.into(),

                blend: Some(wgpu::BlendState{
                    color: wgpu::BlendComponent{
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,},
                    alpha: wgpu::BlendComponent::OVER
                }),

                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),

        primitive: wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),

        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut buffers = Storrage {
        uniform_buffer,
        vertex_buffer,
        diffuse_bind_group,
        camera_buffer,
        index_buffer,
        instance_buffer,
        vertex_list: vec![],
        indecies: vec![],
        instances: Vec::new(),
    };

    let test = //Mesh::default();
        instances::Mesh::from_file_obj(include_str!("./../../assets/untitled.obj").to_string());

    test.load(&mut buffers, device);
    buffers.update_instance_buffer(&game_window.queue);

    let mut cam_controller = camera::CameraController::new(0.01);
    let start_time = SystemTime::now();

    surface.configure(&device, &config);
    game_window
        .event_loop
        .run(move |event, target| {
            let _ = (&adapter, &shader, &pipeline_layout);

            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                cam_controller.process_events(&event);
                cam_controller.update_camera(&mut cam);
                match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::Resized(physical_size) => {
                        config.width = physical_size.width.max(1);
                        config.height = physical_size.height.max(1);
                        surface.configure(device, &config);
                        game_window.window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        //test.cframe.position.y = 10.0;
                        //buffers.update_instance_buffer(&device);

                        render_scene({
                            &mut RenderScene {
                                start_time,
                                render_pipeline: &render_pipeline,
                                depth_texture: &depth_texture,
                                camera_bind_group: &camera_bind_group,
                                surface,
                                device,
                                window: &game_window.window,
                                queue: &game_window.queue,
                                camera_uniform,
                                camera: &cam,
                                buffers: &buffers,
                            }
                        });
                    }
                    _ => {}
                }
            }
        })
        .unwrap();
}
