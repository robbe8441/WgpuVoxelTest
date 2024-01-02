use crate::Storrage;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CFrame {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub size: cgmath::Vector3<f32>,
}

impl Default for CFrame {
    fn default() -> Self {
        CFrame {
            position : [0.0, 0.0, 0.0].into(),
            rotation : cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0),
            size : [1.0, 1.0, 1.0].into(),
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
    pub cframe : CFrame,
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

    pub fn load(&mut self, store : &mut Storrage, device: &wgpu::Device) {
        let to_move = self.indicies.drain(..);
        store.indecies.extend(to_move);

        let new_vertex = self.vertecies.drain(..);
        store.vertex_list.extend(new_vertex);

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

    store.instances.push(self.cframe);
    let instance_data = store.instances.iter().map(CFrame::to_raw).collect::<Vec<_>>();

    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&instance_data),
        usage: wgpu::BufferUsages::VERTEX,
    });

    println!("{:?}", store.instances);

    store.vertex_buffer = vertex_buffer;
    store.index_buffer = index_buffer;
    store.instance_buffer = instance_buffer;
    }
}   

impl Default for Mesh {
    fn default() -> Self {
        Mesh {
            cframe : CFrame { 
            size : [0.1, 0.1, 0.1].into(), ..Default::default() },
            vertecies: vec![
                Vertex {
                    position: [-0.5, -0.5, -0.5],
                    color: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [0.5, -0.5, -0.5],
                    color: [0.5, 0.5, 0.0],
                },
                Vertex {
                    position: [0.5, 0.5, -0.5],
                    color: [1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [-0.5, 0.5, -0.5],
                    color: [1.0, 0.5, 0.5],
                },
                Vertex {
                    position: [-0.5, 0.5, 0.5],
                    color: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [0.5, 0.5, 0.5],
                    color: [1.0, 0.0, 1.0],
                },
                
                Vertex {
                    position: [0.5, -0.5, 0.5],
                    color: [0.0, 1.0, 1.0],
                },
                Vertex {
                    position: [-0.5, -0.5, 0.5],
                    color: [1.0, 1.0, 1.0],
                },
                
                
            ],
            indicies: vec![
                0,2,3, 2,0,1,
                3,2,5, 5,4,3,
                7,5,6, 5,7,4,
                0,6,1, 0,7,6,
                1,6,5, 5,2,1,
                4,7,0, 0,3,4

            ],
        }
    }
}
