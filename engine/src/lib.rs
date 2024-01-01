use wgpu::util::DeviceExt;
use winit::event::{Event, WindowEvent};
pub mod camera;
pub mod display_handler;
use camera::Camera;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

struct RenderScene<'a> {
    render_pipeline: &'a wgpu::RenderPipeline,
    vertex_buffer: &'a wgpu::Buffer,
    elements_to_draw: u32,
    queue: &'a wgpu::Queue,
    device: &'a wgpu::Device,
    surface: &'a wgpu::Surface,
    window: &'a winit::window::Window,
    camera_bind_group: &'a wgpu::BindGroup,
    camera_buffer: &'a mut wgpu::Buffer,
    camera_uniform: CameraUniform,
    camera : &'a Camera,
}

fn render_scene(scene: &mut RenderScene) {
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

            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        scene.camera_uniform.update_view_proj(&scene.camera);
        scene.queue.write_buffer(&scene.camera_buffer, 0, bytemuck::cast_slice(&[scene.camera_uniform]));
        render_pass.set_pipeline(&scene.render_pipeline);
        render_pass.set_vertex_buffer(0, scene.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, scene.camera_bind_group, &[]);
        render_pass.draw(0..scene.elements_to_draw, 0..1);
    }
    scene.queue.submit(Some(encoer.finish()));
    frame.present();
    scene.window.request_redraw();
}

pub async fn run(game_window: display_handler::GameWindow) {
    let vertesices: Vec<Vertex> = vec![
        Vertex {
            position: [-1.0, -1.0, 0.0],
            color: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, -1.0, 0.0],
            color: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [0.0, 1.0, 0.0],
            color: [0.0, 0.0, 1.0],
        },
    ];
    let device = &game_window.device;
    let adapter = &game_window.adapter;
    let surface = &game_window.surface;
    let size = game_window.window.inner_size();

    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertesices),
        usage: wgpu::BufferUsages::VERTEX,
    });

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

    let mut camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let camera_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
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
        bind_group_layouts: &[&camera_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),

        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },

        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),

        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });



    let mut cam_controller = camera::CameraController::new(0.1);

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
                    WindowEvent::RedrawRequested => render_scene({
                        &mut RenderScene {
                        render_pipeline: &render_pipeline,
                        vertex_buffer: &vertex_buffer,
                        camera_bind_group: &camera_bind_group,
                        elements_to_draw: vertesices.len() as u32,
                        surface,
                        device,
                        window: &game_window.window,
                        queue: &game_window.queue,
                        camera_uniform,
                        camera : &cam,
                        camera_buffer : &mut camera_buffer,
                    }}),
                    _ => {}
                }
            }
        })
        .unwrap()
}
