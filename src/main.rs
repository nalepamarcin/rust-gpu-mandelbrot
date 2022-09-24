mod parameters;
mod wgpu;

use crate::parameters::Parameters;


pub fn init_logger(log_level: tracing::Level, log_spans: tracing_subscriber::fmt::format::FmtSpan) -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(log_level)
        .with_span_events(log_spans)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}


fn get_params() -> Parameters {
    Parameters {
        img_size_px: 8192,
        workgroup_size: 16,
        max_iter: 256,
        limits: [-2.2, 0.8, -1.5, 1.5]
    }
}


fn run(params: &Parameters, store_to_file: bool) {
    let start_time = std::time::Instant::now();
    let steps = pollster::block_on(wgpu::run_wgpu(&params));
    tracing::info!("Computing time: {:?}", start_time.elapsed());

    if !store_to_file
    { return; }

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


fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger(tracing::Level::INFO,
                tracing_subscriber::fmt::format::FmtSpan::ENTER |
                    tracing_subscriber::fmt::format::FmtSpan::CLOSE
    )?;
    Ok(run(&get_params(), true))
}
