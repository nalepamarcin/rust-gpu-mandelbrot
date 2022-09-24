@group(0)
@binding(0)
var<storage, write> v_pixels: array<u32>;


@compute
@workgroup_size({{wg_size}}, {{wg_size}})
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>
) {
    var max_iter: u32 = {{max_iter}};
    var row_stride: u32 = {{row_stride}}; // workgroup_size * group_count = image_size

    // mandelbrot
    // p_n+1 = p_n * p_n + c
    var c = vec2(
        mix({{img_min_x}}, {{img_max_x}}, f32(global_id.x) / {{img_size}}),
        mix({{img_min_y}}, {{img_max_y}}, f32(global_id.y) / {{img_size}})
    );

    var p = vec2(0.0f, 0.0f);
    for (var i = 0u; i < max_iter; i += 1u) {
        if length(p) > 2.0f {
            v_pixels[global_id.y * row_stride + global_id.x] = i;
            break;
        }
        p = vec2(
            p[0]*p[0] - p[1]*p[1],
            p[0]*p[1] + p[1]*p[0]
        ) + c;
    }
}
