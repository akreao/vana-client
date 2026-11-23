//! World sun shadows (ours, not retail; Settings > Scene > World Shadows, config `world_shadows`): the zone's scenery
//! drawn into a depth map from the zone's sun, around the camera's target, and read back by the scenery and actor
//! shaders (zone.wgsl `world_lit`), which darken only the sun's direct term: FFXI paints its shade into the vertex
//! colours, which the ambient term keeps, so nothing is darkened twice.
//!
//! The maps are cached: the scenery barely moves, so a cascade is drawn again only when the camera's target has moved
//! a part of its side away, the sun has turned by a small step, or what is shown changed (a sub-area, a hidden link);
//! a voyage's moving world is drawn each frame. Low is one map; High a sharp near cascade inside a wide one. Off, at
//! night's change-over and indoors (a fixed light) the shaders' strength is 0 and every term is retail's.

use super::Gpu;
use glam::{Mat4, Vec2, Vec3, Vec4};

/// Each cascade's map size in texels.
pub const SIZE: u32 = 2048;
/// How far the light's eye sits back from the target, and the depth range it covers (Depth32Float: precise enough).
const BACK: f32 = 400.0;
const DEPTH: f32 = 800.0;
/// The lowest the sun's elevation is taken as, in degrees: a sun near the horizon would lay every shadow across the
/// whole map (and the cascades) and catch every bump of the coarse terrain.
const MIN_ELEVATION: f32 = 25.0;
/// A cascade is drawn again once the target has moved this part of its half side from the map's centre.
const RECENTRE: f32 = 0.25;
/// ... or the sun has turned more than this (radians; 0.2 degrees, about 2 s of game time at noon).
const TURN: f32 = 0.2 * std::f32::consts::PI / 180.0;
/// How far the camera's eye may move from where the maps were drawn before they are drawn again (an orbit about the
/// target: the LOD bands and the camera list's reach go by the eye).
const EYE_STEP: f32 = 4.0;
/// The most a shadow takes off the sun's direct term (a little of it is kept: the sky's light, which the game's
/// ambient term does not carry).
pub const STRENGTH: f32 = 0.85;
/// Each normal offset against acne, in map texels.
const OFFSET_TEXELS: f32 = 1.5;

/// The cascades' half sides in world units at a level (config `world_shadows`: 0 off, 1 Low, 2 High).
pub fn halves(level: u32) -> &'static [f32] {
    match level {
        0 => &[],
        1 => &[40.0],
        _ => &[20.0, 80.0],
    }
}

/// The direction the zone's light travels for the maps, its elevation held at MIN_ELEVATION or above. The world's y
/// points down, so a light shining down travels toward +y; one travelling up (below the horizon) casts none.
pub fn direction(dir: Vec3) -> Option<Vec3> {
    let d = dir.try_normalize()?;
    if d.y <= 0.0 {
        return None;
    }
    let h = (d.x * d.x + d.z * d.z).sqrt();
    if h < 1e-4 {
        return Some(Vec3::Y);
    }
    let min = MIN_ELEVATION.to_radians().tan();
    if d.y / h < min {
        return Vec3::new(d.x, min * h, d.z).try_normalize();
    }
    Some(d)
}

/// The light's view for a direction (`d` from direction()).
fn light_view(d: Vec3) -> Mat4 {
    let up = if d.x.abs() < 1e-3 && d.z.abs() < 1e-3 { Vec3::Z } else { Vec3::Y };
    glam::camera::rh::view::look_at_mat4(Vec3::ZERO, d, up)
}

/// The cascade's view-projection (orthographic, depth 0..1) for light direction `d`, centred on `target` snapped to
/// whole texels of a `half`-side map, and that centre in light space (x, y). Snapping keeps the edges from shimmering
/// when the map is drawn again elsewhere.
pub fn matrix(d: Vec3, target: Vec3, half: f32) -> (Mat4, Vec2) {
    let view = light_view(d);
    let texel = 2.0 * half / SIZE as f32;
    let t = view.transform_point3(target);
    let c = Vec2::new((t.x / texel).round() * texel, (t.y / texel).round() * texel);
    let (l, r, b, tp) = (c.x - half, c.x + half, c.y - half, c.y + half);
    let (zn, zf) = (-(t.z + BACK), -(t.z + BACK) + DEPTH);
    let proj = Mat4::from_cols(
        Vec4::new(2.0 / (r - l), 0.0, 0.0, 0.0),
        Vec4::new(0.0, 2.0 / (tp - b), 0.0, 0.0),
        Vec4::new(0.0, 0.0, -1.0 / (zf - zn), 0.0),
        Vec4::new(-(r + l) / (r - l), -(tp + b) / (tp - b), -zn / (zf - zn), 1.0),
    );
    (proj * view, c)
}

/// A cascade's cached state: what its map was drawn for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drawn {
    pub dir: Vec3,
    pub centre: Vec2,
    pub vp: Mat4,
}

/// Which cascades to draw again this frame (their indices) for light direction `d` (direction()'s) and the camera's
/// `target`, given what each was drawn for; `all` (a voyage, a change in what is shown) draws every one. Each one
/// drawn takes its new state.
pub fn plan(drawn: &mut [Option<Drawn>], halves: &[f32], d: Vec3, target: Vec3, all: bool) -> Vec<usize> {
    let turned = drawn.iter().flatten().any(|c| c.dir.angle_between(d) > TURN);
    let mut out = Vec::new();
    for (k, (slot, &half)) in drawn.iter_mut().zip(halves).enumerate() {
        let stale = match slot {
            None => true,
            Some(c) => {
                let t = light_view(c.dir).transform_point3(target);
                all || turned || (Vec2::new(t.x, t.y) - c.centre).abs().max_element() > RECENTRE * half
            }
        };
        if stale {
            let (vp, centre) = matrix(d, target, half);
            *slot = Some(Drawn { dir: d, centre, vp });
            out.push(k);
        }
    }
    out
}

/// The shaders' strength for a light of this colour at this twilight level (sunshadow::twilight): none for a dim
/// light (night, dusk), as the character shadows.
pub fn strength(diffuse: Vec3, level: f32) -> f32 {
    let lum = diffuse.dot(Vec3::new(0.299, 0.587, 0.114));
    STRENGTH * ((lum - 0.05) / 0.25).clamp(0.0, 1.0) * level.clamp(0.0, 1.0)
}

/// zone.wgsl's WorldShadowU.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Params {
    vp0: [[f32; 4]; 4],
    vp1: [[f32; 4]; 4],
    info: [f32; 4],
    misc: [f32; 4],
}

impl Params {
    /// The cascades' matrices and half sides at `strength` (0 = off), cast by a light travelling along `dir`.
    pub fn new(cascades: &[(Mat4, f32)], strength: f32, dir: Vec3) -> Self {
        let dir = dir.normalize_or_zero();
        let vp = |k: usize| cascades.get(k).map_or(Mat4::IDENTITY, |c| c.0).to_cols_array_2d();
        let off = |k: usize| cascades.get(k).map_or(0.0, |c| OFFSET_TEXELS * 2.0 * c.1 / SIZE as f32);
        let on = !cascades.is_empty() && strength > 0.0;
        Self {
            vp0: vp(0),
            vp1: vp(1),
            info: [if on { strength } else { 0.0 }, cascades.len() as f32, off(0), off(1)],
            misc: [1.0 / SIZE as f32, dir.x, dir.y, dir.z],
        }
    }
}

/// The globals' world shadow bindings (zone.wgsl bindings 6..9) as they stand when the shadows are off: a zero
/// uniform and a 1 x 1 map for both cascades. The scenery's own depth pass binds these too (its map can't be bound
/// while drawn into).
pub struct Off {
    pub uniform: wgpu::Buffer,
    pub map: wgpu::TextureView,
    pub cmp: wgpu::Sampler,
}

impl Off {
    pub fn new(gpu: &Gpu) -> Self {
        let device = &gpu.device;
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("world shadow (off)"),
            size: std::mem::size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        // Mapped at creation: zeroed.
        uniform.unmap();
        Self { uniform, map: map(device, 1, "world shadow (off)"), cmp: compare_sampler(device) }
    }
}

/// The globals layout's world shadow entries (bindings 6..9).
pub fn layout_entries() -> [wgpu::BindGroupLayoutEntry; 4] {
    let depth = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Depth,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    [
        wgpu::BindGroupLayoutEntry {
            binding: 6,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Params>() as u64),
            },
            count: None,
        },
        depth(7),
        depth(8),
        wgpu::BindGroupLayoutEntry {
            binding: 9,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
            count: None,
        },
    ]
}

fn map(device: &wgpu::Device, size: u32, label: &str) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: super::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

/// Lit where the point is no deeper than the map (LessEqual), filtered over the four texels.
fn compare_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("world shadow compare"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        compare: Some(wgpu::CompareFunction::LessEqual),
        ..Default::default()
    })
}

/// The world shadows at a level: the cascades' maps, the shaders' uniform, the depth pass's matrices (group 3 of the
/// silhouette layout, 256 bytes a cascade) and the cache (behind a lock: the renderer draws through `&self`).
pub struct WorldShadow {
    pub level: u32,
    pub maps: Vec<wgpu::TextureView>,
    pub uniform: wgpu::Buffer,
    pub matrix: wgpu::Buffer,
    pub matrix_group: wgpu::BindGroup,
    state: std::sync::Mutex<State>,
}

#[derive(Default)]
struct State {
    drawn: Vec<Option<Drawn>>,
    /// What was shown when the maps were drawn (ZoneRenderer::shown_key); a change draws them all again.
    shown: u64,
    /// Whether door parts were moving at the last frame: their final pose is drawn once more after they stop.
    moving: bool,
    /// The camera's eye when the maps were drawn: the casters' LOD bands and the camera list's reach are measured
    /// from it, so an orbit far enough draws them again.
    eye: Vec3,
    /// The last frame's strength and how many cascades it drew (the diagnostics and the benchmark).
    strength: f32,
    redrawn: usize,
}

impl WorldShadow {
    pub fn new(gpu: &Gpu, sil_layout: &wgpu::BindGroupLayout, level: u32) -> Self {
        let device = &gpu.device;
        let n = halves(level).len();
        let maps =
            (0..n).map(|k| map(device, SIZE, if k == 0 { "world shadow near" } else { "world shadow far" })).collect();
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("world shadow"),
            size: std::mem::size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        uniform.unmap();
        let matrix = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("world shadow matrices"),
            size: 256 * n.max(1) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let matrix_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("world shadow matrices"),
            layout: sil_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &matrix,
                    offset: 0,
                    size: wgpu::BufferSize::new(64),
                }),
            }],
        });
        let state = State { drawn: vec![None; n], ..Default::default() };
        Self { level, maps, uniform, matrix, matrix_group, state: std::sync::Mutex::new(state) }
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The second cascade's map, or the first's where there is one only (the shader never reads it then).
    pub fn far(&self) -> &wgpu::TextureView {
        self.maps.last().unwrap_or(&self.maps[0])
    }

    /// The cascades to draw this frame (index, matrix) for light direction `d` around `target`: plan()'s, all of
    /// them when `all`, what is shown (`shown`) changed, door parts are `moving` or just stopped, or the `eye` has
    /// moved EYE_STEP. Their matrices are written for the depth pass.
    #[allow(clippy::too_many_arguments)]
    pub fn stale(
        &self,
        gpu: &Gpu,
        d: Vec3,
        target: Vec3,
        eye: Vec3,
        all: bool,
        shown: u64,
        moving: bool,
    ) -> Vec<(usize, Mat4)> {
        let mut s = self.state();
        let all = all || s.shown != shown || moving || s.moving || s.eye.distance(eye) > EYE_STEP;
        s.shown = shown;
        s.moving = moving;
        let out: Vec<(usize, Mat4)> = plan(&mut s.drawn, halves(self.level), d, target, all)
            .into_iter()
            .filter_map(|k| Some((k, s.drawn[k]?.vp)))
            .collect();
        for (k, vp) in &out {
            super::Put::put(&gpu.queue, &self.matrix, 256 * *k as u64, bytemuck::cast_slice(&vp.to_cols_array()));
        }
        if !out.is_empty() {
            s.eye = eye;
        }
        s.redrawn = out.len();
        out
    }

    /// Whether every cascade has been drawn (a map the shaders can keep reading while a sub-area uploads).
    pub fn drawn(&self) -> bool {
        self.state().drawn.iter().all(Option::is_some)
    }

    /// Writes the shaders' uniform for this frame at `strength` (0 = off), the light travelling along `dir`.
    pub fn set(&self, gpu: &Gpu, strength: f32, dir: Vec3) {
        let mut s = self.state();
        let cascades: Vec<(Mat4, f32)> =
            s.drawn.iter().zip(halves(self.level)).map_while(|(d, &h)| d.map(|d| (d.vp, h))).collect();
        let strength = if cascades.len() == self.maps.len() { strength } else { 0.0 };
        if strength <= 0.0 {
            s.redrawn = 0;
        }
        s.strength = strength;
        super::Put::put(&gpu.queue, &self.uniform, 0, bytemuck::bytes_of(&Params::new(&cascades, strength, dir)));
    }

    /// This frame's strength and the cascades it drew.
    pub fn stats(&self) -> (f32, usize) {
        let s = self.state();
        (s.strength, s.redrawn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_holds_a_low_sun() {
        // Below the horizon (travelling up): none.
        assert!(direction(Vec3::new(1.0, -0.2, 0.0)).is_none());
        assert!(direction(Vec3::ZERO).is_none());
        assert_eq!(direction(Vec3::new(0.0, 3.0, 0.0)), Some(Vec3::Y));
        let low = direction(Vec3::new(1.0, 0.05, 0.0)).unwrap();
        assert!((low.y.atan2(low.x).to_degrees() - MIN_ELEVATION).abs() < 0.01, "{low}");
        let high = Vec3::new(0.3, 1.0, 0.1).normalize();
        assert!((direction(high).unwrap() - high).length() < 1e-6);
    }

    #[test]
    fn matrix_frames_the_target() {
        let d = direction(Vec3::new(0.4, 1.0, -0.2)).unwrap();
        let target = Vec3::new(-120.0, 8.0, 300.0);
        let (vp, _) = matrix(d, target, 40.0);
        let c = vp.project_point3(target);
        assert!(c.x.abs() < 0.01 && c.y.abs() < 0.01 && c.z > 0.0 && c.z < 1.0, "{c}");
        // A point toward the light is nearer.
        assert!(vp.project_point3(target - d * 10.0).z < c.z);
        // 40 units across the light lands at the map's edge.
        let side = light_view(d).inverse().transform_vector3(Vec3::X);
        assert!((vp.project_point3(target + side * 40.0).x - 1.0).abs() < 0.01);
    }

    #[test]
    fn plan_draws_only_when_stale() {
        let h = halves(2);
        let mut drawn = vec![None; h.len()];
        let d = direction(Vec3::new(0.3, 1.0, 0.2)).unwrap();
        let t = Vec3::new(10.0, 0.0, 10.0);
        assert_eq!(plan(&mut drawn, h, d, t, false), vec![0, 1]);
        assert!(plan(&mut drawn, h, d, t, false).is_empty());
        // A small step: nothing; past a quarter of the near half side (5 units), the near one only.
        assert!(plan(&mut drawn, h, d, t + Vec3::new(2.0, 0.0, 0.0), false).is_empty());
        assert_eq!(plan(&mut drawn, h, d, t + Vec3::new(9.0, 0.0, 9.0), false), vec![0]);
        // Past the far one's quarter (20 units) both.
        assert_eq!(plan(&mut drawn, h, d, t + Vec3::new(40.0, 0.0, 40.0), false), vec![0, 1]);
        let t = t + Vec3::new(40.0, 0.0, 40.0);
        // The sun turning a little (0.1 degree): kept; past 0.2 degrees: both.
        let turn = |deg: f32| Mat4::from_rotation_z(deg.to_radians()).transform_vector3(d);
        assert!(plan(&mut drawn, h, turn(0.1), t, false).is_empty());
        assert_eq!(plan(&mut drawn, h, turn(0.5), t, false), vec![0, 1]);
        assert_eq!(plan(&mut drawn, h, turn(0.5), t, true), vec![0, 1]);
    }

    /// The cache: drawn once, kept while nothing changes, all drawn again when the eye orbits past EYE_STEP, parts
    /// move (and once more as they stop) or what is shown changes.
    #[test]
    fn stale_redraws_on_orbit_parts_and_shown() {
        let Ok(gpu) = Gpu::headless() else { return };
        let sil = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            }],
        });
        let ws = WorldShadow::new(&gpu, &sil, 2);
        let d = direction(Vec3::new(0.3, 1.0, 0.2)).unwrap();
        let t = Vec3::ZERO;
        let eye = Vec3::new(0.0, -3.0, 6.0);
        let n = |eye: Vec3, shown: u64, moving: bool| ws.stale(&gpu, d, t, eye, false, shown, moving).len();
        assert!(!ws.drawn());
        assert_eq!(n(eye, 0, false), 2);
        assert!(ws.drawn());
        assert_eq!(n(eye, 0, false), 0);
        assert_eq!(n(eye + Vec3::X * 2.0, 0, false), 0);
        assert_eq!(n(Vec3::new(6.0, -3.0, 0.0), 0, false), 2);
        let eye = Vec3::new(6.0, -3.0, 0.0);
        assert_eq!((n(eye, 0, true), n(eye, 0, true), n(eye, 0, false), n(eye, 0, false)), (2, 2, 2, 0));
        assert_eq!((n(eye, 7, false), n(eye, 7, false)), (2, 0));
    }

    #[test]
    fn strength_off_when_dim_or_at_the_change() {
        assert_eq!(strength(Vec3::ONE, 0.0), 0.0);
        assert_eq!(strength(Vec3::ZERO, 1.0), 0.0);
        assert!((strength(Vec3::ONE, 1.0) - STRENGTH).abs() < 1e-6);
        let p = Params::new(&[], 1.0, Vec3::Y);
        assert_eq!(p.info[0], 0.0);
        let p = Params::new(&[(Mat4::IDENTITY, 40.0)], 0.5, Vec3::new(0.0, 2.0, 0.0));
        assert_eq!(p.misc[1..], [0.0, 1.0, 0.0]);
        assert_eq!((p.info[0], p.info[1]), (0.5, 1.0));
        assert!((p.info[2] - 1.5 * 80.0 / SIZE as f32).abs() < 1e-6);
    }
}
