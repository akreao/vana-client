//! Wet ground (ours, not retail; Settings > Scene > Wet Ground, config `wet_ground`): while it rains outdoors, the
//! ground looks wet. Retail's rain (the weather's own falling-drop effects) is left as it is; this changes the surfaces
//! it falls on. Wet ground darkens (wet stone and earth absorb more light), takes a shine of the sky's colour that grows
//! toward grazing angles (Fresnel), gathers puddles in patches and shows rain ripples close to the camera. The wetness
//! comes on over half a minute once rain starts and dries off over a minute and a half after it stops
//! (SceneFx::step_rain), so a passing shower leaves the ground wet a while.
//!
//! One full-size pass from depth, nothing else: the world position of each pixel and of its four neighbours give the
//! surface's normal. Level ground gets the shine, the puddles and the ripples; slopes and walls only darken, and less;
//! the sky and what faces down are left alone. The pass writes a multiply and an add in one blend (scene x alpha +
//! colour), so it reads no scene copy. The puddle patches and the ripples come from a hash of the world position (fixed
//! to the ground as the camera moves); each pixel checks the 3 x 3 ripple cells around it, so the cost is the same
//! everywhere and nothing grows with the scene.
//!
//! Limits of a screen-space pass: ground under a roof or an overhang gets wet too (the pass can't see the sky from it),
//! and characters darken a little with the ground.

use super::Gpu;
use super::scenefx::{Frame, pipeline};

/// The weather tags it rains in (dat::weather::WEATHER_TAGS): rain, squall, thunder, thunderstorm.
pub const RAIN_TAGS: [&str; 4] = ["rain", "squl", "thdr", "bolt"];

/// Whether weather `number` (0x057's, or an event's) wets the ground.
pub fn rains(number: u16) -> bool {
    RAIN_TAGS.contains(&crate::dat::weather::weather_tag(number))
}

/// Seconds for the ground to soak once rain starts, and to dry off once it stops.
pub const SOAK_SECS: f32 = 30.0;
pub const DRY_SECS: f32 = 90.0;

/// The wetness `wet` (0 dry .. 1 soaked) eased `dt` seconds toward `raining`.
pub fn ease_wet(wet: f32, raining: bool, dt: f32) -> f32 {
    if raining { (wet + dt / SOAK_SECS).min(1.0) } else { (wet - dt / DRY_SECS).max(0.0) }
}

/// (least the wet ground keeps of its colour, puddle cover 0..1, ripple brightness, sheen brightness) for Subtle (1)
/// and Strong (2).
pub fn params(strength: u32) -> [f32; 4] {
    match strength {
        2 => [0.6, 0.45, 0.55, 0.7],
        _ => [0.75, 0.25, 0.35, 0.45],
    }
}

/// How far from the eye the ground is wet (faded out over the last third), and the ripples (units).
const WET_FAR: f32 = 180.0;
const RIPPLE_FAR: f32 = 30.0;

const WGSL: &str = r#"
struct U {
    inv_view_proj: mat4x4<f32>,
    // xyz: the eye; w: the clock (seconds).
    eye: vec4<f32>,
    // x: wetness 0..1; y: the least the ground keeps of its colour; z: puddle cover; w: ripple brightness.
    p: vec4<f32>,
    // rgb: the sky's colour (the zone's fog this frame, gamma space); a: the sheen's brightness.
    sky: vec4<f32>,
    // x: how far the ground is wet; y: how far the ripples go.
    far: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var depth: texture_2d<f32>;

// The world position at pixel `px` (clamped to the target), and its depth in w.
fn world_at(px: vec2<i32>, dims: vec2<i32>) -> vec4<f32> {
    let q = clamp(px, vec2<i32>(0), dims - vec2<i32>(1));
    let d = textureLoad(depth, q, 0).x;
    let uv = (vec2<f32>(q) + 0.5) / vec2<f32>(dims);
    let h = u.inv_view_proj * vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
    return vec4<f32>(h.xyz / h.w, d);
}

// Of the steps to the two neighbours along `o`, the one along the same surface (the shorter): an edge's other side
// is far behind or in front. At the target's border, the neighbour inside it.
fn step_along(p: vec3<f32>, px: vec2<i32>, o: vec2<i32>, dims: vec2<i32>) -> vec3<f32> {
    let ia = px + o;
    let ib = px - o;
    let in_a = all(ia >= vec2<i32>(0)) && all(ia < dims);
    let in_b = all(ib >= vec2<i32>(0)) && all(ib < dims);
    if (!in_a) { return p - world_at(ib, dims).xyz; }
    if (!in_b) { return world_at(ia, dims).xyz - p; }
    let da = world_at(ia, dims).xyz - p;
    let db = p - world_at(ib, dims).xyz;
    return select(db, da, dot(da, da) < dot(db, db));
}

fn hash(c: vec2<i32>) -> u32 {
    var h = bitcast<u32>(c.x) * 0x8da6b343u ^ bitcast<u32>(c.y) * 0xd8163841u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    return h ^ (h >> 15u);
}

fn unit(h: u32) -> f32 {
    return f32(h & 0xffffu) / 65535.0;
}

// Smooth value noise 0..1 over the ground (x, z).
fn noise(g: vec2<f32>) -> f32 {
    let c = vec2<i32>(floor(g));
    let f = fract(g);
    let s = f * f * (3.0 - 2.0 * f);
    let a = unit(hash(c));
    let b = unit(hash(c + vec2<i32>(1, 0)));
    let d = unit(hash(c + vec2<i32>(0, 1)));
    let e = unit(hash(c + vec2<i32>(1, 1)));
    return mix(mix(a, b, s.x), mix(d, e, s.x), s.y);
}

// The rain ripples at ground point g (x, z): each cell of the grid has one drop landing at a time, at a place and
// moment of its own, whose ring widens and fades over its period. The 3 x 3 cells around g cover every ring that can
// reach it.
const CELL: f32 = 0.9;
fn ripples(g: vec2<f32>, t: f32) -> f32 {
    let c = vec2<i32>(floor(g / CELL));
    var sum = 0.0;
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let k = c + vec2<i32>(i, j);
            let h = hash(k);
            let period = 0.8 + 0.6 * unit(h >> 16u);
            let n = t / period + unit(h);
            let age = fract(n);
            // A new place for each drop (the period's count in the hash).
            let h2 = hash(k * 7 + vec2<i32>(i32(floor(n)) & 1023, 31));
            let at = (vec2<f32>(k) + 0.15 + 0.7 * vec2<f32>(unit(h2), unit(h2 >> 16u))) * CELL;
            let r = distance(g, at);
            let radius = age * 0.55;
            let x = (r - radius) / 0.045;
            sum += exp(-x * x) * (1.0 - age) * (1.0 - age);
        }
    }
    return min(sum, 1.0);
}

@fragment fn fs_wet(i: VOut) -> @location(0) vec4<f32> {
    let dims = vec2<i32>(textureDimensions(depth));
    let px = vec2<i32>(i.pos.xy);
    let c = world_at(px, dims);
    if (c.w >= 1.0) { discard; }
    let p = c.xyz;
    let dist = distance(p, u.eye.xyz);
    if (dist >= u.far.x) { discard; }
    let dx = step_along(p, px, vec2<i32>(1, 0), dims);
    let dy = step_along(p, px, vec2<i32>(0, 1), dims);
    var n = cross(dx, dy);
    let len = length(n);
    if (len < 1e-8) { discard; }
    n = n / len;
    let v = (u.eye.xyz - p) / max(dist, 1e-4);
    if (dot(n, v) < 0.0) { n = -n; }
    // World y points down: ground faces -y. Rain wets everything it falls on, so slopes and walls darken too, less
    // (a soft ramp, so the faces of coarse terrain don't show as patches); what faces down stays dry.
    let up = smoothstep(0.75, 0.95, -n.y);
    let side = smoothstep(-0.1, 0.3, -n.y);
    if (side <= 0.0) { discard; }
    let wet = u.p.x * mix(0.5 * side, 1.0, smoothstep(0.2, 0.8, -n.y)) * (1.0 - smoothstep(u.far.x * 0.65, u.far.x, dist));
    // Puddles in patches a few units across, where the noise is high, on level ground only; their edges soft.
    let g = p.xz;
    let cover = u.p.z;
    let puddle = smoothstep(1.0 - cover, 1.0 - cover + 0.12, noise(g * 0.3) * 0.7 + noise(g * 1.1) * 0.3) * wet * up;
    // Darker where wet, darker still (but mirror-like) in the puddles.
    let keep = mix(1.0, u.p.y, wet) * mix(1.0, 0.8, puddle);
    // The sky's shine off the film of water, which lies level whatever the face under it: little looking down, much at
    // grazing angles. A film on wet level ground, a mirror in the puddles.
    let f = 0.04 + 0.96 * pow(1.0 - clamp(-v.y, 0.0, 1.0), 5.0);
    let level = wet * smoothstep(0.5, 0.9, -n.y);
    var add = u.sky.rgb * u.sky.a * f * (0.35 * level + puddle);
    // The ripples, near the camera only, on wet level ground and brightest in puddles.
    if (dist < u.far.y) {
        let near = 1.0 - smoothstep(u.far.y * 0.5, u.far.y, dist);
        let ring = ripples(g, u.eye.w);
        add += u.sky.rgb * (u.p.w * ring * near * (0.3 * level * up + puddle));
    }
    return vec4<f32>(add, keep);
}
"#;

pub struct Rain {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    /// The group, by the depth view it reads.
    group: super::BindCache<wgpu::TextureView>,
    strength: u32,
}

/// The pass's uniform for `frame` at `strength` with the ground `wet` (0..1) at `clock` seconds, or None when there is
/// nothing to draw: dry ground, indoors (no sun or moon in the frame) or no zone fog to take the sky's colour from.
pub fn uniform(frame: &Frame, strength: u32, wet: f32, clock: f32) -> Option<[f32; 32]> {
    if wet <= 0.0 || frame.sun.is_none() {
        return None;
    }
    let fog = frame.fog?;
    let [keep, cover, ripple, sheen] = params(strength);
    let mut u = [0.0f32; 32];
    u[..16].copy_from_slice(&frame.view_proj.inverse().to_cols_array());
    u[16..20].copy_from_slice(&[frame.eye.x, frame.eye.y, frame.eye.z, clock]);
    u[20..24].copy_from_slice(&[wet.min(1.0), keep, cover, ripple]);
    u[24..28].copy_from_slice(&[fog.color[0], fog.color[1], fog.color[2], sheen]);
    u[28..32].copy_from_slice(&[WET_FAR, RIPPLE_FAR, 0.0, 0.0]);
    Some(u)
}

impl Rain {
    /// `scene`: the scene target's format.
    pub fn new(gpu: &Gpu, scene: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wet ground"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("wet ground"),
            source: wgpu::ShaderSource::Wgsl(format!("{}{WGSL}", super::scenefx::VERTEX).into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("wet ground"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        // The scene x the pass's alpha (darker), plus its colour (the shine); the scene's alpha kept.
        let wet = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::SrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: super::scenefx::ADD.alpha,
        };
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wet ground"),
            size: 32 * 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline: pipeline(device, "wet ground", &pl, &module, "fs_wet", scene, Some(wet)),
            layout,
            uniform,
            group: Default::default(),
            strength: 0,
        }
    }

    pub fn set_strength(&mut self, strength: u32) {
        self.strength = strength;
    }

    /// The wet ground over `scene` by `depth` (the scene's), when the uniform says there is any.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        scene: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        frame: &Frame,
        wet: f32,
        clock: f32,
    ) {
        let Some(u) = uniform(frame, self.strength, wet, clock) else { return };
        super::Put::put(&gpu.queue, &self.uniform, 0, bytemuck::cast_slice(&u));
        let g = self.group.get(depth.clone(), || {
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("wet ground"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(depth) },
                ],
            })
        });
        super::scenefx::pass(encoder, "wet ground", scene, true, &self.pipeline, &g);
    }
}

#[cfg(test)]
mod tests {
    use super::super::scenefx::test_util::{filled, read};
    use super::super::scenefx::{FogFrame, Frame, SunFrame};
    use super::*;
    use glam::Vec3;

    const FOG: FogFrame = FogFrame { color: [0.8, 0.8, 0.8], start: 20.0, end: 200.0 };
    const SUN: SunFrame = SunFrame { dir: [0.0, 1.0, 0.0], color: [1.0; 3], moon: false };

    #[test]
    fn rain_weathers() {
        let tag = |t: &str| crate::dat::weather::WEATHER_TAGS.iter().position(|x| *x == t).unwrap() as u16;
        assert!(["rain", "squl", "thdr", "bolt"].iter().all(|t| rains(tag(t))));
        assert!(["fine", "suny", "clod", "snow", "bliz", "fogd"].iter().all(|t| !rains(tag(t))));
    }

    #[test]
    fn soaks_then_dries_slower() {
        let mut w = 0.0;
        for _ in 0..15 {
            w = ease_wet(w, true, 1.0);
        }
        assert!((w - 0.5).abs() < 1e-5, "{w}");
        w = ease_wet(w, true, 100.0);
        assert_eq!(w, 1.0);
        w = ease_wet(w, false, 45.0);
        assert!((w - 0.5).abs() < 1e-5, "{w}");
        assert_eq!(ease_wet(w, false, 1000.0), 0.0);
    }

    fn cam(eye: Vec3, target: Vec3) -> crate::render::Camera {
        crate::render::Camera { jitter: glam::Vec2::ZERO, eye, target, up: Vec3::NEG_Y, fov_y: 0.8 }
    }

    fn frame(c: &crate::render::Camera) -> Frame {
        Frame { view_proj: c.view_proj(1.0), eye: c.eye, target: c.target, fog: Some(FOG), sun: Some(SUN) }
    }

    #[test]
    fn nothing_dry_indoors_or_unlit() {
        let c = cam(Vec3::new(0.0, -2.0, 0.0), Vec3::new(0.0, 0.0, 10.0));
        let f = frame(&c);
        assert!(uniform(&f, 1, 0.0, 0.0).is_none(), "dry");
        assert!(uniform(&Frame { sun: None, ..f }, 1, 1.0, 0.0).is_none(), "indoors");
        assert!(uniform(&Frame { fog: None, ..f }, 1, 1.0, 0.0).is_none(), "no zone lighting");
        let u = uniform(&f, 2, 1.0, 3.0).unwrap();
        assert_eq!((u[19], u[20], u[21]), (3.0, 1.0, 0.6));
    }

    /// A grey scene whose depth is a flat floor (y = 0, y down) seen from 2 units above, or a wall straight ahead,
    /// after the pass.
    fn after(gpu: &Gpu, rain: &Rain, wall: bool, wet: f32) -> Vec<[u8; 4]> {
        let n = 32u32;
        let c = if wall {
            cam(Vec3::new(0.0, -2.0, 0.0), Vec3::new(0.0, -2.0, 10.0))
        } else {
            cam(Vec3::new(0.0, -2.0, 0.0), Vec3::new(0.0, 0.0, 6.0))
        };
        let f = frame(&c);
        let (tex, view) = filled(gpu, [n, n], |_, _| [128, 128, 128, 255]);
        // The depth of each pixel's ray hitting the floor (y = 0) or the wall (z = 8), written as R32Float.
        let vp = c.view_proj(1.0);
        let inv = vp.inverse();
        let data: Vec<f32> = (0..n * n)
            .map(|k| {
                let (x, y) = (k % n, k / n);
                let ndc =
                    glam::Vec2::new((x as f32 + 0.5) / n as f32 * 2.0 - 1.0, 1.0 - (y as f32 + 0.5) / n as f32 * 2.0);
                let a = inv.project_point3(ndc.extend(0.0));
                let b = inv.project_point3(ndc.extend(1.0));
                let dir = (b - a).normalize();
                let t = if wall { (8.0 - a.z) / dir.z } else { -a.y / dir.y };
                if t <= 0.0 || !t.is_finite() || t > 150.0 { 1.0 } else { vp.project_point3(a + dir * t).z }
            })
            .collect();
        let dt = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        gpu.queue.write_texture(
            dt.as_image_copy(),
            bytemuck::cast_slice(&data),
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(n * 4), rows_per_image: Some(n) },
            wgpu::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
        );
        let depth = dt.create_view(&Default::default());
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        rain.draw(gpu, &mut enc, &view, &depth, &f, wet, 1.0);
        gpu.queue.submit([enc.finish()]);
        read(gpu, &tex)
    }

    #[test]
    fn floor_wet_wall_less_sky_untouched() {
        let Ok(gpu) = Gpu::headless() else { return };
        let mut rain = Rain::new(&gpu, wgpu::TextureFormat::Rgba8Unorm);
        rain.set_strength(2);
        let n = 32usize;
        let floor = after(&gpu, &rain, false, 1.0);
        // The bottom row looks down at the floor near the eye: darker than the scene, and the alpha kept.
        let near = floor[(n - 1) * n + n / 2];
        assert!(near[0] < 128 && near[3] == 255, "{near:?}");
        // The top row is sky (no hit): untouched.
        assert_eq!(floor[n / 2], [128, 128, 128, 255]);
        // Grazing floor further off shines more than the floor under the camera.
        let mid = floor[(n / 2 + 2) * n + n / 2];
        assert!(mid[0] > near[0], "{mid:?} {near:?}");
        // A wall darkens less than the floor and gets no shine.
        let wall = after(&gpu, &rain, true, 1.0);
        let w = wall[(n / 2) * n + n / 2];
        assert!(w[0] < 128 && w[0] > near[0] && w[3] == 255, "{w:?} {near:?}");
        // Dry ground: no pass at all.
        assert!(after(&gpu, &rain, false, 0.0).iter().all(|p| *p == [128, 128, 128, 255]));
    }
}
