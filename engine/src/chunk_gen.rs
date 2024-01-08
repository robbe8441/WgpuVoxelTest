use crate::texture::Texture;
use noise::{NoiseFn, SuperSimplex};

const CHUNK_SIZE: u32 = 100;
const NOISE_SIZE: f64 = 50.0;

pub fn generate_chunk(device: &wgpu::Device, queue: &wgpu::Queue) -> Texture {
    let total_blocks = CHUNK_SIZE.pow(3);
    let noise = SuperSimplex::new(5);
    let mut result: Vec<u8> = Vec::with_capacity(total_blocks as usize);

    for i in 0..total_blocks {
        let x = i % CHUNK_SIZE;
        let y = (i / CHUNK_SIZE) % CHUNK_SIZE;
        let z = (i / CHUNK_SIZE.pow(2)) % CHUNK_SIZE;

        let val = noise.get([
            x as f64 / NOISE_SIZE,
            y as f64 / NOISE_SIZE,
            z as f64 / NOISE_SIZE,
        ]);

        if val > 0.0 {
            result.push(255);
        } else {
            result.push(0);
        }

    }

    let size = wgpu::Extent3d {
        width: CHUNK_SIZE,
        height: CHUNK_SIZE,
        depth_or_array_layers: CHUNK_SIZE,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::R8Uint,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        &result,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(CHUNK_SIZE),
            rows_per_image: Some(CHUNK_SIZE),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    Texture {
        view,
        sampler,
        texture,
    }
}
