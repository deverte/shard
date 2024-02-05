use std::io::Write;
use wgpu::util::DeviceExt;


pub fn save_png(
    image_data: Vec<u8>,
    wh: (usize, usize),
    path: String,
) {
    log::info!("Saving PNG...");

    let mut png_data = Vec::<u8>::with_capacity(image_data.len());
    let mut encoder = png::Encoder::new(
        std::io::Cursor::new(&mut png_data),
        wh.0 as u32,
        wh.1 as u32,
    );
    encoder.set_color(png::ColorType::Rgba);
    let mut png_writer = encoder.write_header().unwrap();
    png_writer.write_image_data(&image_data[..]).unwrap();
    png_writer.finish().unwrap();

    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(&png_data[..]).unwrap();

    log::info!("PNG saved.");
}


pub async fn read_and_save(
    input: std::path::PathBuf,
    output: std::path::PathBuf,
    wh: (usize, usize),
) {
    log::info!("Reading shader from file and saving...");

    let source = std::fs::read_to_string(input).expect("Can't read shader.");
    let image_data = render(source, wh, None).await;
    save_png(
        image_data.to_vec(),
        wh,
        output.into_os_string().into_string().unwrap(),
    );

    log::info!("Done.");
}


pub async fn render(
    source: String,
    wh: (usize, usize),
    frame: Option<usize>,
) -> Vec<u8> {
    log::info!("Rendering...");

    let mut texture_data = Vec::<u8>::with_capacity(wh.0 * wh.1 * 4);

    log::info!("Creating context...");

    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .unwrap();

    let shader = device.create_shader_module(
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                std::borrow::Cow::Borrowed(&source),
            ),
        },
    );

    let render_target = device.create_texture(
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: wh.0 as u32,
                height: wh.1 as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: (
                wgpu::TextureUsages::RENDER_ATTACHMENT |
                wgpu::TextureUsages::COPY_SRC
            ),
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        },
    );

    let output_staging_buffer = device.create_buffer(
        &wgpu::BufferDescriptor {
            label: None,
            size: texture_data.capacity() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        },
    );

    log::info!("Creating uniforms...");

    let res_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Resolution buffer"),
            contents: bytemuck::cast_slice(&[wh.0, wh.1]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        },
    );

    let frame_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Frame index buffer"),
            contents: bytemuck::cast_slice(&[frame.unwrap_or(0)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        },
    );

    let default_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Default group"),
        },
    );

    let default_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout: &default_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: res_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: frame_buffer.as_entire_binding(),
                },
            ],
            label: Some("Default bind group"),
        }
    );

    log::info!("Uniforms created.");

    let render_pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &default_bind_group_layout,
            ],
            push_constant_ranges: &[],
        }
    );

    let pipeline = device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(
                wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[
                        Some(wgpu::TextureFormat::Rgba8UnormSrgb.into()),
                    ],
                },
            ),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        },
    );

    log::info!("Context created.");

    log::info!("Rendering to texture...");

    let texture_view = render_target
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut command_encoder = device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    {
        let mut render_pass = command_encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            },
        );
        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &default_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    command_encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &render_target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &output_staging_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some((wh.0 * 4) as u32),
                rows_per_image: Some(wh.1 as u32),
            },
        },
        wgpu::Extent3d {
            width: wh.0 as u32,
            height: wh.1 as u32,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(command_encoder.finish()));

    log::info!("Rendered to texture.");

    log::info!("Copying buffer to local...");

    let buffer_slice = output_staging_buffer.slice(..);
    let (sender, receiver) = flume::bounded(1);
    buffer_slice
        .map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
    device.poll(wgpu::Maintain::wait()).panic_on_timeout();
    receiver.recv_async().await.unwrap().unwrap();

    {
        let view = buffer_slice.get_mapped_range();
        texture_data.extend_from_slice(&view[..]);
    }

    output_staging_buffer.unmap();

    log::info!("Copied buffer to local.");

    log::info!("Rendering complete.");

    return texture_data;
}