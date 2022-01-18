mod sdf;

use sdf::{Vector, SDF};
use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::Window,
};

async fn run(event_loop: EventLoop<()>, window: Window) {
	let size = window.inner_size();
	let instance = wgpu::Instance::new(wgpu::Backends::all());
	let surface = unsafe { instance.create_surface(&window) };
	let adapter = instance
		.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::default(),
			force_fallback_adapter: false,
			compatible_surface: Some(&surface),
		})
		.await
		.expect("Failed to find an appropriate adapter");

	let (device, queue) = adapter
		.request_device(&wgpu::DeviceDescriptor::default(), None)
		.await
		.expect("Failed to create device");

	let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
		label: None,
		source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
	});

	let sdf1 = SDF::sphere(0.5).translate(Vector(0.5, 0.0, 0.0));
	let sdf2 = SDF::sphere(0.5);
	let sdf = sdf1.subtract(sdf2);
	let bytes = sdf.to_bytes();
	let extent = wgpu::Extent3d { width: bytes.len() as u32 / 4, ..wgpu::Extent3d::default() };
	let texture = device.create_texture(&wgpu::TextureDescriptor {
		label: None,
		size: extent,
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D1,
		format: wgpu::TextureFormat::R32Float,
		usage: wgpu::TextureUsages::TEXTURE_BINDING
			| wgpu::TextureUsages::RENDER_ATTACHMENT
			| wgpu::TextureUsages::COPY_DST,
	});
	let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
	queue.write_texture(
		texture.as_image_copy(),
		&bytes,
		wgpu::ImageDataLayout::default(),
		extent,
	);

	let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label: None,
		entries: &[wgpu::BindGroupLayoutEntry {
			binding: 0,
			visibility: wgpu::ShaderStages::FRAGMENT,
			ty: wgpu::BindingType::Texture {
				multisampled: false,
				sample_type: wgpu::TextureSampleType::Float { filterable: false },
				view_dimension: wgpu::TextureViewDimension::D1,
			},
			count: None,
		}],
	});
	let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &bind_group_layout,
		entries: &[wgpu::BindGroupEntry {
			binding: 0,
			resource: wgpu::BindingResource::TextureView(&view),
		}],
		label: None,
	});

	let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: None,
		bind_group_layouts: &[&bind_group_layout],
		push_constant_ranges: &[],
	});
	let swapchain_format = surface.get_preferred_format(&adapter).unwrap();
	let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: None,
		layout: Some(&pipeline_layout),
		vertex: wgpu::VertexState { module: &shader, entry_point: "vert_main", buffers: &[] },
		fragment: Some(wgpu::FragmentState {
			module: &shader,
			entry_point: "frag_main",
			targets: &[swapchain_format.into()],
		}),
		primitive: wgpu::PrimitiveState::default(),
		depth_stencil: None,
		multisample: wgpu::MultisampleState::default(),
		multiview: None,
	});

	let mut config = wgpu::SurfaceConfiguration {
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
		format: swapchain_format,
		width: size.width,
		height: size.height,
		present_mode: wgpu::PresentMode::Mailbox,
	};

	surface.configure(&device, &config);

	event_loop.run(move |event, _, control_flow| {
		let _ = (&instance, &adapter, &shader, &pipeline_layout);
		*control_flow = ControlFlow::Wait;
		match event {
			Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
				config.width = size.width;
				config.height = size.height;
				surface.configure(&device, &config);
			}
			Event::RedrawRequested(_) => {
				let frame = surface
					.get_current_texture()
					.expect("Failed to acquire next swap chain texture");
				let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
				let mut encoder = device
					.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
				{
					let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
						label: None,
						color_attachments: &[wgpu::RenderPassColorAttachment {
							view: &view,
							resolve_target: None,
							ops: wgpu::Operations {
								load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
								store: true,
							},
						}],
						depth_stencil_attachment: None,
					});
					pass.set_pipeline(&render_pipeline);
					pass.set_bind_group(0, &bind_group, &[]);
					pass.draw(0..6, 0..2);
				}

				queue.submit(Some(encoder.finish()));
				frame.present();
			}
			Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
				*control_flow = ControlFlow::Exit
			}
			_ => {}
		}
	});
}

fn main() {
	let event_loop = EventLoop::new();
	let window = winit::window::Window::new(&event_loop).unwrap();
	env_logger::init();
	pollster::block_on(run(event_loop, window));
}
