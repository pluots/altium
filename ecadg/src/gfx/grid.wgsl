struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct GridUniformBuf {
    offset_mod: vec2<f32>,
    spacing_pct: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> grid_ctx: GridUniformBuf;

struct GridInstanceBuf {
    @location(2) spacing_mult: f32,
    @location(3) saturation: f32,
    @location(4) is_vert: u32,
    @location(5) _padding: u32,
}

// vertices alternate ends of the line
@vertex
fn vs_main(
    @builtin(vertex_index) idx: u32, 
    instance: GridInstanceBuf,
) -> VertexOut {
    let rem = idx % u32(2);
    // -1.0 or +1.0
    let bound = (f32(rem) * -2.0) + 1.0;
    // Multiply by the index to get offset. Subtract remainder from this for a floor-like function
    // (alternating vertices use the same multiplier)
    let offset_mult = f32(idx / u32(2));
    let sat = instance.saturation;    
    
    var out: VertexOut;
    out.color = vec4<f32>(sat, sat, sat, sat);
    
    if instance.is_vert == u32(0) {
        // horizontal
        let line_offset = -1.0 + (grid_ctx.spacing_pct.x * instance.spacing_mult * offset_mult)
            - grid_ctx.offset_mod.y;
        out.position = vec4<f32>(bound, line_offset , 0.0, 1.0);   
    } else {
        // vertical
        let line_offset = -1.0 + (grid_ctx.spacing_pct.y * instance.spacing_mult * offset_mult)
            + grid_ctx.offset_mod.x;
        out.position = vec4<f32>(line_offset, bound, 0.0, 1.0);   
    }

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}
