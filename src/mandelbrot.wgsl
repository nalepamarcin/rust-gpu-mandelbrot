@group(0)
@binding(0)
var<storage, write> v_pixels: array<u32>;


// c - coordinates of complex point to check
// returns 0 if point inside set, otherwise number of iterations (up to max_iter) necessary to escape the set for sure
fn mandelbrot(c: vec2<f32>) -> u32 {
    // mandelbrot
    // z_n+1 = z_n * z_n + c
    var z = vec2(0.0f, 0.0f);
    for (var i = 0u; i < {{max_iter}}; i += 1u) {
        if length(z) > 2.0f {
            return i;
        }
        z = vec2(
            z[0]*z[0] - z[1]*z[1],
            z[0]*z[1] + z[1]*z[0]
        ) + c;
    }
    return 0u;
}


@compute
@workgroup_size({{wg_size}}, {{wg_size}})
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var img_x = global_id.x * 4u;
    var img_y = global_id.y;

    v_pixels[global_id.y * {{row_stride}} + global_id.x] =
        mandelbrot(vec2(mix({{img_min_x}}, {{img_max_x}}, f32(img_x)      / {{img_size}}),
                        mix({{img_min_y}}, {{img_max_y}}, f32(img_y)      / {{img_size}}))) |
        mandelbrot(vec2(mix({{img_min_x}}, {{img_max_x}}, f32(img_x + 1u) / {{img_size}}),
                        mix({{img_min_y}}, {{img_max_y}}, f32(img_y)      / {{img_size}}))) << 8u |
        mandelbrot(vec2(mix({{img_min_x}}, {{img_max_x}}, f32(img_x + 2u) / {{img_size}}),
                        mix({{img_min_y}}, {{img_max_y}}, f32(img_y)      / {{img_size}}))) << 16u |
        mandelbrot(vec2(mix({{img_min_x}}, {{img_max_x}}, f32(img_x + 3u) / {{img_size}}),
                        mix({{img_min_y}}, {{img_max_y}}, f32(img_y)      / {{img_size}}))) << 24u;
}
