
struct Parameters {
    img_size_px: u16,
    workgroup_size: u16,
    max_iter: u16,
    limits: [f32; 4]
}


fn get_params() -> Parameters {
    Parameters {
        img_size_px: 8192,
        workgroup_size: 16,
        max_iter: 256,
        limits: [-2.2, 0.8, -1.5, 1.5]
    }
}


async fn run() {
    let params = get_params();

    let start_time = std::time::Instant::now();
    let steps = execute_gpu(&params).await.unwrap();
    tracing::info!("Computing time: {:?}", start_time.elapsed());

    let img_size = params.img_size_px as u32;
    let mut imgbuf = image::GrayImage::new(img_size, img_size);

    let start_time = std::time::Instant::now();
    for y in 0..img_size {
        for x in 0..img_size {
            let v = steps[(img_size * y + x) as usize] as u8;
            if v != 0 {
                let v = 255.0f32 * (v as f32 / params.max_iter as f32);
                imgbuf.get_pixel_mut(x, y).0[0] = v as u8;
            }
        }
    }
    tracing::info!("Processing time: {:?}", start_time.elapsed());

    let start_time = std::time::Instant::now();
    imgbuf.save("fractal.png").unwrap();
    tracing::info!("Saving time: {:?}", start_time.elapsed());
}


async fn execute_gpu(params: &Parameters) -> Option<Vec<u32>> {
    // Instantiates instance of WebGPU
    let instance = wgpu::Instance::new(wgpu::Backends::all());

    // `request_adapter` instantiates the general connection to the GPU
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None
        })
        .await?;

    let mut requested_limits = wgpu::Limits::default();
    // requesting buffer binds of 256MB
    requested_limits.max_storage_buffer_binding_size = 2 << 28;

    // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
    //  `features` being the available features.
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: requested_limits,
            },
            None,
        )
        .await
        .unwrap();

    tracing::info!("Selected device: {:?}", adapter.get_info());

    execute_gpu_inner(&device, &queue, params).await
}


async fn execute_gpu_inner(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    params: &Parameters
) -> Option<Vec<u32>> {
    assert_eq!(params.img_size_px % params.workgroup_size, 0);
    let no_groups = params.img_size_px / params.workgroup_size;

    let mut shader_src = include_str!("mandelbrot.wgsl").to_string();
    shader_src = shader_src.replace("{{wg_size}}", &*format!("{}u", params.workgroup_size));
    shader_src = shader_src.replace("{{max_iter}}", &*format!("{}u", params.max_iter));
    shader_src = shader_src.replace("{{row_stride}}", &*format!("{}u", params.img_size_px));
    shader_src = shader_src.replace("{{img_size}}", &*format!("{}.0f", params.img_size_px));

    shader_src = shader_src.replace("{{img_min_x}}", &*format!("{}f", params.limits[0]));
    shader_src = shader_src.replace("{{img_max_x}}", &*format!("{}f", params.limits[1]));
    shader_src = shader_src.replace("{{img_min_y}}", &*format!("{}f", params.limits[2]));
    shader_src = shader_src.replace("{{img_max_y}}", &*format!("{}f", params.limits[3]));

    // Loads the shader from WGSL
    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(shader_src)),
    });

    let buffer_size: usize = std::mem::size_of::<u32>() * params.img_size_px as usize * params.img_size_px as usize;
    let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: buffer_size as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &cs_module,
        entry_point: "main",
    });

    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        }],
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.insert_debug_marker("compute collatz iterations");
        cpass.dispatch_workgroups(no_groups as u32, no_groups as u32, 1); // Number of cells to run, the (x,y,z) size of item being processed
    }

    queue.submit(Some(encoder.finish()));

    let buffer_slice = storage_buffer.slice(..);
    let (sender, receiver) = futures::channel::oneshot::channel::<Result<(), wgpu::BufferAsyncError>>();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    device.poll(wgpu::Maintain::Wait);

    if let Ok(()) = receiver.await.unwrap() {
        let data = buffer_slice.get_mapped_range();
        let result = bytemuck::cast_slice(&data).to_vec();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        storage_buffer.unmap();

        Some(result)
    } else {
        panic!("failed to run compute on gpu!")
    }
}


pub fn init_logger(log_level: tracing::Level, log_spans: tracing_subscriber::fmt::format::FmtSpan) -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(log_level)
        .with_span_events(log_spans)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger(tracing::Level::INFO,
                tracing_subscriber::fmt::format::FmtSpan::ENTER |
                    tracing_subscriber::fmt::format::FmtSpan::CLOSE
    )?;

    pollster::block_on(run());
    Ok(())
}
