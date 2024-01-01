//! Demo for a Shipyard game

#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

mod math;
mod sdf;

pub use std::collections::HashMap;
pub use wgpu::Color;
pub use {math::*, sdf::*};

#[derive(Clone, Copy)]
#[repr(C)]
struct Camera {
    pos: Vec3,
    dir: Vec3,
}

#[repr(C)]
struct GpuUniforms {
    window_width: f32,
    window_height: f32,
    seconds: f32,

    min_dist: f32,
    max_dist: f32,
    max_iter: u32,

    camera: Camera,
}

unsafe fn to_byte_slice<T>(x: &T, size: usize) -> &[u8] {
    std::slice::from_raw_parts((x as *const T).cast::<u8>(), size)
}

#[allow(clippy::semicolon_if_nothing_returned)] // pollster macro trips this lint
#[pollster::main]
async fn main() {
    // winit
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = winit::window::Window::new(&event_loop).unwrap();
    window.set_title("The Shipyard");

    // wgpu
    let instance = wgpu::Instance::default();
    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .unwrap();

    // wgpu (pipeline)
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(include_str!("shader.wgsl"))),
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let format = surface.get_capabilities(&adapter).formats[0];
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vertex",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fragment",
            targets: &[Some(format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    // shipyard
    let mut input = winit_input_helper::WinitInputHelper::new();
    let mut timer = std::time::Instant::now();
    let mut seconds = 0.0;
    let mut camera = Camera {
        pos: ORIGIN,
        dir: Z,
    };

    let scene = Sdf::sphere(1.0, Material::Flat(Color::RED))
        .union(Sdf::sphere(1.0, Material::Flat(Color::GREEN)).translate(X));

    event_loop
        .run(|event, window_target| {
            if input.update(&event) {
                if input.close_requested() || input.destroyed() {
                    window_target.exit();
                }

                // update
                let delta = timer.elapsed();
                timer = std::time::Instant::now();
                seconds += delta.as_secs_f32();

                let angle = seconds * (2.0 * PI) * 0.1;
                camera.pos = Vec3 {
                    x: angle.cos(),
                    y: angle.sin(),
                    z: -5.0,
                };

                // render
                let size = window.inner_size();
                surface.configure(
                    &device,
                    &wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format,
                        width: size.width.max(1),
                        height: size.height.max(1),
                        present_mode: wgpu::PresentMode::Fifo,
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        view_formats: vec![],
                    },
                );

                let uniforms_data = GpuUniforms {
                    window_width: size.width as f32,
                    window_height: size.height as f32,
                    seconds,
                    min_dist: MIN_DIST,
                    max_dist: MAX_DIST,
                    max_iter: MAX_ITER as u32,
                    camera,
                };
                let (distances_data, materials_data) = scene.to_gpu_repr();

                let uniforms_size = std::mem::size_of::<GpuUniforms>();
                let uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: uniforms_size as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                queue.write_buffer(&uniforms_buffer, 0, unsafe {
                    to_byte_slice(&uniforms_data, uniforms_size)
                });

                let distances_size = std::mem::size_of::<GpuDistance>() * distances_data.len();
                let distances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: distances_size as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                queue.write_buffer(&distances_buffer, 0, unsafe {
                    to_byte_slice(&distances_data[0], distances_size)
                });

                let materials_size = std::mem::size_of::<GpuMaterial>() * materials_data.len();
                let materials_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: materials_size as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                queue.write_buffer(&materials_buffer, 0, unsafe {
                    to_byte_slice(&materials_data[0], materials_size)
                });

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &uniforms_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &distances_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &materials_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                    ],
                });

                let frame = surface.get_current_texture().unwrap();
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    pass.set_pipeline(&pipeline);
                    pass.set_bind_group(0, &bind_group, &[]);
                    pass.draw(0..3, 0..1);
                }
                queue.submit([encoder.finish()]);
                frame.present();

                window.request_redraw();
            }
        })
        .unwrap();
}
