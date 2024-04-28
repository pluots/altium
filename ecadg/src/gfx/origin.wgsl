// Shader to draw the origin crosshairs

struct OriginUniformBuf {
    offset: vec2<f32>,
    hdims: vec2<f32>,
    vdims: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> origin_ctx: OriginUniformBuf;

const HALF_HEIGHT = 0.4;
const HALF_WIDTH = 0.05;

@vertex
fn vs_main(
    @builtin(vertex_index) idx: u32,
) -> @builtin(position) vec4<f32> {
    let offset = origin_ctx.offset;
    let hdims = origin_ctx.hdims;
    let vdims = origin_ctx.vdims;
    var pos: vec2<f32>;

    // 2 vertical triangles
    if idx == 0 {
        pos = vec2<f32>(offset.x - (vdims.x / 2), offset.y + (vdims.y / 2));
    } else if idx == 1 || idx == 3 {
        pos = vec2<f32>(offset.x + (vdims.x / 2), offset.y + (vdims.y / 2));
    } else if idx == 2 || idx == 5  {
        pos = vec2<f32>(offset.x - (vdims.x / 2), offset.y - (vdims.y / 2));
    } else if idx == 4 {
        pos = vec2<f32>(offset.x + (vdims.x / 2), offset.y - (vdims.y / 2));
    // 2 horizontal triangles
    } else if idx == 6 {
        pos = vec2<f32>(offset.x - (hdims.x / 2), offset.y + (hdims.y / 2));
    } else if idx == 7 || idx == 9 {
        pos = vec2<f32>(offset.x + (hdims.x / 2), offset.y + (hdims.y / 2));
    } else if idx == 8 || idx == 11{
        pos = vec2<f32>(offset.x - (hdims.x / 2), offset.y - (hdims.y / 2));
    } else if idx == 10 {
        pos = vec2<f32>(offset.x + (hdims.x / 2), offset.y - (hdims.y / 2));
    }

    return vec4<f32>(pos.x, pos.y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}
