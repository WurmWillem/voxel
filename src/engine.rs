use crate::{
    camera::*,
    instance::*,
    texture::Texture,
    vertices::{self, INDICES},
    Manager,
};
use wgpu::util::DeviceExt;
use winit::{event::*, window::Window};

pub struct Engine {
    pub manager: Manager,

    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    block_bind_group: wgpu::BindGroup,
    camera: Camera,
    cam_uniform: CameraUniform,
    projection: Projection,
    pub camera_controller: CameraController,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    depth_texture: Texture,
    pub mouse_pressed: bool,
}
impl Engine {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let (surface, device, queue, config, size) = Manager::set_wgpu_up(window).await;

        let block_bytes = include_bytes!("block.png");
        let block_texture = Texture::from_bytes(&device, &queue, block_bytes, "block png").unwrap();

        let (block_bind_group, texture_bind_group_layout) =
            Texture::create_bind_groups(&device, &block_texture);

        let camera = Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection =
            Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = CameraController::new(4.0, 0.4);

        let mut cam_uniform = CameraUniform::new();
        cam_uniform.update_view_proj(&camera, &projection); // UPDATED!

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[cam_uniform]),
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

        let (instances, instance_buffer) = Instance::create_instances(&device);

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline = Manager::create_render_pipeline(
            &device,
            &config,
            &texture_bind_group_layout,
            &camera_bind_group_layout,
        );

        let (vertex_buffer, index_buffer) = vertices::generate_buffers(&device);
        let num_indices = INDICES.len() as u32;

        let manager = Manager {
            surface,
            device,
            queue,
            config,
            size,
        };

        Self {
            manager,
            cam_uniform,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            block_bind_group,
            camera,
            camera_controller,
            camera_buffer,
            projection,
            camera_bind_group,
            instances,
            instance_buffer,
            depth_texture,
            mouse_pressed: false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if !(new_size.width > 0 && new_size.height > 0) {
            return;
        }
        self.manager.size = new_size;
        self.manager.config.width = new_size.width;
        self.manager.config.height = new_size.height;
        self.manager
            .surface
            .configure(&self.manager.device, &self.manager.config);
        self.depth_texture = Texture::create_depth_texture(
            &self.manager.device,
            &self.manager.config,
            "depth_texture",
        );
        self.projection.resize(new_size.width, new_size.height);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        // self.camera_controller.process_events(event)
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.cam_uniform
            .update_view_proj(&self.camera, &self.projection);
        self.manager.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.cam_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.manager.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.manager
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.2,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.set_bind_group(0, &self.block_bind_group, &[]);
            // render_pass.draw_indexed(0..(self.num_indices - 6), 0, 0..self.instances.len() as _);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);

            /*render_pass.set_bind_group(0, &self.grass_bind_group, &[]);
            render_pass.draw_indexed(
                (self.num_indices - 6)..self.num_indices,
                0,
                0..self.instances.len() as _,
            );*/
        }
        self.manager.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
