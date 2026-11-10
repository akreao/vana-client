// FUN_10042d00's copy pass: the viewport squeezed into the 256x256 A8R8G8B8 target, flipped. An XYZRHW quad
// (0, 0)..(256, 256) with uv (-0.5 / W, (H - 0.5) / H)..((W - 0.5) / W, -0.5 / H): texel (i, j), its centre at
// (i, j) under D3D8's rules, samples u = (i W / 256 - 0.5) / W, v = (H - 0.5) / H - j / 256 (linear, MIRROR);
// colour TEXTURE, alpha TFACTOR 0x80 (TSS 0x103531c0, RS 0x1035322c: no blend, no z).
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

@vertex
fn vs(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((k << 1u) & 2u), f32(k & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let wh = vec2<f32>(textureDimensions(src));
    let ij = floor(p.xy);
    let uv = vec2<f32>((ij.x * wh.x / 256.0 - 0.5) / wh.x, (wh.y - 0.5) / wh.y - ij.y / 256.0);
    return vec4<f32>(textureSampleLevel(src, samp, uv, 0.0).rgb, 128.0 / 255.0);
}
