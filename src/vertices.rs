use wgpu::{util::DeviceExt, Buffer, Device};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}
impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub fn generate_buffers(device: &Device) -> (Buffer, Buffer) {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("index buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    (vertex_buffer, index_buffer)
}

#[rustfmt::skip]
pub const VERTICES: &[Vertex] = &[
    //0, 1, 2, 3
    Vertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [0.5, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [0.5, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },

    //4, 5, 6, 7
    Vertex {
        position: [-1.0, -1.0, 2.0],
        tex_coords: [0.5, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 2.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 2.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 2.0],
        tex_coords: [0.5, 0.0],
    },

    //8, 9, 10, 11, grass tex
    Vertex { 
        position: [1.0, 1.0, 0.0],  //front right
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0], //front left
        tex_coords: [0.5, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 2.0],  //back right
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 2.0], //back left
        tex_coords: [0.5, 0.0],
    },
];
#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, 
    4, 5, 6, 6, 7, 4,
    1, 2, 5, 6, 5, 2,
    0, 3, 4, 3, 4, 7,

    8, 9, 10, 10, 11, 9
    // 0, 1, 4, 4, 1, 5
    ];

//pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
