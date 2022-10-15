# What is it?
Program for testing various methods of calculating the [Mandelbrot set](https://en.wikipedia.org/wiki/Mandelbrot_set) in Rust.

Implemented calculation methods (all inside `src/backends`):
* CPU
    * Classic double loop approach for each x, y pixel in the image
    * Iterator approach
    * `Rayon`'s parallel iterator
* GPU
    * `OpenGL` with `SPIR-V` compute shaders (both standard and `OpCapability Int8 & Int16` versions)
    * `WGPU` with standard `SPIR-V` and `WGSL` shaders

Shader sources available [here](https://github.com/nalepamarcin/rust-gpu-mandelbrot-shaders).

# How to build?
`cargo build --release`

# How to run?
See `./mandelbrot --help`:
```
Usage: mandelbrot <BACKEND_TYPE> <IMG_SIZE_PX> <MAX_ITER> <XMIN> <XMAX> <YMIN> <YMAX>

Arguments:
  <BACKEND_TYPE>  Type of the backend to run [possible values: cpu-loop, cpu-iter, cpu-par-iter, opengl-spirv, opengl-spirv-u8, wgpu-spirv, wgpu-wgsl]
  <IMG_SIZE_PX>   Final image size
  <MAX_ITER>      Maximal number of iterations for pixel
  <XMIN>          
  <XMAX>          
  <YMIN>          
  <YMAX>          

Options:
  -h, --help  Print help information
```

e.g.: `./mandelbrot -- wgpu-wgsl 8192 255 -2.2 0.8 -1.5 1.5` for nicely centered final image of size 8192x8192px and maximum (255) iteration capability.
