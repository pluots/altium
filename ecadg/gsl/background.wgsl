// @binding(0) @group(0) var<uniform> frame : u32;
// @vertex
// fn vtx_main(@builtin(vertex_index) vertex_index : u32) -> @builtin(position) vec4f {
//   var pos = array(
//     vec2( 0.0,  0.5),
//     vec2(-0.5, -0.5),
//     vec2( 0.5, -0.5)
//   );

//   return vec4f(pos[vertex_index], 0, 1);
// }

// @fragment
// fn frag_main() -> @location(0) vec4f {
//   return vec4(0, sin(f32(frame) / 128), 0, 1);
// }

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}
 
