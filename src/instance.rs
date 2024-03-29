use cgmath::{InnerSpace, Rotation3, Zero};
use wgpu::{util::DeviceExt, Buffer, Device};

pub const INST_PER_ROW: usize = 1414;
pub struct Instance {
    pub pos: cgmath::Vector3<f32>,
    pub rot: cgmath::Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    //Data that will go in buffer
    model: [[f32; 4]; 4],
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        /*println!(
            "{:?}",
            cgmath::Matrix4::from_translation(self.pos)
                * cgmath::Matrix4::from(self.rot)
                * cgmath::Matrix4::from_scale(0.5)
        );*/
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.pos)
                * cgmath::Matrix4::from(self.rot)
                * cgmath::Matrix4::from_scale(0.5))
            .into(),
        }
    }

    pub fn create_instances(device: &Device) -> (Vec<Instance>, Buffer) {
        use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
        use noise::{Fbm, Perlin};
        let fbm = Fbm::<Perlin>::new(0);

        let p = PlaneMapBuilder::<_, 2>::new(&fbm)
            .set_size(INST_PER_ROW, INST_PER_ROW)
            .build();

        let mut instances = vec![];
        for z in 0..INST_PER_ROW {
            for y in 0..1 {
                for x in 0..INST_PER_ROW {
                    let v = p.get_value(x, z) + 1.006;
              
                    let mut pos = cgmath::Vector3 {
                        x: x as f32,
                        y: (v * 100.) as u32 as f32 + y as f32,
                        z: z as f32,
                    };
                    // pos.y = pos.y * 40. + y as f32;

                    let rotation = if pos.is_zero() {
                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can effect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(pos.normalize(), cgmath::Deg(0.0))
                    };

                    instances.push(Instance { pos, rot: rotation });
                }
            }
        }

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        (instances, instance_buffer)
    }
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in
                // the shader.
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
