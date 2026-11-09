// Water reflections (render/reflect.rs, ours). World y grows downward: "up" is -y.

struct Params {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    // x: sky gradient on; y: the target holds gamma bytes; z: seconds; w: the frame copy is readable.
    info: vec4<f32>,
    clear: vec4<f32>,
    fog_color: vec4<f32>,
    // x, y: fog start, end; z, w: near and far planes.
    fog: vec4<f32>,
    sky_colors: array<vec4<f32>, 8>,
    sky_stops: array<vec4<f32>, 2>,
};

@group(0) @binding(0) var<uniform> u: Params;
@group(0) @binding(1) var depth: texture_depth_2d;
@group(0) @binding(2) var scene: texture_2d<f32>;

// The march: fixed steps (no loop runs longer on a busy screen), then a few halvings on a hit.
const STEPS: i32 = 20;
const REFINE: i32 = 5;
// How far a reflected ray is followed, in world units.
const RAY: f32 = 80.0;
// Fresnel for water (F0 = 0.02), lifted a little so calm water shows its reflection from above too, and capped so the
// water's own colour always shows through.
const F0: f32 = 0.02;
const LIFT: f32 = 0.12;
const MAX_WEIGHT: f32 = 0.7;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) world: vec3<f32>,
};

@vertex
fn vs(@location(0) p: vec3<f32>) -> VOut {
    var o: VOut;
    o.pos = u.view_proj * vec4<f32>(p, 1.0);
    o.world = p;
    return o;
}

fn linear(d: f32) -> f32 {
    return u.fog.z * u.fog.w / (u.fog.w - d * (u.fog.w - u.fog.z));
}

fn to_target(c: vec3<f32>) -> vec3<f32> {
    return select(pow(c, vec3<f32>(2.2)), c, u.info.y > 0.5);
}

fn sky_stop(k: u32) -> f32 {
    return u.sky_stops[k / 4u][k % 4u];
}

// As zone.wgsl's fs_sky (FUN_1003cd20): 8 rings by elevation, linear between them; gamma-space colours.
fn sky(dir: vec3<f32>) -> vec3<f32> {
    let e = asin(clamp(-dir.y, -1.0, 1.0)) / 1.5707963;
    var c = u.clear.rgb;
    if (u.info.x > 0.5) {
        c = u.sky_colors[0].rgb;
        for (var k = 0u; k < 7u; k = k + 1u) {
            let a = sky_stop(k);
            let b = sky_stop(k + 1u);
            if (e >= a && b > a) {
                c = mix(u.sky_colors[k].rgb, u.sky_colors[k + 1u].rgb, clamp((e - a) / (b - a), 0.0, 1.0));
            }
        }
    }
    return c;
}

// Ripples: a few travelling waves' slopes, their sum tilting the normal; faded out with distance before they alias.
fn ripple_normal(p: vec3<f32>, dist: f32) -> vec3<f32> {
    let t = u.info.z;
    var g = vec2<f32>(0.0);
    let waves = array<vec4<f32>, 4>(
        // direction xz, wavenumber, speed
        vec4<f32>(0.80, 0.60, 1.70, 1.10),
        vec4<f32>(-0.55, 0.83, 2.60, 1.45),
        vec4<f32>(0.20, -0.98, 4.10, 1.90),
        vec4<f32>(-0.93, -0.37, 6.30, 2.40),
    );
    for (var i = 0; i < 4; i = i + 1) {
        let w = waves[i];
        let ph = dot(w.xy, p.xz) * w.z + t * w.w * w.z;
        g = g + w.xy * (cos(ph) * w.z * (0.012 / w.z));
    }
    let k = clamp(1.0 - dist / 60.0, 0.0, 1.0);
    return normalize(vec3<f32>(g.x * k, -1.0, g.y * k));
}

fn project(q: vec3<f32>) -> vec4<f32> {
    return u.view_proj * vec4<f32>(q, 1.0);
}

fn to_uv(c: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(c.x / c.w * 0.5 + 0.5, 0.5 - c.y / c.w * 0.5);
}

fn scene_w(uv: vec2<f32>, size: vec2<f32>) -> f32 {
    let px = vec2<i32>(clamp(uv * size, vec2<f32>(0.0), size - 1.0));
    return linear(textureLoad(depth, px, 0));
}

// How the ray at `c` stands against the scene: > 0 behind the surface there (and close enough to count as a hit).
fn behind(c: vec4<f32>, size: vec2<f32>, thick: f32) -> f32 {
    let d = c.w - scene_w(to_uv(c), size);
    return select(-1.0, d, d > 0.0 && d < thick);
}

@fragment
fn fs(i: VOut) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(depth));
    // The depth test, against the scene (water draws no depth): hidden behind scenery or an actor.
    let px = vec2<i32>(i.pos.xy);
    if (i.pos.z > textureLoad(depth, px, 0) + 1e-6) {
        discard;
    }
    let to_p = i.world - u.eye.xyz;
    let dist = length(to_p);
    let v = to_p / dist;
    // Seen from under the surface: nothing.
    if (v.y <= 0.0) {
        discard;
    }
    let n = ripple_normal(i.world, dist);
    var r = reflect(v, n);
    r.y = min(r.y, -0.02);
    r = normalize(r);

    var colour = sky(r);
    var from_scene = 0.0;
    if (u.info.w > 0.5) {
        var prev = 0.0;
        var hit = -1.0;
        for (var s = 1; s <= STEPS; s = s + 1) {
            let f = f32(s) / f32(STEPS);
            let t = RAY * f * f;
            let c = project(i.world + r * t);
            if (c.w <= u.fog.z) {
                break;
            }
            let uv = to_uv(c);
            if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
                break;
            }
            if (behind(c, size, max(0.5, (t - prev) * 1.5)) > 0.0) {
                hit = t;
                break;
            }
            prev = t;
        }
        if (hit > 0.0) {
            var a = prev;
            var b = hit;
            for (var k = 0; k < REFINE; k = k + 1) {
                let m = 0.5 * (a + b);
                if (behind(project(i.world + r * m), size, max(0.5, (b - a) * 3.0)) > 0.0) {
                    b = m;
                } else {
                    a = m;
                }
            }
            let uv = to_uv(project(i.world + r * b));
            let texel = vec2<i32>(clamp(uv * size, vec2<f32>(0.0), size - 1.0));
            let hit_rgb = textureLoad(scene, texel, 0).rgb;
            // Faded toward the sky near the screen's edges and the ray's end.
            let edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
            from_scene = smoothstep(0.0, 0.08, edge) * (1.0 - smoothstep(0.7, 1.0, b / RAY));
            // The copy holds target values; the sky is gamma-space: bring the sky to the target first.
            return finish(mix(to_target(colour), hit_rgb, from_scene), n, v, dist);
        }
    }
    return finish(to_target(colour), n, v, dist);
}

// The Fresnel weight, fogged away with distance (the water there is already the fog's colour).
fn finish(rgb: vec3<f32>, n: vec3<f32>, v: vec3<f32>, dist: f32) -> vec4<f32> {
    let cos_i = clamp(dot(n, -v), 0.0, 1.0);
    let fresnel = F0 + (1.0 - F0) * pow(1.0 - cos_i, 5.0);
    let fog = clamp((u.fog.y - dist) / max(u.fog.y - u.fog.x, 1e-3), 0.0, 1.0);
    let w = clamp(LIFT + fresnel, 0.0, MAX_WEIGHT) * fog;
    return vec4<f32>(rgb, w);
}
