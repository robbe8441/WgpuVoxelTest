
use winit::{
    event::{Event, WindowEvent, self},
    event_loop::EventLoop,
    window::{Window, self}, raw_window_handle::{WindowHandle, HasRawWindowHandle, HasDisplayHandle, DisplayHandle, HasWindowHandle},
};

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    let instance = wgpu::Instance::default();

    let surface = unsafe {instance.create_surface(&window).unwrap()};
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference : wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter : false,
        compatible_surface : Some(&surface)
    }).await.expect("Cant create adapter");

    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label : Some("device"),
            features : wgpu::Features::empty(),
            limits : wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
        }, None)
    .await.expect("cant create device");

    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label : Some("pipeline_layout"),
        bind_group_layouts : &[],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];



    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label : Some("Render Pipeline"),
        layout: Some(&pipeline_layout),

        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[]
        },

        fragment : Some(wgpu::FragmentState { module: &shader, entry_point: "fs_main", 
            targets: &[Some(swapchain_format.into())]

        }),

        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None
    });


    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![]
    };

    surface.configure(&device, &config);

    let window = &window;
    event_loop.run(move |event, target| {
        let _ = (&instance, &adapter, &shader, &pipeline_layout);

        if let Event::WindowEvent {
            window_id : _,
            event,
        } = event {
            match event {
                WindowEvent::CloseRequested => target.exit(),
                _ => {}
            }
        }

    }).unwrap()
}


pub async fn create_window() {
    let event_loop = EventLoop::new().unwrap();
    let builder = winit::window::WindowBuilder::new();
    let window = builder.build(&event_loop).unwrap();

    run(event_loop, window).await;
}
