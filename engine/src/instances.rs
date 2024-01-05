use crate::Storrage;
use std::rc::Rc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    uv_cords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CFrame {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Default for CFrame {
    fn default() -> Self {
        CFrame {
            position: [0.0, 0.0, 0.0].into(),
            rotation: cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

impl CFrame {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Mesh {
    pub cframe: Box<CFrame>,
    pub vertecies: Vec<Vertex>,
    pub indicies: Vec<u16>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Mesh {
    pub fn load(&self, store: &mut Storrage, device: &wgpu::Device) {
        let mut indecies = self.indicies.clone();
        let to_move = indecies.drain(..);
        store.indecies.extend(to_move);

        let mut new_vertex = self.vertecies.clone();
        let move_vertex = new_vertex.drain(..);
        store.vertex_list.extend(move_vertex);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&store.vertex_list),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&store.indecies),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        let cframe_rc = Rc::new(self.cframe.clone());
        store.instances.push(**cframe_rc);
        store.vertex_buffer = vertex_buffer;
        store.index_buffer = index_buffer;
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Mesh {
            cframe: Box::new(CFrame::default()),
            vertecies: vec![
                Vertex {
                    position: [-0.5, -0.5, -0.5],
                    uv_cords: [0.0, 0.0],
                },
                Vertex {
                    position: [0.5, -0.5, -0.5],
                    uv_cords: [0.5, 0.5],
                },
                Vertex {
                    position: [0.5, 0.5, -0.5],
                    uv_cords: [1.0, 0.0],
                },
                Vertex {
                    position: [-0.5, 0.5, -0.5],
                    uv_cords: [0.5, 0.5],
                },
                Vertex {
                    position: [-0.5, 0.5, 0.5],
                    uv_cords: [0.0, 1.0],
                },
                Vertex {
                    position: [0.5, 0.5, 0.5],
                    uv_cords: [0.0, 1.0],
                },
                Vertex {
                    position: [0.5, -0.5, 0.5],
                    uv_cords: [1.0, 1.0],
                },
                Vertex {
                    position: [-0.5, -0.5, 0.5],
                    uv_cords: [1.0, 1.0],
                },
            ],
            indicies: vec![
                0, 2, 3, 2, 0, 1, 3, 2, 5, 5, 4, 3, 7, 5, 6, 5, 7, 4, 0, 6, 1, 0, 7, 6, 1, 6, 5, 5,
                2, 1, 4, 7, 0, 0, 3, 4,
            ],
        }
    }
}
