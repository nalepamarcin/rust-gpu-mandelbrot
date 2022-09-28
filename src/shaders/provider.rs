
pub enum ShaderType {
    WGSL
}


pub fn get_shader(
    params: &crate::parameters::Parameters,
    device: &wgpu::Device
) -> ((u32, u32, u32), wgpu::ShaderModule)
{
    match params.shader_type {
        ShaderType::WGSL => {
            // processing must be divided into full workgroups
            assert_eq!(params.img_size_px % params.workgroup_size, 0);
            let no_groups = params.img_size_px / params.workgroup_size;
            // one shader invocation processes 4 consecutive pixels (due to stupid wgsl limitation of working only on u32)
            assert_eq!(no_groups % 4, 0);

            let mut shader_src = include_str!("mandelbrot.wgsl").to_string();
            shader_src = shader_src.replace("{{wg_size}}", &*format!("{}u", params.workgroup_size));
            shader_src = shader_src.replace("{{max_iter}}", &*format!("{}u", params.max_iter));
            shader_src = shader_src.replace("{{row_stride}}", &*format!("{}u", params.img_size_px / 4));
            shader_src = shader_src.replace("{{img_size}}", &*format!("{}.0f", params.img_size_px));

            shader_src = shader_src.replace("{{img_min_x}}", &*format!("{}f", params.limits[0]));
            shader_src = shader_src.replace("{{img_max_x}}", &*format!("{}f", params.limits[1]));
            shader_src = shader_src.replace("{{img_min_y}}", &*format!("{}f", params.limits[2]));
            shader_src = shader_src.replace("{{img_max_y}}", &*format!("{}f", params.limits[3]));

            let wg_size = ((no_groups / 4) as u32, no_groups as u32, 1);
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(shader_src)),
            });

            return (wg_size, shader_module);
        }
    }
}
