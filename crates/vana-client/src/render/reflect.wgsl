// Water reflections (render/reflect.rs, ours). World y grows downward: "up" is -y.

struct Params {
    view_proj: mat4x4<f32>,
    // The projection alone and the inverse view, for effect meshes (they come in view space).
    proj: mat4x4<f32>,
    view_inv: mat4x4<f32>,
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
    // The zone's water levels (reflect.rs Level): height, 1 when exact; counts.x of them. counts.y: 1 to show the
    // water found in flat colours instead (FFXI_REFLECT_DEBUG).
    levels: array<vec4<f32>, 16>,
    counts: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u: Params;
@group(0) @binding(1) var depth: texture_2d<f32>;
@group(0) @binding(2) var scene: texture_2d<f32>;
// The mirrored characters (render/reflect.rs, ZoneRenderer::mirror_actors) at half size, transparent elsewhere: drawn
// through the camera mirrored in the plane under their feet, so on a reflecting surface they sit where their
// reflection belongs, front and all (the screen-space march only has the side the camera sees).
// One layer per plane: a surface reads only the layer of its own height.
@group(0) @binding(3) var layer: texture_2d_array<f32>;
@group(0) @binding(4) var layer_samp: sampler;
// Effect meshes only: the effects' per-element output (render/effects.rs `outs`, OUT_WORDS each): element-to-view
// matrix (0..16), TFACTOR colour (16..20), UV offset (20, 21), visible (23).
@group(0) @binding(5) var<storage, read> douts: array<f32>;
@group(1) @binding(0) var fx_tex: texture_2d<f32>;
@group(1) @binding(1) var fx_samp: sampler;

const OUT_WORDS: u32 = 44u;
// An effect mesh that writes depth (op flag 0x1000): its water fades nothing at the shore (the depth is its own).
override FX_OPAQUE: bool = false;

// A surface takes the layer whose height is nearest its own, within this much (world units): a drawn sea may lie up to
// reflect.rs LEVEL_TOL off the collision height the characters are mirrored in.
const PLANE_MATCH: f32 = 0.75;
// Water levels (reflect.rs LEVEL_TOL, BED_ABOVE).
const LEVEL_TOL: f32 = 0.75;
const BED_ABOVE: f32 = 3.0;
// A face is flat when its normal is within about 18 degrees of straight up or down (reflect.rs FLAT).
const FLAT: f32 = 0.95;

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
// Fresnel for water (F0 = 0.02), capped so the water's own colour always shows through. No floor under it: looking
// straight down at water shows the water, whatever its colour.
const F0: f32 = 0.02;
const MAX_WEIGHT: f32 = 0.7;
// How far the reflection takes on the water's own hue (0: none, 1: the reflection times the hue at full brightness).
const TINT: f32 = 0.6;
// Water whose own alpha is at least this reflects fully (effect sheets are often drawn at TFACTOR alpha 0x80 or less).
const COVER: f32 = 0.15;
// A mirrored character shows at least this strongly over the water (times its alpha), so it reads from above too.
const CHARACTER: f32 = 0.3;
// See-through water: the reflection fades in over this much water depth under the surface (world units).
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
    // A floor's strength; 0 for see-through water, -1 for opaque water (reflect.rs SEE_THROUGH, OPAQUE).
    @location(1) @interpolate(flat) kind: f32,
    // The water's own colour and alpha (gamma).
    @location(2) water: vec4<f32>,
    // Effect meshes: the texture's coordinates, and 1 where the vertex lies on a water level.
    @location(3) uv: vec2<f32>,
    @location(4) on_level: f32,
};

@vertex
fn vs(@location(0) p: vec4<f32>, @location(1) c: vec4<f32>) -> VOut {
    var o: VOut;
    o.pos = u.view_proj * vec4<f32>(p.xyz, 1.0);
    o.world = p.xyz;
    o.kind = p.w;
    o.water = c;
    o.uv = vec2<f32>(0.0);
    // Scenery: placed on a level at load (2 tells it from an effect mesh's 1).
    o.on_level = 2.0;
    return o;
}

// Whether a surface at height `y` can be one of the zone's water levels (reflect.rs Level::holds).
fn on_level(y: f32) -> bool {
    for (var k = 0u; k < u32(u.counts.x); k = k + 1u) {
        let l = u.levels[k];
        if (l.y > 0.5) {
            if (abs(y - l.x) <= LEVEL_TOL) {
                return true;
            }
        } else if (y >= l.x - BED_ABOVE && y <= l.x + LEVEL_TOL) {
            return true;
        }
    }
    return false;
}

// An effect mesh drawn as the effects draw it (effects.wgsl vs_mesh: the element's model-view, TFACTOR and UV offset;
// instance = element).
@vertex
fn vs_fx(
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
    @builtin(instance_index) e: u32,
) -> VOut {
    var o: VOut;
    let b = e * OUT_WORDS;
    if (douts[b + 23u] < 0.5) {
        o.pos = vec4<f32>(2.0, 2.0, 2.0, 1.0);
        return o;
    }
    let mv = mat4x4<f32>(
        vec4<f32>(douts[b], douts[b + 1u], douts[b + 2u], douts[b + 3u]),
        vec4<f32>(douts[b + 4u], douts[b + 5u], douts[b + 6u], douts[b + 7u]),
        vec4<f32>(douts[b + 8u], douts[b + 9u], douts[b + 10u], douts[b + 11u]),
        vec4<f32>(douts[b + 12u], douts[b + 13u], douts[b + 14u], douts[b + 15u]));
    let vp = mv * vec4<f32>(pos, 1.0);
    o.pos = u.proj * vp;
    o.world = (u.view_inv * vp).xyz;
    o.kind = 0.0;
    let tf = vec4<f32>(douts[b + 16u], douts[b + 17u], douts[b + 18u], douts[b + 19u]);
    o.water = color * tf;
    o.uv = uv + vec2<f32>(douts[b + 20u], douts[b + 21u]);
    o.on_level = select(0.0, 1.0, on_level(o.world.y));
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
    let d = textureLoad(depth, vec2<i32>(i.pos.xy), 0).x;
    if (i.kind > 0.0) {
        return floor_reflection(i, d, size);
    }
    return water(i, i.water, d, size, i.kind == 0.0, 1e-6);
}

// An effect water mesh: its texture times its colour is the water's own; only where it is flat and on a level.
@fragment
fn fs_fx(i: VOut) -> @location(0) vec4<f32> {
    // Sampled and differentiated before anything is discarded (uniform control flow).
    let t = textureSample(fx_tex, fx_samp, i.uv);
    let n = cross(dpdx(i.world), dpdy(i.world));
    // The effects' depth bias (render/effects.rs ZBIAS, -15 constant and slope): an opaque mesh wrote its depth that
    // much nearer than it lies.
    let bias = 16.0 * max(abs(dpdx(i.pos.z)), abs(dpdy(i.pos.z))) + 2e-6;
    let size = vec2<f32>(textureDimensions(depth));
    let d = textureLoad(depth, vec2<i32>(i.pos.xy), 0).x;
    if (i.on_level < 0.999 || abs(n.y) < FLAT * length(n)) {
        discard;
    }
    return water(i, i.water * t, d, size, !FX_OPAQUE, bias);
}

// The reflection on a water pixel: `own` the water's colour and alpha, `d` the scene's depth there, `fade_shallow`
// for see-through water (the scene's depth is the bed under it), `tol` the depth test's allowance.
fn water(i: VOut, own: vec4<f32>, d: f32, size: vec2<f32>, fade_shallow: bool, tol: f32) -> vec4<f32> {
    // The depth test, against the scene with a small margin (the water draws no depth here, and opaque water's own
    // depth is in it already, placed by the GPU from the same data, or biased nearer for an effect mesh).
    let w_scene = linear(d);
    if (i.pos.z > d + tol && linear(i.pos.z) > w_scene + 0.05 + 0.003 * w_scene) {
        discard;
    }
    let to_p = i.world - u.eye.xyz;
    let dist = length(to_p);
    let v = to_p / dist;
    // Seen from under the surface: nothing.
    if (v.y <= 0.0) {
        discard;
    }
    var shore = 1.0;
    if (fade_shallow) {
        // How far below the surface the scene behind it lies (y grows downward): the water's depth along this ray.
        let under = u.eye.xyz + v * (w_scene / max(dot(v, u.forward.xyz), 1e-3));
        shore = smoothstep(0.0, SHORE, under.y - i.world.y);
        if (shore <= 0.0) {
            discard;
        }
    }
    let n = ripple_normal(i.world, dist);
    var r = reflect(v, n);
    r.y = min(r.y, -0.02);
    r = normalize(r);

    var colour = to_target(sky(r));
    if (u.info.w > 0.5) {
        let b = march(i.world, r, RAY, size, jitter(i.pos.xy));
        if (b > 0.0) {
            let uv = to_uv(project(i.world + r * b));
            let texel = vec2<i32>(clamp(uv * size, vec2<f32>(0.0), size - 1.0));
            let hit_rgb = textureLoad(scene, texel, 0).rgb;
            // Faded toward the sky near the screen's edges and the ray's end.
            let edge = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
            let from_scene = smoothstep(0.0, 0.08, edge) * (1.0 - smoothstep(0.7, 1.0, b / RAY));
            // The copy holds target values; the sky is gamma-space: brought to the target above.
            colour = mix(colour, hit_rgb, from_scene);
        }
    }
    // The water's hue at full brightness (its colour is gamma bytes, as the target's when the shaders encode it).
    let hue = to_target(own.rgb / max(max(own.r, own.g), max(own.b, 1e-3)));
    colour = colour * mix(vec3<f32>(1.0), hue, TINT);
    let cos_i = clamp(dot(n, -v), 0.0, 1.0);
    let fresnel = F0 + (1.0 - F0) * pow(1.0 - cos_i, 5.0);
    let fog = clamp((u.fog.y - dist) / max(u.fog.y - u.fog.x, 1e-3), 0.0, 1.0);
    // The water's alpha says how much of the bed shows through it, not how much it reflects: it only fades the
    // reflection out where the water itself fades out (a sheet's edges, holes in its texture).
    let alpha = smoothstep(0.0, COVER, own.a);
    let w = min(fresnel, MAX_WEIGHT) * alpha * fog;
    // A mirrored character over it, bent by the ripples.
    let ch = mirrored(i.pos.xy, size, n.xz * 0.04, i.world.y);
    let wc = mix(w, max(w, CHARACTER * alpha * fog), ch.a);
    // FFXI_REFLECT_DEBUG: the water found, scenery magenta and effect meshes cyan, at half strength.
    if (u.counts.y > 0.5) {
        return vec4<f32>(select(vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(1.0, 0.0, 1.0), i.on_level > 1.5), 0.5);
    }
    return vec4<f32>(mix(colour, ch.rgb, ch.a), wc * shore);
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
        let d = w - linear(textureLoad(depth, vec2<i32>(px), 0).x);
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
        if (w > linear(textureLoad(depth, vec2<i32>(px), 0).x)) {
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
    let w = i.kind * mix(0.7, 1.0, fresnel) * found * fog;
    return vec4<f32>(rgb, w);
}
