use winit::{
    window::{Window, Icon},
    event_loop::EventLoop,
};
use std::path::Path;


pub struct GameWindow {
    pub window: Window,
    pub event_loop : EventLoop<()>,
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    pub adapter: wgpu::Adapter,
    pub surface: wgpu::Surface,

}

impl GameWindow {
    pub async fn new() -> Self {
        let (window, event_loop) = create_window().await;
        let (device, queue, adapter, surface) = create_surface(&window).await;

        GameWindow {
            surface,
            window,
            event_loop,
            queue,
            device,
            adapter,
        }
    }
}




async fn create_surface(window:&Window) -> (wgpu::Device, wgpu::Queue, wgpu::Adapter, wgpu::Surface) {
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&window).unwrap() };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Cant create adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("device"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("cant create device");

    (device, queue, adapter, surface)
}



pub async fn create_window() -> (Window, EventLoop<()>){
    let event_loop = EventLoop::new().unwrap();
    let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/Icon.png");
    println!("{}", icon_path);

    let builder = winit::window::WindowBuilder::new()
        .with_title("Voxel Game Engine")
        .with_window_icon(Some(load_icon(std::path::Path::new(icon_path))))
        .with_theme(Some(winit::window::Theme::Dark));
    let window = builder.build(&event_loop).unwrap();

    (window, event_loop)
}

fn load_icon(path: &Path) -> Icon{
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
