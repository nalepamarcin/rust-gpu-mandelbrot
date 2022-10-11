mod backends;
mod parameters;
mod shaders;

use crate::parameters::BackendType;


pub fn init_logger(log_level: tracing::Level, log_spans: tracing_subscriber::fmt::format::FmtSpan) -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(log_level)
        .with_span_events(log_spans)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}


fn process_col(img_size: u32, data: &[u8]) -> image::RgbImage {
    let colormap = scarlet::colormap::ListedColorMap::inferno().vals;
    let colormap: Vec<[u8; 3]> = colormap.iter().map(|[r, g, b]| {
        [(*r as f32 * 255.0) as u8,
         (*g as f32 * 255.0) as u8,
         (*b as f32 * 255.0) as u8]
    }).collect();

    let color_image_data: Vec<u8> = data.iter().flat_map(|v| { colormap[*v as usize] }).collect();
    image::RgbImage::from_raw(img_size, img_size, color_image_data).unwrap()
}


fn get_data_from_backend(params: &parameters::Parameters) -> Vec<u8> {
    match params.backend_type {
        BackendType::WgpuSpirv | BackendType::WgpuWgsl =>
            pollster::block_on(backends::wgpu::run_wgpu(&params))
    }
}


fn run(params: &parameters::Parameters, store_to_file: bool) {
    let start_time = std::time::Instant::now();
    let data = get_data_from_backend(&params);
    tracing::info!("Computing time: {:?}", start_time.elapsed());

    if !store_to_file
    { return; }

    let start_time = std::time::Instant::now();
    let img_size = params.img_size_px as u32;
    let imgbuf = process_col(img_size, data.as_slice());
    tracing::info!("Processing time: {:?}", start_time.elapsed());

    let start_time = std::time::Instant::now();
    imgbuf.save("fractal.png").unwrap();
    tracing::info!("Saving time: {:?}", start_time.elapsed());
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger(tracing::Level::INFO,
                tracing_subscriber::fmt::format::FmtSpan::ENTER |
                    tracing_subscriber::fmt::format::FmtSpan::CLOSE
    )?;
    let params = parameters::get_params();
    Ok(run(&params, true))
}
