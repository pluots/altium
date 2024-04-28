
struct Globals {
    resolution: vec2<f32>,
    scroll_offset: vec2<f32>,
    scale: f32,
};

struct Primitive {
    color: vec4<f32>,
    translate: vec2<f32>,
    z_index: i32,
    width: f32,
    angle: f32,
    scale: f32,
    pad1: i32,
    pad2: i32,
};

struct Primitives {
    primitives: array<Primitive, 256>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<uniform> u_primitives: Primitives;

const INVERT_Y = vec2<f32>(1.0, -1.0);
const NM_PER_M: f32 = 1e9;
const M_PER_NM: f32 = 1e-9;


struct VertexOutput {
    @location(0) v_color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) a_position: vec2<f32>,
    @location(1) a_normal: vec2<f32>,
    @location(2) a_color: vec4<f32>,
    @location(3) a_stroke_width: f32,
    @builtin(instance_index) instance_idx: u32
) -> VertexOutput {
    // let local_pos = (a_position * )
    let local_pos = a_position + a_normal * a_stroke_width;
    let world_pos = (local_pos * M_PER_NM) + globals.scroll_offset;
    let transformed_pos = (world_pos / globals.scale) / (0.5 * globals.resolution) ;
    let position = vec4<f32>(transformed_pos.x, transformed_pos.y, 1.0, 1.0);
    // let color = vec4<f32>(0.0, 1.0, 1.0, 0.8);
    // var prim: Primitive = u_primitives.primitives[a_prim_id + instance_idx];

    // var invert_y = vec2<f32>(1.0, -1.0);

    // var rotation = mat2x2<f32>(
    //     vec2<f32>(cos(prim.angle), -sin(prim.angle)),
    //     vec2<f32>(sin(prim.angle), cos(prim.angle))
    // );

    // var local_pos = (a_position * prim.scale + a_normal * prim.width) * rotation;
    // var world_pos = local_pos - globals.scroll_offset + prim.translate;
    // var transformed_pos = world_pos * globals.zoom / (0.5 * globals.resolution) * invert_y;

    // var z = f32(prim.z_index) / 4096.0;
    // var position = vec4<f32>(transformed_pos.x, transformed_pos.y, z, 1.0);

    // return VertexOutput(vec4<f32>(0.0, 1.0, 1.0, 0.8), vec4<f32>(a_position[0], a_position[1], 1.0, 1.0));
    // return VertexOutput(a_color, vec4<f32>(a_position[0], a_position[1], 1.0, 1.0));
    // return VertexOutput(prim.color,c position);
    return VertexOutput(a_color, position);
}

struct FragOutput {
    @location(0) out_color: vec4<f32>,
};

@fragment
fn fs_main(
    @location(0) v_color: vec4<f32>,
) -> FragOutput {
    return FragOutput(v_color);
}
