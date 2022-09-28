
struct InputParameters {
    draw_bounds: vec4<f32>, // -x, x, -y, y
    max_iter: u32,          // max number of iterations to run
    img_size_px: u32        // in pixels
}

@group(0)
@binding(0)
var<uniform> input_parameters: InputParameters;

@group(0)
@binding(1)
var<storage, write> v_pixels: array<u32>;

var<private> img_size: f32;

// c - coordinates of complex point to check
// returns 0 if point inside set, otherwise number of iterations (up to max_iter) necessary to escape the set for sure
fn mandelbrot(c: vec2<f32>) -> u32 {
    // mandelbrot
    // z_n+1 = z_n * z_n + c
    var z = vec2(0.0f, 0.0f);
    for (var i = 0u; i < input_parameters.max_iter; i += 1u) {
        if (length(z) > 2.0f) {
            return i;
        }
        z = vec2(
            z[0]*z[0] - z[1]*z[1],
            z[0]*z[1] + z[1]*z[0]
        ) + c;
    }
    return 0u;
}


fn mandelmsaax4(c: vec2<f32>) -> u32 {
    // distribution with heavier middle
    var dpx = 1.0f / 4.0f / img_size;
    var sum =
        mandelbrot(c) +
        mandelbrot(c + vec2(-dpx, -dpx)) +
        mandelbrot(c + vec2( dpx, -dpx)) +
        mandelbrot(c + vec2( dpx,  dpx)) +
        mandelbrot(c + vec2(-dpx,  dpx));
    return sum / 5u;
}


fn mandelmsaax16(c: vec2<f32>) -> u32 {
    // uniform distribution of 16 points across pixel
    var dpx = 1.0f / 8.0f / img_size;
    var dpx2 = 3.0f / 8.0f / img_size;

    var sum =
        mandelbrot(c + vec2(-dpx2, -dpx2)) +
        mandelbrot(c + vec2(-dpx,  -dpx2)) +
        mandelbrot(c + vec2( dpx,  -dpx2)) +
        mandelbrot(c + vec2( dpx2, -dpx2)) +

        mandelbrot(c + vec2(-dpx2, -dpx)) +
        mandelbrot(c + vec2(-dpx,  -dpx)) +
        mandelbrot(c + vec2( dpx,  -dpx)) +
        mandelbrot(c + vec2( dpx2, -dpx)) +

        mandelbrot(c + vec2(-dpx2, dpx)) +
        mandelbrot(c + vec2(-dpx,  dpx)) +
        mandelbrot(c + vec2( dpx,  dpx)) +
        mandelbrot(c + vec2( dpx2, dpx)) +

        mandelbrot(c + vec2(-dpx2, dpx2)) +
        mandelbrot(c + vec2(-dpx,  dpx2)) +
        mandelbrot(c + vec2( dpx,  dpx2)) +
        mandelbrot(c + vec2( dpx2, dpx2));

    return sum / 16u;
}


fn mandelproc(c: vec2<f32>) -> u32 {
    return u32(clamp(
        f32(mandelmsaax16(c)) / f32(input_parameters.max_iter) * 1.5f * 255.0f,
        0.0f,
        255.0f
    ));
}


fn get_calc_x(pixel: u32) -> f32 {
    return mix(
        input_parameters.draw_bounds[0],
        input_parameters.draw_bounds[1],
        f32(pixel) / img_size
    );
}


@compute
@workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    img_size = f32(input_parameters.img_size_px);

    var img_x = global_id.x * 4u;
    var img_y_f = mix(
        input_parameters.draw_bounds[2],
        input_parameters.draw_bounds[3],
        f32(global_id.y) / img_size
    );

    v_pixels[global_id.y * (input_parameters.img_size_px / 4u) + global_id.x] =
        mandelproc(vec2(get_calc_x(img_x     ), img_y_f))        |
        mandelproc(vec2(get_calc_x(img_x + 1u), img_y_f)) << 8u  |
        mandelproc(vec2(get_calc_x(img_x + 2u), img_y_f)) << 16u |
        mandelproc(vec2(get_calc_x(img_x + 3u), img_y_f)) << 24u;
}
