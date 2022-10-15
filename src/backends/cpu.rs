use rayon::prelude::*;


#[derive(Clone, Copy)]
struct Vec2 {
    pub x: f32,
    pub y: f32
}


impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    pub fn length_sq(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
}


impl std::ops::Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}


fn mix(a: f32, b: f32, v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    a * (1.0 - v) + b * v
}


fn mandelbrot(c: Vec2, max_iter: u8) -> u8 {
    // mandelbrot
    // z_n+1 = z_n * z_n + c
    let mut z = Vec2::new(0.0, 0.0);
    for i in 0..max_iter {
        // if we got outside circle of radius 2 we will diverge to infinity
        if z.length_sq() > 4.0 {
            return i;
        }
        z = Vec2::new(
            z.x*z.x - z.y*z.y + c.x,
            z.x*z.y + z.y*z.x + c.y
        );
    }
    return 0;
}


fn mandelmsaax16(c: Vec2, max_iter: u8, img_size: f32) -> u32 {
    // uniform distribution of 16 points across pixel
    let dpx = 1.0 / 8.0 / img_size;
    let dpx2 = 3.0 / 8.0 / img_size;

    let sum =
        mandelbrot(c + Vec2::new(-dpx2, -dpx2), max_iter) as u32 +
        mandelbrot(c + Vec2::new(-dpx,  -dpx2), max_iter) as u32 +
        mandelbrot(c + Vec2::new( dpx,  -dpx2), max_iter) as u32 +
        mandelbrot(c + Vec2::new( dpx2, -dpx2), max_iter) as u32 +

        mandelbrot(c + Vec2::new(-dpx2, -dpx), max_iter) as u32 +
        mandelbrot(c + Vec2::new(-dpx,  -dpx), max_iter) as u32 +
        mandelbrot(c + Vec2::new( dpx,  -dpx), max_iter) as u32 +
        mandelbrot(c + Vec2::new( dpx2, -dpx), max_iter) as u32 +

        mandelbrot(c + Vec2::new(-dpx2, dpx), max_iter) as u32 +
        mandelbrot(c + Vec2::new(-dpx,  dpx), max_iter) as u32 +
        mandelbrot(c + Vec2::new( dpx,  dpx), max_iter) as u32 +
        mandelbrot(c + Vec2::new( dpx2, dpx), max_iter) as u32 +

        mandelbrot(c + Vec2::new(-dpx2, dpx2), max_iter) as u32 +
        mandelbrot(c + Vec2::new(-dpx,  dpx2), max_iter) as u32 +
        mandelbrot(c + Vec2::new( dpx,  dpx2), max_iter) as u32 +
        mandelbrot(c + Vec2::new( dpx2, dpx2), max_iter) as u32;

    return sum / 16;
}


fn mandelproc(c: Vec2, max_iter: u8, img_size: f32) -> u8 {
    (mandelmsaax16(c, max_iter, img_size) as f32 / max_iter as f32 * 1.5 * 255.0).clamp(
        0.0,
        255.0
    ) as u8
}


fn mandelbrot_for_xy(x: u16, y: u16, input_parameters: &crate::parameters::Parameters) -> u8 {
    let img_size_f = input_parameters.img_size_px as f32;

    let img_x = mix(
        input_parameters.limits[0],
        input_parameters.limits[1],
        x as f32 / img_size_f
    );
    let img_y = mix(
        input_parameters.limits[2],
        input_parameters.limits[3],
        y as f32 / img_size_f
    );

    mandelproc(Vec2::new(img_x, img_y), input_parameters.max_iter, img_size_f)
}


pub fn run_cpu_loops(params: &crate::parameters::Parameters) -> crate::result::ComputeResult {
    let start_time = std::time::Instant::now();
    let mut data = vec![0u8; params.img_size_px as usize * params.img_size_px as usize];
    let init_time = start_time.elapsed();
    for y in 0..params.img_size_px {
        let row_idx = y as usize * params.img_size_px as usize;
        for x in 0..params.img_size_px {
            data[row_idx + x as usize] = mandelbrot_for_xy(x, y, &params);
        }
    }

    crate::result::ComputeResult {
        data,
        initialization_time: init_time,
        computation_time: start_time.elapsed() - init_time,
        data_fetch_time: std::time::Duration::ZERO
    }
}


pub fn run_cpu_iter(params: &crate::parameters::Parameters) -> crate::result::ComputeResult {
    let start_time = std::time::Instant::now();
    let img_size_px = params.img_size_px as usize;
    let data = (0..(img_size_px * img_size_px)).map(|i|{
        let (x, y) = (i % img_size_px, i / img_size_px);
        mandelbrot_for_xy(x as u16, y as u16, &params)
    }).collect();

    crate::result::ComputeResult {
        data,
        initialization_time: std::time::Duration::ZERO,
        computation_time: start_time.elapsed(),
        data_fetch_time: std::time::Duration::ZERO
    }
}


pub fn run_cpu_par_iter(params: &crate::parameters::Parameters) -> crate::result::ComputeResult {
    let start_time = std::time::Instant::now();
    let img_size_px = params.img_size_px as usize;
    let data = (0..(img_size_px * img_size_px)).into_par_iter().map(|i|{
        let (x, y) = (i % img_size_px, i / img_size_px);
        mandelbrot_for_xy(x as u16, y as u16, &params)
    }).collect();

    crate::result::ComputeResult {
        data,
        initialization_time: std::time::Duration::ZERO,
        computation_time: start_time.elapsed(),
        data_fetch_time: std::time::Duration::ZERO
    }
}
