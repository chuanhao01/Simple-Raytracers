struct Uniforms{
    vp_height: u32,
    vp_width: u32
}
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

alias TriangleVertices = array<vec2f, 6>;
var<private> vertices: TriangleVertices = TriangleVertices(
    vec2f(-1.0, -1.0),
    vec2f(1.0, 1.0),
    vec2f(-1.0, 1.0),
    vec2f(-1.0, -1.0),
    vec2f(1.0, -1.0),
    vec2f(1.0, 1.0),
);

@vertex fn display_vs(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4f{
    return vec4f(vertices[vid], 0.0, 1.0);
}

@fragment fn display_fs(@builtin(position) pos: vec4f) -> @location(0) vec4f{
    let color = pos.xy / vec2f(f32(uniforms.vp_width - 1u), f32(uniforms.vp_height - 1u));
    return vec4f(color, 0.0, 1.0);
}
