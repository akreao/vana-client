// Water reflections (render/reflect.rs, ours). World y grows downward: "up" is -y.

struct Params {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    forward: vec4<f32>,
    // x: sky gradient on; y: the target holds gamma bytes; z: seconds; w: the frame copy is readable.
    info: vec4<f32>,
    clear: vec4<f32>,
    fog_color: vec4<f32>,
    // x, y: fog start, end; z, w: near and far planes.
    fog: vec4<f32>,
    sky_colors: array<vec4<f32>, 8>,
    sky_stops: array<vec4<f32>, 2>,
    // The height each mirrored-characters layer belongs to (1e9 for an empty one).
    mirror_heights: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u: Params;
@group(0) @binding(1) var depth: texture_depth_2d;
@group(0) @binding(2) var scene: texture_2d<f32>;
// The mirrored characters (render/reflect.rs, ZoneRenderer::mirror_actors) at half size, transparent elsewhere: drawn
// through the camera mirrored in the plane under their feet, so on a reflecting surface they sit where their
// reflection belongs, front and all (the screen-space march only has the side the camera sees).
// One layer per plane: a surface reads only the layer of its own height.
@group(0) @binding(3) var layer: texture_2d_array<f32>;
@group(0) @binding(4) var layer_samp: sampler;

// A surface takes the layer whose height is nearest its own, within this much (world units).
const PLANE_MATCH: f32 = 0.3;

// The mirrored characters at a screen point (`size` the scene's) on a surface at height `y`, the water's ripples
// bending it a little; nothing when no layer belongs to that height.
fn mirrored(px: vec2<f32>, size: vec2<f32>, bend: vec2<f32>, y: f32) -> vec4<f32> {
    let d = abs(u.mirror_heights - vec4<f32>(y));
    var k = 0;
    for (var j = 1; j < 4; j++) {
        if (d[j] < d[k]) {
            k = j;
        }
    }
    if (d[k] > PLANE_MATCH) {
        return vec4<f32>(0.0);
    }
    return textureSampleLevel(layer, layer_samp, px / size + bend, k, 0.0);
}

// The march: at most this many screen steps (no loop runs longer on a busy screen), then a few halvings on a hit.
const STEPS: i32 = 32;
const REFINE: i32 = 5;
// How far a reflected ray is followed, in world units.
const RAY: f32 = 80.0;
// Fresnel for water (F0 = 0.02), lifted a little so calm water shows its reflection from above too, and capped so the
// water's own colour always shows through.
const F0: f32 = 0.02;
const LIFT: f32 = 0.12;
const MAX_WEIGHT: f32 = 0.7;
// The reflection fades in over this much water depth under the surface (world units), so it never shows where the
// collision's surface stands above the drawn water's edge (Bibiki Bay's shores sit about half a unit above its sea).
const SHORE: f32 = 1.0;
// Floors: a reflection fades out over this much of the ray (a glossy floor shows what stands on it, not the far end of
// the room), and a floor's own depth is matched within a small margin (its faces are the scenery's, placed on the CPU).
const FLOOR_RAY: f32 = 8.0;
// A hit is the ray passing behind the scene by less than this (plus 1% of the distance): thin things such as
// characters are about this deep, and a deeper allowance smears them down the floor where the ray passes far behind.
const THICK: f32 = 0.6;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) world: vec3<f32>,
    // A floor's strength; 0 for water.
    @location(1) @interpolate(flat) floor: f32,
};

@vertex
fn vs(@location(0) p: vec4<f32>) -> VOut {
    var o: VOut;
    o.pos = u.view_proj * vec4<f32>(p.xyz, 1.0);
    o.world = p.xyz;
    o.floor = p.w;
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

// Interleaved gradient noise (Jimenez 2014) in 0.25..1: a fixed per-pixel start for the march's steps.
fn jitter(px: vec2<f32>) -> f32 {
    return 0.25 + 0.75 * fract(52.9829189 * fract(dot(floor(px), vec2<f32>(0.06711056, 0.00583715))));
}

fn project(q: vec3<f32>) -> vec4<f32> {
    return u.view_proj * vec4<f32>(q, 1.0);
}

fn to_uv(c: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(c.x / c.w * 0.5 + 0.5, 0.5 - c.y / c.w * 0.5);
}

@fragment
fn fs(i: VOut) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(depth));
    // The depth test, against the scene (water draws no depth): hidden behind scenery or an actor.
    let px = vec2<i32>(i.pos.xy);
    let d = textureLoad(depth, px, 0);
    if (i.floor > 0.0) {
        return floor_reflection(i, d, size);
    }
    if (i.pos.z > d + 1e-6) {
        discard;
    }
    let to_p = i.world - u.eye.xyz;
    let dist = length(to_p);
    let v = to_p / dist;
    // Seen from under the surface: nothing.
    if (v.y <= 0.0) {
        discard;
    }
    // How far below the surface the scene behind it lies (y grows downward): the water's depth along this ray.
    let under = u.eye.xyz + v * (linear(d) / max(dot(v, u.forward.xyz), 1e-3));
    let shore = smoothstep(0.0, SHORE, under.y - i.world.y);
    if (shore <= 0.0) {
        discard;
    }
    let n = ripple_normal(i.world, dist);
    var r = reflect(v, n);
    r.y = min(r.y, -0.02);
    r = normalize(r);

    var colour = sky(r);
    var from_scene = 0.0;
    if (u.info.w > 0.5) {
        let b = march(i.world, r, RAY, size, jitter(i.pos.xy));
        if (b > 0.0) {
            let uv = to_uv(project(i.world + r * b));
            let texel = vec2<i32>(clamp(uv * size, vec2<f32>(0.0), size - 1.0));
            let hit_rgb = textureLoad(scene, texel, 0).rgb;
            // Faded toward the sky near the screen's edges and the ray's end.
            let edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
            from_scene = smoothstep(0.0, 0.08, edge) * (1.0 - smoothstep(0.7, 1.0, b / RAY));
            // The copy holds target values; the sky is gamma-space: bring the sky to the target first.
            return with_character(finish(mix(to_target(colour), hit_rgb, from_scene), n, v, dist), i, n, size, shore);
        }
    }
    return with_character(finish(to_target(colour), n, v, dist), i, n, size, shore);
}

// The water's colour and weight with a mirrored character over it: the character at least as strong as a calm
// reflection seen from above (the lift plus a little), bent by the ripples; then the shoreline fade.
fn with_character(c: vec4<f32>, i: VOut, n: vec3<f32>, size: vec2<f32>, shore: f32) -> vec4<f32> {
    let ch = mirrored(i.pos.xy, size, n.xz * 0.04, i.world.y);
    let w = mix(c.a, max(c.a, LIFT + 0.25), ch.a);
    return vec4<f32>(mix(c.rgb, ch.rgb, ch.a), w * shore);
}

// The first scene point the reflected ray from `p` along `r` meets on screen within `reach`, as its distance along the
// ray, or a negative distance on a miss.
//
// The march steps evenly across the screen, not along the ray: the ray's ends are projected, cut at the near plane,
// and walked in at most STEPS equal screen steps (every pixel or more), with depth interpolated as 1/w, as the
// rasteriser does. Steps fixed in world units land on different pixels as the camera moves, which made reflections
// warp and bubble on floors. Each pixel starts its steps at its own fraction `j` of a step (a fixed screen pattern),
// so neighbouring rays sample between each other's steps and thin things show as fine grain, not bands. A step counts
// as a hit when the ray has passed behind the scene by less than the step's own depth change (plus a margin, at most
// THICK); then REFINE halvings between the last two steps.
fn march(p: vec3<f32>, r: vec3<f32>, reach: f32, size: vec2<f32>, j: f32) -> f32 {
    let near = u.fog.z * 2.0;
    let c0 = project(p);
    var len = reach;
    var c1 = project(p + r * len);
    if (c1.w < near) {
        // Shorten the ray to end just past the near plane (w is linear along it).
        len = len * (c0.w - near) / max(c0.w - c1.w, 1e-6);
        c1 = project(p + r * len);
    }
    let s0 = to_uv(c0) * size;
    let s1 = to_uv(c1) * size;
    let n = clamp(ceil(length(s1 - s0)), 1.0, f32(STEPS));
    let iw0 = 1.0 / c0.w;
    let iw1 = 1.0 / c1.w;
    var prev = 0.0;
    var hit = -1.0;
    for (var k = 0; k < STEPS; k = k + 1) {
        let f = (f32(k) + j) / n;
        if (f > 1.0) {
            break;
        }
        let px = mix(s0, s1, f);
        if (any(px < vec2<f32>(0.0)) || any(px >= size)) {
            break;
        }
        let w = 1.0 / mix(iw0, iw1, f);
        let w_prev = 1.0 / mix(iw0, iw1, prev);
        let d = w - linear(textureLoad(depth, vec2<i32>(px), 0));
        if (d > 0.0 && d < min(abs(w - w_prev) + 0.1, THICK) + 0.01 * w) {
            hit = f;
            break;
        }
        prev = f;
    }
    if (hit < 0.0) {
        return -1.0;
    }
    var lo = prev;
    var hi = hit;
    for (var k = 0; k < REFINE; k = k + 1) {
        let m = 0.5 * (lo + hi);
        let w = 1.0 / mix(iw0, iw1, m);
        let px = clamp(mix(s0, s1, m), vec2<f32>(0.0), size - 1.0);
        if (w > linear(textureLoad(depth, vec2<i32>(px), 0))) {
            hi = m;
        } else {
            lo = m;
        }
    }
    // The screen fraction back to the distance along the ray (perspective-correct).
    return len * hi * iw1 / mix(iw0, iw1, hi);
}

// A polished floor: only what the march finds on screen, by the floor's strength, fading with the ray's length and at
// the screen's edges; a miss draws nothing (no sky under a roof).
fn floor_reflection(i: VOut, d: f32, size: vec2<f32>) -> vec4<f32> {
    let w_scene = linear(d);
    let w_floor = linear(i.pos.z);
    if (w_floor > w_scene + 0.05 + 0.003 * w_scene) {
        discard;
    }
    let to_p = i.world - u.eye.xyz;
    let dist = length(to_p);
    let v = to_p / dist;
    if (v.y <= 0.0) {
        discard;
    }
    let n = vec3<f32>(0.0, -1.0, 0.0);
    let r = normalize(reflect(v, n));
    // The march's hit, faded with the ray's length and at the screen's edges (none without the frame copy).
    var rgb = vec3<f32>(0.0);
    var found = 0.0;
    if (u.info.w > 0.5) {
        let b = march(i.world, r, FLOOR_RAY, size, jitter(i.pos.xy));
        if (b > 0.0) {
            let uv = to_uv(project(i.world + r * b));
            let texel = vec2<i32>(clamp(uv * size, vec2<f32>(0.0), size - 1.0));
            rgb = textureLoad(scene, texel, 0).rgb;
            let edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
            found = (1.0 - smoothstep(0.0, FLOOR_RAY, b)) * smoothstep(0.0, 0.08, edge);
        }
    }
    // A mirrored character over it.
    let ch = mirrored(i.pos.xy, size, vec2<f32>(0.0), i.world.y);
    rgb = mix(rgb, ch.rgb, ch.a);
    found = max(found, ch.a);
    if (found <= 0.0) {
        discard;
    }
    let cos_i = clamp(dot(n, -v), 0.0, 1.0);
    let fresnel = F0 + (1.0 - F0) * pow(1.0 - cos_i, 5.0);
    let fog = clamp((u.fog.y - dist) / max(u.fog.y - u.fog.x, 1e-3), 0.0, 1.0);
    let w = i.floor * mix(0.7, 1.0, fresnel) * found * fog;
    return vec4<f32>(rgb, w);
}

// The Fresnel weight, fogged away with distance (the water there is already the fog's colour).
fn finish(rgb: vec3<f32>, n: vec3<f32>, v: vec3<f32>, dist: f32) -> vec4<f32> {
    let cos_i = clamp(dot(n, -v), 0.0, 1.0);
    let fresnel = F0 + (1.0 - F0) * pow(1.0 - cos_i, 5.0);
    let fog = clamp((u.fog.y - dist) / max(u.fog.y - u.fog.x, 1e-3), 0.0, 1.0);
    let w = clamp(LIFT + fresnel, 0.0, MAX_WEIGHT) * fog;
    return vec4<f32>(rgb, w);
}
