use crate::parameters::Parameters;


pub async fn run_wgpu(params: &Parameters) -> Vec<u8> {
    // Instantiates instance of WebGPU
    let instance = wgpu::Instance::new(wgpu::Backends::all());

    // `request_adapter` instantiates the general connection to the GPU
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None
        })
        .await.unwrap();

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
) -> Vec<u8> {
    let (wg_size, shader_module) = crate::shaders::provider::get_shader(&params, &device);

    let buffer_size: usize = std::mem::size_of::<u8>() * params.img_size_px as usize * params.img_size_px as usize;
    let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: buffer_size as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &shader_module,
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
        cpass.dispatch_workgroups(wg_size.0, wg_size.1, wg_size.2);
    }

    queue.submit(Some(encoder.finish()));

    let buffer_slice = storage_buffer.slice(..);
    let (sender, receiver) = futures::channel::oneshot::channel::<Result<(), wgpu::BufferAsyncError>>();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    device.poll(wgpu::Maintain::Wait);

    receiver.await.unwrap().unwrap();
    let data = buffer_slice.get_mapped_range();
    let result = data.to_vec();

    // With the current interface, we have to make sure all mapped views are
    // dropped before we unmap the buffer.
    drop(data);
    storage_buffer.unmap();

    result
}
