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
    var my_cx: f32 = mix({{img_min_x}}, {{img_max_x}}, f32(global_id.x) / {{img_size}});
    var my_cy: f32 = mix({{img_min_y}}, {{img_max_y}}, f32(global_id.y) / {{img_size}});

    var r = my_cx * my_cx + my_cy * my_cy;
    v_pixels[global_id.y * row_stride + global_id.x] = u32(r < 1.0f);
}
