use clap::Parser;


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum BackendType {
    OpenglSpirv,
    WgpuSpirv,
    WgpuWgsl
}


pub struct Parameters {
    pub backend_type: BackendType,
    pub img_size_px: u16,
    pub max_iter: u8,
    pub limits: [f32; 4]
}


#[derive(clap::Parser)]
pub struct Arguments {
    /// Type of the backend to run
    pub backend_type: BackendType,

    /// Final image size
    pub img_size_px: u16,

    /// Maximal number of iterations for pixel
    pub max_iter: u8,


    pub xmin: f32,
    pub xmax: f32,
    pub ymin: f32,
    pub ymax: f32
}

pub fn get_params() -> Parameters {
    let args = Arguments::parse();
    Parameters {
        backend_type: args.backend_type,
        img_size_px: args.img_size_px,
        max_iter: args.max_iter,
        limits: [
            args.xmin,
            args.xmax,
            args.ymin,
            args.ymax
        ]
    }
}
