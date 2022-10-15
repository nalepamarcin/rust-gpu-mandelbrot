use crate::parameters::BackendType;


pub fn get_spirv_configuration(params: &crate::parameters::Parameters)
-> ((u32, u32, u32), Vec<u8>) {
    const WORKGROUP_SIZE: u16 = 16;
    // processing must be divided into full workgroups
    assert_eq!(params.img_size_px % WORKGROUP_SIZE, 0);
    let no_groups = params.img_size_px / WORKGROUP_SIZE;
    // one shader invocation processes 4 consecutive pixels (due to stupid wgsl limitation of working only on u32)
    assert_eq!(no_groups % 4, 0);

    #[repr(C)]
    struct InputParameters {
        draw_bounds: [f32; 4], // -x, x, -y, y
        img_size_px: u32,      // in pixels
        max_iter: u32          // max number of iterations to ru
    }

    let input_parameters = InputParameters {
        draw_bounds: params.limits,
        img_size_px: params.img_size_px as u32,
        max_iter: params.max_iter as u32
    };

    let input_params_as_bytes = unsafe { std::slice::from_raw_parts(
        (&input_parameters as *const InputParameters) as *const u8,
        std::mem::size_of::<InputParameters>()
    )}.to_vec();

    assert_eq!(input_params_as_bytes.len(), 24);

    let wg_size = ((no_groups / 4) as u32, no_groups as u32, 1);
    return (wg_size, input_params_as_bytes);
}


pub fn get_spirv_configuration_u8(params: &crate::parameters::Parameters)
-> ((u32, u32, u32), Vec<u8>) {
    const WORKGROUP_SIZE: u16 = 16;
    // processing must be divided into full workgroups
    assert_eq!(params.img_size_px % WORKGROUP_SIZE, 0);
    let no_groups = params.img_size_px / WORKGROUP_SIZE;

    #[repr(C)]
    struct InputParameters {
        draw_bounds: [f32; 4], // -x, x, -y, y
        img_size_px: u32,      // in pixels
        max_iter: u32,         // max number of iterations to run
    }

    let input_parameters = InputParameters {
        draw_bounds: params.limits,
        img_size_px: params.img_size_px as u32,
        max_iter: params.max_iter as u32
    };

    let input_params_as_bytes = unsafe { std::slice::from_raw_parts(
        (&input_parameters as *const InputParameters) as *const u8,
        std::mem::size_of::<InputParameters>()
    )}.to_vec();

    assert_eq!(input_params_as_bytes.len(), 24);

    let wg_size = (no_groups as u32, no_groups as u32, 1);
    return (wg_size, input_params_as_bytes);
}


fn build_wgsl(
    params: &crate::parameters::Parameters,
    device: &wgpu::Device
) -> ((u32, u32, u32), wgpu::ShaderModule, Vec<u8>)
{
    const WORKGROUP_SIZE: u16 = 16;
    // processing must be divided into full workgroups
    assert_eq!(params.img_size_px % WORKGROUP_SIZE, 0);
    let no_groups = params.img_size_px / WORKGROUP_SIZE;
    // one shader invocation processes 4 consecutive pixels (due to stupid wgsl limitation of working only on u32)
    assert_eq!(no_groups % 4, 0);

    #[repr(C)]
    struct InputParameters {
        draw_bounds: [f32; 4], // -x, x, -y, y
        max_iter: u32,         // max number of iterations to ru
        img_size_px: u32       // in pixels
    }

    let input_parameters = InputParameters {
        draw_bounds: params.limits,
        max_iter: params.max_iter as u32,
        img_size_px: params.img_size_px as u32
    };

    let input_params_as_bytes = unsafe { std::slice::from_raw_parts(
        (&input_parameters as *const InputParameters) as *const u8,
        std::mem::size_of::<InputParameters>()
    )}.to_vec();

    assert_eq!(input_params_as_bytes.len(), 24);

    let wg_size = ((no_groups / 4) as u32, no_groups as u32, 1);
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(include_str!("mandelbrot.wgsl"))),
    });

    return (wg_size, shader_module, input_params_as_bytes);
}


fn build_spirv(
    params: &crate::parameters::Parameters,
    device: &wgpu::Device
) -> ((u32, u32, u32), wgpu::ShaderModule, Vec<u8>)
{
    let (wg_size, input_params_as_bytes) = get_spirv_configuration(&params);
    let shader_module = device.create_shader_module(wgpu::include_spirv!("mandelbrot-u32.rs.spv"));
    return (wg_size, shader_module, input_params_as_bytes);
}


pub fn get_wgpu_shader(
    params: &crate::parameters::Parameters,
    device: &wgpu::Device
) -> ((u32, u32, u32), wgpu::ShaderModule, Vec<u8>)
{
    match params.backend_type {
        BackendType::WgpuWgsl => build_wgsl(params, device),
        BackendType::WgpuSpirv => build_spirv(params, device),
        _ => panic!("Invalid backend for WGPU")
    }
}
