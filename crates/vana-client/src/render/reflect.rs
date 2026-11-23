//! Water reflections (ours, not retail; Settings > Graphics > Reflections, config `reflections`).
//!
//! Where: on the water as it is drawn. The collision's type-9 faces (the ones fishing and wading use) give the zone's
//! water levels (`water_levels`); a drawn surface on one of them is water, found two ways, with no list per zone:
//! - Scenery: a placed model whose upward faces lie on an exact level and make up nearly all of it (`is_water`),
//!   checked once at load. Its faces are copied out in world space with its colour (texture average x vertex colour).
//! - Effect meshes (most open water: Buburimu's `umi1`): a flat mesh of a zone effect, its vertices on a level. These
//!   move and are placed on the GPU, so the effects draw (render/effects.rs) draws their reflection right after each
//!   one, through the element's own matrix, texture and colour (`FxWater`, reflect.wgsl `vs_fx` / `fs_fx`).
//!
//! How: after the actors, one copy of the frame is taken. Each water pixel marches its reflected ray through the
//! scene's depth a fixed number of steps; a hit takes the copy's colour there, a miss (or a ray leaving the screen)
//! takes retail's sky gradient in that direction. The reflection is tinted toward the water's own hue and weighted
//! by Fresnel times the water's own alpha and the fog, so water of any colour keeps its colour. See-through water
//! fades out where it is shallow (the scene's depth lies little below the surface). Ripples bend the surface normal,
//! worked out in the shader.
//!
//! Floors (the option's "Water and Floors"): polished floors chosen by hand, per zone and texture (`floor_textures`).
//! Their upward-facing triangles are copied out of the scenery at load, in world space, and drawn in the same pass
//! with the strength listed for them. A floor reflects only what the march finds on screen, fading with the ray's
//! length (a glossy look, and no sky under a roof); it has no ripples and no shoreline fade.
//!
//! Cost: one scene-sized copy in a zone with water, a draw of the scenery water in view, and one extra draw per
//! effect water mesh drawn; nothing in a zone without water. The scenery faces go in tiles at load (a sort, linear
//! apart from it) and each frame tests only the tiles' boxes.

use super::Gpu;
use crate::dat::collision::{Collision, water_of};
use glam::{Mat4, Vec3, Vec4};

/// Tiles of water faces tested against the view each frame, this many world units square.
const TILE: f32 = 32.0;
/// Water farther than this from the eye takes no reflection (the fog covers it).
const REACH: f32 = 300.0;
/// A face is flat when its normal is within about 18 degrees of straight up or down.
const FLAT: f32 = 0.95;

/// Polished floors by zone: (texture name past its 8-byte prefix, reflection strength). Picked by looking at each
/// zone's large upward-facing textures; cobbles, grass and carpets are left out.
pub fn floor_textures(zone: u32) -> &'static [(&'static str, f32)] {
    match zone {
        // Ru'Lude Gardens: the patterned marble, then the smooth stone.
        243 => &[("e_yuka", 0.35), ("yuka02", 0.35), ("yuka01", 0.25), ("kaka", 0.2)],
        // Lower Jeuno: the smooth grey floor.
        245 => &[("yuka1", 0.2)],
        // Port Jeuno: the diamond-tiled stone.
        246 => &[("p", 0.25)],
        // Aht Urhgan Whitegate: the tiled floor.
        50 => &[("yu", 0.25)],
        _ => &[],
    }
}

/// A floor triangle's world normal must point this far up (y grows downward).
const FLOOR_UP: f32 = 0.9;

/// Whether a placed triangle is floor: every corner's normal, through the placement's matrix, points up.
pub fn faces_up(m: &Mat4, normals: [[f32; 3]; 3]) -> bool {
    normals.iter().all(|n| m.transform_vector3(Vec3::from(*n)).normalize_or_zero().y < -FLOOR_UP)
}

/// A drawn water surface lies within this of its collision water height (Bibiki Bay's sea sits about half a unit
/// below the height its collision gives).
pub const LEVEL_TOL: f32 = 0.75;
/// A type-9 face without a water height may be the bed under the drawn surface: the surface may stand this far above
/// it (y grows downward).
pub const BED_ABOVE: f32 = 3.0;
/// Heights closer than this are one level.
const LEVEL_MERGE: f32 = 0.25;
/// At most this many levels reach the shader (the rest are dropped, those with the fewest faces first).
pub const MAX_LEVELS: usize = 16;
/// A scenery face is on a level when its centre lies within the level's faces' box grown by this much.
const LEVEL_MARGIN: f32 = 4.0;
/// A placed model is water when at least this share of its area is flat faces on a level...
const WATER_SHARE: f32 = 0.9;
/// ... and that is at least this much area (square units): a stray step or a lid is not.
const WATER_AREA: f32 = 4.0;

/// One water height of the zone, as its collision gives it: from the faces' object (`exact`, the height fishing and
/// wading use) or the height of flat type-9 faces without one (the surface, or a bed under it). `lo`/`hi`: the x/z
/// box of its faces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Level {
    pub y: f32,
    pub exact: bool,
    pub lo: [f32; 2],
    pub hi: [f32; 2],
    pub faces: u32,
}

impl Level {
    /// Whether a surface at height `y` can be this water.
    pub fn holds(&self, y: f32) -> bool {
        if self.exact { (y - self.y).abs() <= LEVEL_TOL } else { y >= self.y - BED_ABOVE && y <= self.y + LEVEL_TOL }
    }

    /// Whether (x, z) lies in its box grown by `margin`.
    pub fn covers(&self, x: f32, z: f32, margin: f32) -> bool {
        x >= self.lo[0] - margin && x <= self.hi[0] + margin && z >= self.lo[1] - margin && z <= self.hi[1] + margin
    }
}

/// The zone's water levels (type-9 faces), most faces first, at most MAX_LEVELS. Built once at load, linear apart
/// from a sort.
pub fn water_levels(c: &Collision) -> Vec<Level> {
    let mut raw: Vec<(f32, bool, [Vec3; 3])> = Vec::new();
    for (i, t) in c.tris.iter().enumerate() {
        if c.kinds.get(i) != Some(&9) {
            continue;
        }
        let Some(o) = c.object.get(i).and_then(|&o| c.objects.get(o as usize)) else { continue };
        let h = water_of(o.flags);
        if h != 0.0 {
            // A bed wholly above its own surface (a bank) is no water.
            if t.iter().all(|v| v.y < h - 0.05) {
                continue;
            }
            raw.push((h, true, *t));
        } else {
            let n = (t[1] - t[0]).cross(t[2] - t[0]);
            if n.length_squared() < 1e-8 || n.normalize().y.abs() < FLAT {
                continue;
            }
            raw.push(((t[0].y + t[1].y + t[2].y) / 3.0, false, *t));
        }
    }
    raw.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.total_cmp(&b.0)));
    let mut out: Vec<Level> = Vec::new();
    for (y, exact, t) in raw {
        let (lo, hi) = t.iter().fold(([f32::MAX; 2], [f32::MIN; 2]), |(lo, hi), v| {
            ([lo[0].min(v.x), lo[1].min(v.z)], [hi[0].max(v.x), hi[1].max(v.z)])
        });
        match out.last_mut() {
            Some(l) if l.exact == exact && (y - l.y).abs() <= LEVEL_MERGE => {
                l.y = (l.y * l.faces as f32 + y) / (l.faces + 1) as f32;
                l.lo = [l.lo[0].min(lo[0]), l.lo[1].min(lo[1])];
                l.hi = [l.hi[0].max(hi[0]), l.hi[1].max(hi[1])];
                l.faces += 1;
            }
            _ => out.push(Level { y, exact, lo, hi, faces: 1 }),
        }
    }
    out.sort_by_key(|a| std::cmp::Reverse(a.faces));
    out.truncate(MAX_LEVELS);
    out
}

/// The level a surface at `p` can be, within its box grown by `margin`.
pub fn level_at(levels: &[Level], p: Vec3, margin: f32) -> Option<&Level> {
    levels.iter().find(|l| l.holds(p.y) && l.covers(p.x, p.z, margin))
}

/// Whether a placed model (its triangles in world space) is water: nearly all of its area is flat faces on an exact
/// level. A shore's sand or a quay reaching the waterline is mostly elsewhere, so it is not. Levels without a water
/// height are left out: their faces are as often the bed as the surface (Bibiki Bay's sand, Windurst's house floors
/// and Port Windurst's sea floor all lie flat on one), and those zones' water is effect meshes.
pub fn is_water(tris: &[[Vec3; 3]], levels: &[Level]) -> bool {
    let exact: Vec<Level> = levels.iter().filter(|l| l.exact).copied().collect();
    if exact.is_empty() {
        return false;
    }
    let levels = &exact[..];
    let (mut all, mut on) = (0.0f32, 0.0f32);
    for t in tris {
        let n = (t[1] - t[0]).cross(t[2] - t[0]);
        let a = n.length() * 0.5;
        all += a;
        if a > 0.0
            && (n.y / (2.0 * a)).abs() >= FLAT
            && level_at(levels, (t[0] + t[1] + t[2]) / 3.0, LEVEL_MARGIN).is_some()
        {
            on += a;
        }
    }
    on >= WATER_AREA && on >= WATER_SHARE * all
}

/// Characters are mirrored in a reflecting surface this far below their feet at most (y grows downward), or this far
/// above them (feet in shallow water, or a floor's own unevenness).
const MIRROR_BELOW: f32 = 3.0;
const MIRROR_ABOVE: f32 = 0.5;
/// Characters farther than this from the eye are not mirrored.
pub const MIRROR_REACH: f32 = 40.0;
/// At most this many reflecting planes take mirrored characters a frame.
pub const MIRROR_PLANES: usize = 4;
/// The plane grid's cell side, world units.
const PLANE_CELL: f32 = 4.0;

/// The reflecting faces by 4-unit cell, for finding the surface under a character's feet; over effect water (no
/// faces on the CPU), the exact water level whose box holds the feet.
#[derive(Default)]
pub struct PlaneGrid {
    /// Each face's x/z corners and its plane as y = a x + b z + c, and whether it is a floor.
    faces: Vec<([[f32; 2]; 3], [f32; 3], bool)>,
    /// Per cell, the faces whose box covers it.
    cells: std::collections::HashMap<(i32, i32), Vec<u32>>,
    /// The exact water levels.
    levels: Vec<Level>,
}

impl PlaneGrid {
    /// Each face over the cells its box covers (faces standing on edge are left out).
    pub fn build(water: &WaterSurface, floors: &WaterSurface, levels: &[Level]) -> Self {
        let mut grid = Self { levels: levels.iter().filter(|l| l.exact).copied().collect(), ..Default::default() };
        for (s, floor) in [(water, false), (floors, true)] {
            for f in s.verts.as_chunks::<3>().0 {
                let [p0, p1, p2] = f.map(|v| Vec3::from(v.pos));
                let n = (p1 - p0).cross(p2 - p0);
                if n.y.abs() < 1e-6 {
                    continue;
                }
                // n . (p - p0) = 0 solved for y.
                let plane = [-n.x / n.y, -n.z / n.y, p0.y + (n.x * p0.x + n.z * p0.z) / n.y];
                let id = grid.faces.len() as u32;
                grid.faces.push(([[p0.x, p0.z], [p1.x, p1.z], [p2.x, p2.z]], plane, floor));
                let c = |v: f32| (v / PLANE_CELL).floor() as i32;
                for cx in c(p0.x.min(p1.x).min(p2.x))..=c(p0.x.max(p1.x).max(p2.x)) {
                    for cz in c(p0.z.min(p1.z).min(p2.z))..=c(p0.z.max(p1.z).max(p2.z)) {
                        grid.cells.entry((cx, cz)).or_default().push(id);
                    }
                }
            }
        }
        grid
    }

    /// The reflecting surface's height under `feet`: of the faces whose triangle holds the feet's x/z, the height
    /// nearest the feet from MIRROR_ABOVE above them to MIRROR_BELOW below; floors only when `floors`. With no face,
    /// an exact water level in that range whose box holds the feet (the sea an effect mesh draws).
    pub fn under(&self, feet: Vec3, floors: bool) -> Option<f32> {
        let key = ((feet.x / PLANE_CELL).floor() as i32, (feet.z / PLANE_CELL).floor() as i32);
        let (x, z) = (feet.x, feet.z);
        let in_range = |h: &f32| *h >= feet.y - MIRROR_ABOVE && *h <= feet.y + MIRROR_BELOW;
        let nearest = |a: &f32, b: &f32| (a - feet.y).abs().total_cmp(&(b - feet.y).abs());
        let face = self.cells.get(&key).and_then(|cell| {
            cell.iter()
                .map(|&i| &self.faces[i as usize])
                .filter(|(t, _, fl)| (floors || !fl) && inside(t, x, z))
                .map(|(_, p, _)| p[0] * x + p[1] * z + p[2])
                .filter(in_range)
                .min_by(nearest)
        });
        face.or_else(|| {
            self.levels.iter().filter(|l| l.covers(x, z, 0.0)).map(|l| l.y).filter(in_range).min_by(nearest)
        })
    }
}

/// Whether (x, z) lies in the triangle (edges included), either winding.
fn inside(t: &[[f32; 2]; 3], x: f32, z: f32) -> bool {
    let side = |a: [f32; 2], b: [f32; 2]| (b[0] - a[0]) * (z - a[1]) - (b[1] - a[1]) * (x - a[0]);
    let (d0, d1, d2) = (side(t[0], t[1]), side(t[1], t[2]), side(t[2], t[0]));
    (d0 >= 0.0 && d1 >= 0.0 && d2 >= 0.0) || (d0 <= 0.0 && d1 <= 0.0 && d2 <= 0.0)
}

/// The matrix mirroring world space in the plane y = h.
pub fn mirror(h: f32) -> Mat4 {
    Mat4::from_cols(Vec4::X, Vec4::NEG_Y, Vec4::Z, Vec4::new(0.0, 2.0 * h, 0.0, 1.0))
}

/// A face's kind (vertex `kind`): see-through water fades out where shallow; opaque water (it writes the depth the
/// fade would read) does not; a floor's is its strength.
pub const SEE_THROUGH: f32 = 0.0;
pub const OPAQUE: f32 = -1.0;

/// A reflecting face's corner as the shader takes it: world position, kind, and the surface's own colour (RGBA8,
/// gamma; a floor's is unused).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SurfVert {
    pub pos: [f32; 3],
    pub kind: f32,
    pub color: [u8; 4],
}

/// A reflecting face: world corners, kind (SEE_THROUGH, OPAQUE or a floor's strength) and colour.
#[derive(Clone, Copy, Debug)]
pub struct Face {
    pub corners: [Vec3; 3],
    pub kind: f32,
    pub color: [u8; 4],
}

/// Reflecting faces as one triangle list, grouped by tile.
#[derive(Default)]
pub struct WaterSurface {
    pub verts: Vec<SurfVert>,
    /// Per tile: its box and its range of `verts`.
    pub tiles: Vec<Tile>,
}

pub struct Tile {
    pub lo: Vec3,
    pub hi: Vec3,
    pub first: u32,
    pub count: u32,
}

impl WaterSurface {
    /// Faces grouped in tiles.
    pub fn from_faces(faces: Vec<Face>) -> Self {
        let mut faces: Vec<((i32, i32), Face)> = faces
            .into_iter()
            .map(|f| {
                let centre = (f.corners[0] + f.corners[1] + f.corners[2]) / 3.0;
                (((centre.x / TILE).floor() as i32, (centre.z / TILE).floor() as i32), f)
            })
            .collect();
        faces.sort_unstable_by_key(|f| f.0);
        let mut out = Self { verts: Vec::with_capacity(faces.len() * 3), tiles: Vec::new() };
        let mut k = 0;
        while k < faces.len() {
            let key = faces[k].0;
            let first = out.verts.len() as u32;
            let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
            while k < faces.len() && faces[k].0 == key {
                let f = faces[k].1;
                for v in f.corners {
                    lo = lo.min(v);
                    hi = hi.max(v);
                    out.verts.push(SurfVert { pos: v.to_array(), kind: f.kind, color: f.color });
                }
                k += 1;
            }
            out.tiles.push(Tile { lo, hi, first, count: out.verts.len() as u32 - first });
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.verts.is_empty()
    }

    /// The vertex ranges of the tiles in front of the camera (their boxes against the view's clip planes) and within
    /// REACH of `eye`, neighbouring ranges merged.
    pub fn visible(&self, view_proj: &Mat4, eye: Vec3) -> Vec<std::ops::Range<u32>> {
        let rows = [view_proj.row(0), view_proj.row(1), view_proj.row(2), view_proj.row(3)];
        // Left, right, bottom, top, near (0..1 depth), far.
        let planes =
            [rows[3] + rows[0], rows[3] - rows[0], rows[3] + rows[1], rows[3] - rows[1], rows[2], rows[3] - rows[2]];
        let mut out: Vec<std::ops::Range<u32>> = Vec::new();
        for t in &self.tiles {
            let near = eye.clamp(t.lo, t.hi);
            if near.distance_squared(eye) > REACH * REACH {
                continue;
            }
            let outside = planes.iter().any(|p: &Vec4| {
                // The box corner farthest along the plane's normal.
                let c = Vec3::new(
                    if p.x >= 0.0 { t.hi.x } else { t.lo.x },
                    if p.y >= 0.0 { t.hi.y } else { t.lo.y },
                    if p.z >= 0.0 { t.hi.z } else { t.lo.z },
                );
                p.truncate().dot(c) + p.w < 0.0
            });
            if outside {
                continue;
            }
            match out.last_mut() {
                Some(r) if r.end == t.first => r.end = t.first + t.count,
                _ => out.push(t.first..t.first + t.count),
            }
        }
        out
    }
}

/// A texture's mean colour (RGBA8, gamma), for a scenery water model's colour.
pub fn mean_rgba(rgba: &[u8]) -> [u8; 4] {
    let n = (rgba.len() / 4).max(1) as u64;
    let mut sum = [0u64; 4];
    for p in rgba.as_chunks::<4>().0 {
        for k in 0..4 {
            sum[k] += p[k] as u64;
        }
    }
    sum.map(|s| (s / n) as u8)
}

/// The colour of a scenery water face: its texture's mean times its corner colours' mean (vertex colours as D3D's
/// MODULATE2X: 0x80 is 1).
pub fn water_color(tex: [u8; 4], corners: [[u8; 4]; 3]) -> [u8; 4] {
    std::array::from_fn(|k| {
        let v = corners.iter().map(|c| c[k] as f32).sum::<f32>() / 3.0;
        let scale = if k == 3 { v / 255.0 } else { v / 128.0 };
        (tex[k] as f32 * scale).clamp(0.0, 255.0) as u8
    })
}

const WGSL: &str = include_str!("reflect.wgsl");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Params {
    pub view_proj: [[f32; 4]; 4],
    /// The projection alone and the inverse view: effect meshes come in view space (render/effects.rs `outs`).
    pub proj: [[f32; 4]; 4],
    pub view_inv: [[f32; 4]; 4],
    pub eye: [f32; 4],
    /// The camera's view direction (unit), for the scene's depth as a distance along each pixel's ray.
    pub forward: [f32; 4],
    /// x: 1 with a sky gradient; y: 1 when the target holds gamma bytes (no sRGB encode); z: seconds; w: 1 when the
    /// frame copy can be read (else every ray takes the sky).
    pub info: [f32; 4],
    pub clear: [f32; 4],
    pub fog_color: [f32; 4],
    /// x, y: fog start and end; z, w: the projection's near and far planes.
    pub fog: [f32; 4],
    pub sky_colors: [[f32; 4]; 8],
    pub sky_stops: [[f32; 4]; 2],
    /// The heights of the mirrored characters' layers, one per layer (`NO_PLANE` for an empty one).
    pub mirror_heights: [f32; 4],
    /// The water levels: height, 1 when exact (`Level`); x of `counts` is how many.
    pub levels: [[f32; 4]; MAX_LEVELS],
    pub counts: [f32; 4],
}

impl Params {
    /// The levels as the shader takes them.
    pub fn set_levels(&mut self, levels: &[Level]) {
        self.levels = [[0.0; 4]; MAX_LEVELS];
        for (k, l) in levels.iter().take(MAX_LEVELS).enumerate() {
            self.levels[k] = [l.y, l.exact as u32 as f32, 0.0, 0.0];
        }
        self.counts[0] = levels.len().min(MAX_LEVELS) as f32;
        self.counts[1] = std::env::var_os("FFXI_REFLECT_DEBUG").is_some() as u32 as f32;
    }
}

/// The height of an empty mirrored-characters layer, matching no surface.
pub const NO_PLANE: f32 = 1e9;

/// What the effects draw needs to reflect in an effect water mesh: the pipelines (see-through, then opaque) and the
/// frame's group 0 (render/effects.rs draws each with the mesh's texture at group 1).
pub struct FxWater {
    pub pipes: [wgpu::RenderPipeline; 2],
    pub group: wgpu::BindGroup,
    /// The zone's water heights, to skip meshes nowhere near one before drawing (the shader checks each vertex).
    pub heights: Vec<f32>,
}

/// An effect mesh whose origin is this far from every water height is not drawn again (its vertices could still
/// reach one this far out; sea sheets are placed at their surface).
const FX_NEAR: f32 = 8.0;

impl FxWater {
    /// Whether an effect mesh placed at height `y` (its read-back world position) may lie on the water.
    pub fn near_level(&self, y: f32) -> bool {
        self.heights.iter().any(|h| (y - h).abs() <= FX_NEAR)
    }
}

pub struct Reflections {
    layout: wgpu::BindGroupLayout,
    /// Group 0 for effect meshes: `layout`'s and the effects' per-element output (vertex stage).
    fx_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    /// The effect-mesh pipelines (see-through, opaque), made at the first frame with effects.
    fx_pipes: Option<[wgpu::RenderPipeline; 2]>,
    shader: wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    uniform: wgpu::Buffer,
    /// The water's and the floors' faces, each with its vertex buffer.
    surfaces: [(std::sync::Arc<WaterSurface>, wgpu::Buffer); 2],
    /// The zone's water levels; with none, no effect mesh is water.
    pub levels: std::sync::Arc<Vec<Level>>,
    /// Floors are drawn too (the option's "Water and Floors").
    pub floors: bool,
    /// The frame copy (same size and format as the scene) and the view its pixels are read through.
    copy: Option<(wgpu::Texture, wgpu::TextureView)>,
    /// This frame's copy was taken (else rays take the sky).
    copied: bool,
    /// This frame's mirrored-character heights, for the effect water drawn later.
    mirrored: Option<[f32; 4]>,
    /// A 1 x 1 stand-in when the scene can't be copied, and as a one-layer array when no character is mirrored.
    blank: wgpu::TextureView,
    blank_array: wgpu::TextureView,
    /// The heights under characters' feet (`PlaneGrid`).
    pub planes: PlaneGrid,
    /// The mirrored characters at half the scene's size, one array layer per plane (MIRROR_PLANES): its size, each
    /// layer's view to draw into, the whole array to read, and a depth buffer (cleared per plane).
    layer: Option<([u32; 2], Vec<wgpu::TextureView>, wgpu::TextureView, wgpu::TextureView)>,
    linear: wgpu::Sampler,
}

/// The colour target: the reflection over the frame by the weight in alpha; the frame's alpha kept.
fn target(format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        }),
        write_mask: wgpu::ColorWrites::ALL,
    }
}

impl Reflections {
    /// `format`: the scene's colour format (as the zone pass writes it).
    pub fn new(
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        water: std::sync::Arc<WaterSurface>,
        floors: std::sync::Arc<WaterSurface>,
        levels: std::sync::Arc<Vec<Level>>,
    ) -> Self {
        let device = &gpu.device;
        let tex = |binding, sample_type| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let entries = [
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            tex(1, wgpu::TextureSampleType::Float { filterable: false }),
            tex(2, wgpu::TextureSampleType::Float { filterable: false }),
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("reflections"),
            entries: &entries,
        });
        let mut fx_entries = entries.to_vec();
        fx_entries.push(wgpu::BindGroupLayoutEntry {
            binding: 5,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        let fx_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("reflections fx"),
            entries: &fx_entries,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("reflections"),
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("reflections"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("reflections"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<SurfVert>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Unorm8x4],
                })],
            },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            // The scene's depth is read in the shader instead, so the pass can sample it.
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(target(format))],
            }),
            multiview_mask: None,
            cache: None,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("reflections"),
            size: std::mem::size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let upload = |s: &WaterSurface| {
            let bytes: &[u8] = bytemuck::cast_slice(&s.verts);
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("reflections faces"),
                size: (bytes.len() as u64).max(20),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            if !bytes.is_empty() {
                gpu.queue.write_buffer(&buf, 0, bytes);
            }
            buf
        };
        let (wb, fb) = (upload(&water), upload(&floors));
        let blank_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("reflections blank"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format.remove_srgb_suffix(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let blank = blank_tex.create_view(&Default::default());
        let blank_array = blank_tex.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        log::info!(
            "reflections: {} water levels, {} scenery water faces in {} tiles, {} floor faces in {} tiles",
            levels.len(),
            water.verts.len() / 3,
            water.tiles.len(),
            floors.verts.len() / 3,
            floors.tiles.len()
        );
        let planes = PlaneGrid::build(&water, &floors, &levels);
        let linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("reflections layer"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            layout,
            fx_layout,
            pipeline,
            fx_pipes: None,
            shader,
            format,
            uniform,
            surfaces: [(water, wb), (floors, fb)],
            levels,
            floors: false,
            copy: None,
            copied: false,
            mirrored: None,
            blank,
            blank_array,
            planes,
            layer: None,
            linear,
        }
    }

    /// The mirrored characters' layers for a scene of `size` (half its size, in the pipelines' `format`): each
    /// plane's colour view and the shared depth view, made on first use or a size change.
    pub fn layer(
        &mut self,
        gpu: &Gpu,
        size: [u32; 2],
        format: wgpu::TextureFormat,
    ) -> (&[wgpu::TextureView], &wgpu::TextureView) {
        let half = [size[0].div_ceil(2).max(1), size[1].div_ceil(2).max(1)];
        if self.layer.as_ref().is_none_or(|l| l.0 != half) {
            let colour = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("reflections characters"),
                size: wgpu::Extent3d { width: half[0], height: half[1], depth_or_array_layers: MIRROR_PLANES as u32 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let each = (0..MIRROR_PLANES as u32)
                .map(|k| {
                    colour.create_view(&wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        base_array_layer: k,
                        array_layer_count: Some(1),
                        ..Default::default()
                    })
                })
                .collect();
            let all = colour.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });
            self.layer = Some((half, each, all, super::depth_view(&gpu.device, half[0], half[1])));
        }
        let l = self.layer.as_ref().unwrap();
        (&l.1, &l.3)
    }

    /// The frame's group 0 over `depth` (and `outs`, the effects' per-element output, for effect meshes).
    fn group(&self, gpu: &Gpu, depth: &wgpu::TextureView, outs: Option<&wgpu::Buffer>) -> wgpu::BindGroup {
        let scene = if self.copied { self.copy.as_ref().map_or(&self.blank, |c| &c.1) } else { &self.blank };
        let layer = match (&self.layer, self.mirrored) {
            (Some(l), Some(_)) => &l.2,
            _ => &self.blank_array,
        };
        let mut entries = vec![
            wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(depth) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(scene) },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(layer) },
            wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&self.linear) },
        ];
        if let Some(o) = outs {
            entries.push(wgpu::BindGroupEntry { binding: 5, resource: o.as_entire_binding() });
        }
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("reflections"),
            layout: if outs.is_some() { &self.fx_layout } else { &self.layout },
            entries: &entries,
        })
    }

    /// Takes the frame copy (when the zone has water or floors are in view), then draws the scenery water and floors
    /// in view over `color` (the scene, `color_tex` its texture when it can be copied), reading the scene's `depth`.
    /// `encode_gamma`: the zone pass writes gamma bytes (the copy is read the same way). `mirrored`: the heights of
    /// this frame's mirrored-character layers, when any was drawn. The copy, the uniforms and the mirrored layers
    /// stay for `fx_water` this frame.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        color_tex: Option<&wgpu::Texture>,
        depth: &wgpu::TextureView,
        mut params: Params,
        encode_gamma: bool,
        mirrored: Option<[f32; 4]>,
    ) {
        let vp = Mat4::from_cols_array_2d(&params.view_proj);
        let eye = Vec3::new(params.eye[0], params.eye[1], params.eye[2]);
        let ranges: Vec<Vec<std::ops::Range<u32>>> = self
            .surfaces
            .iter()
            .enumerate()
            .map(|(k, (s, _))| if k == 1 && !self.floors { Vec::new() } else { s.visible(&vp, eye) })
            .collect();
        self.copied = false;
        self.mirrored = None;
        // Effect water may draw later this frame wherever the zone has a level.
        if ranges.iter().all(Vec::is_empty) && self.levels.is_empty() {
            return;
        }
        let src = color_tex.filter(|t| t.usage().contains(wgpu::TextureUsages::COPY_SRC));
        if let Some(t) = src
            && self.copy.as_ref().is_none_or(|(c, _)| c.size() != t.size() || c.format() != t.format())
        {
            let c = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("reflections copy"),
                size: t.size(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: t.format(),
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[t.format().remove_srgb_suffix()],
            });
            // Read as the zone pass wrote it: raw bytes when the shaders encode gamma themselves.
            let vf = if encode_gamma { t.format().remove_srgb_suffix() } else { t.format() };
            let view = c.create_view(&wgpu::TextureViewDescriptor { format: Some(vf), ..Default::default() });
            self.copy = Some((c, view));
        }
        self.copied = match (src, &self.copy) {
            (Some(t), Some((c, _))) => {
                encoder.copy_texture_to_texture(t.as_image_copy(), c.as_image_copy(), t.size());
                true
            }
            _ => false,
        };
        self.mirrored = mirrored.filter(|_| self.layer.is_some());
        params.info[3] = self.copied as u32 as f32;
        params.mirror_heights = self.mirrored.unwrap_or([NO_PLANE; 4]);
        params.set_levels(&self.levels);
        gpu.queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&params));
        if ranges.iter().all(Vec::is_empty) {
            return;
        }
        let group = self.group(gpu, depth, None);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("reflections"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &group, &[]);
        for ((_, buf), ranges) in self.surfaces.iter().zip(ranges) {
            if ranges.is_empty() {
                continue;
            }
            pass.set_vertex_buffer(0, buf.slice(..));
            for r in ranges {
                pass.draw(r, 0..1);
            }
        }
    }

    /// The effect water for this frame's effects draw (after `draw`): `tex_layout` the effects' texture group,
    /// `outs` their per-element output, `depth` the scene's. None in a zone without water levels.
    pub fn fx_water(
        &mut self,
        gpu: &Gpu,
        tex_layout: &wgpu::BindGroupLayout,
        outs: &wgpu::Buffer,
        depth: &wgpu::TextureView,
    ) -> Option<FxWater> {
        if self.levels.is_empty() {
            return None;
        }
        if self.fx_pipes.is_none() {
            let pl = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("reflections fx"),
                bind_group_layouts: &[Some(&self.fx_layout), Some(tex_layout)],
                immediate_size: 0,
            });
            let pipe = |opaque: bool| {
                gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("reflections fx"),
                    layout: Some(&pl),
                    vertex: wgpu::VertexState {
                        module: &self.shader,
                        entry_point: Some("vs_fx"),
                        compilation_options: Default::default(),
                        buffers: &[Some(wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<crate::fx::lib::MeshVertex>() as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![
                                0 => Float32x3, 1 => Float32x3, 2 => Unorm8x4, 3 => Float32x2
                            ],
                        })],
                    },
                    primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &self.shader,
                        entry_point: Some("fs_fx"),
                        compilation_options: wgpu::PipelineCompilationOptions {
                            constants: &[("FX_OPAQUE", opaque as u32 as f64)],
                            ..Default::default()
                        },
                        targets: &[Some(target(self.format))],
                    }),
                    multiview_mask: None,
                    cache: None,
                })
            };
            self.fx_pipes = Some([pipe(false), pipe(true)]);
        }
        let group = self.group(gpu, depth, Some(outs));
        let heights = self.levels.iter().map(|l| l.y).collect();
        self.fx_pipes.clone().map(|pipes| FxWater { pipes, group, heights })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collision(tris: Vec<[Vec3; 3]>, kinds: Vec<u8>, flags: u32) -> Collision {
        let n = tris.len();
        Collision {
            tris,
            flags: vec![0; n],
            cam_pass: vec![false; n],
            decal: vec![false; n],
            normals: vec![Vec3::NEG_Y; n],
            kinds,
            object: vec![0; n],
            objects: vec![crate::dat::collision::Object {
                flags,
                y_range: (0.0, 0.0),
                area: 0,
                lights: [0; 4],
                link: 0,
                door: 0,
                vis: 0,
            }],
            entries: Vec::new(),
            cells: Vec::new(),
            grid: (0, 0),
        }
    }

    fn flat(x: f32, y: f32) -> [Vec3; 3] {
        [Vec3::new(x, y, 0.0), Vec3::new(x + 1.0, y, 0.0), Vec3::new(x, y, 1.0)]
    }

    /// A reflecting face from world corners.
    fn face(corners: [Vec3; 3], kind: f32) -> Face {
        Face { corners, kind, color: [255; 4] }
    }

    /// Camera and frame uniforms for the GPU tests: no fog nearby, `clear` the sky, the copy unread.
    fn params(cam: &crate::render::Camera, clear: [f32; 4]) -> Params {
        Params {
            view_proj: cam.view_proj(1.0).to_cols_array_2d(),
            proj: cam.proj(1.0).to_cols_array_2d(),
            view_inv: cam.view().inverse().to_cols_array_2d(),
            eye: [cam.eye.x, cam.eye.y, cam.eye.z, 1.0],
            forward: (cam.target - cam.eye).normalize().extend(0.0).to_array(),
            info: [0.0, 1.0, 0.0, 0.0],
            clear,
            fog_color: [0.0; 4],
            fog: [500.0, 1000.0, crate::render::NEAR, crate::render::FAR],
            sky_colors: [[0.0; 4]; 8],
            sky_stops: [[0.0; 4]; 2],
            mirror_heights: [NO_PLANE; 4],
            levels: [[0.0; 4]; MAX_LEVELS],
            counts: [0.0; 4],
        }
    }

    #[test]
    fn levels_from_the_water_faces() {
        // Water height 1.5 (bits 6..25, 1/256 units, shifted as water_of reads it).
        let flags = ((1.5f32 * 256.0) as u32) << 6;
        assert_eq!(water_of(flags), 1.5);
        // A bed below the surface (y 3 > 1.5) and a bank wholly above it (y 0): one exact level at 1.5.
        let c = collision(vec![flat(0.0, 3.0), flat(50.0, 0.0), flat(60.0, 3.1)], vec![9, 9, 9], flags);
        let l = water_levels(&c);
        assert_eq!(l.len(), 1);
        assert_eq!((l[0].y, l[0].exact, l[0].faces), (1.5, true, 2));
        assert_eq!((l[0].lo, l[0].hi), ([0.0, 0.0], [61.0, 1.0]));
        // Without a height: flat faces only, by their own height; other kinds and steep faces left out.
        let steep = [Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)];
        let c = collision(vec![flat(0.0, 2.0), steep, flat(100.0, 2.0), flat(5.0, 2.1)], vec![9, 9, 1, 9], 0);
        let l = water_levels(&c);
        assert_eq!(l.len(), 1);
        assert!(!l[0].exact && (l[0].y - 2.05).abs() < 1e-5 && l[0].faces == 2, "{l:?}");
    }

    #[test]
    fn levels_hold_the_drawn_surface() {
        let exact = Level { y: 20.0, exact: true, lo: [0.0, 0.0], hi: [10.0, 10.0], faces: 1 };
        // Bibiki Bay's sea half a unit off its collision height still holds; a quay 2 above does not.
        assert!(exact.holds(19.5) && exact.holds(20.6));
        assert!(!exact.holds(18.0) && !exact.holds(21.0));
        // A bed level: the surface may stand up to BED_ABOVE above it (y grows downward), not below it.
        let bed = Level { exact: false, ..exact };
        assert!(bed.holds(17.5) && bed.holds(20.5));
        assert!(!bed.holds(16.0) && !bed.holds(21.0));
        assert!(exact.covers(12.0, -2.0, 4.0) && !exact.covers(15.0, 5.0, 4.0));
        assert_eq!(level_at(&[bed, exact], Vec3::new(5.0, 20.2, 5.0), 0.0), Some(&bed));
        assert_eq!(level_at(&[exact], Vec3::new(50.0, 20.0, 5.0), 4.0), None);
    }

    #[test]
    fn water_is_a_model_wholly_on_a_level() {
        let levels = [Level { y: 20.0, exact: true, lo: [0.0, 0.0], hi: [100.0, 100.0], faces: 1 }];
        let sheet = |y: f32, x: f32| {
            [
                [Vec3::new(x, y, 0.0), Vec3::new(x + 10.0, y, 0.0), Vec3::new(x, y, 10.0)],
                [Vec3::new(x + 10.0, y, 0.0), Vec3::new(x + 10.0, y, 10.0), Vec3::new(x, y, 10.0)],
            ]
        };
        // A sheet at the level: water.
        assert!(is_water(&sheet(20.1, 0.0), &levels));
        // A beach: half at the waterline, half up the shore: not water.
        let beach = [sheet(20.0, 0.0), sheet(17.0, 10.0)].concat();
        assert!(!is_water(&beach, &levels));
        // A sheet away from the level's faces, or a stray small face on it: not water.
        assert!(!is_water(&sheet(20.0, 300.0), &levels));
        let tiny = [[Vec3::new(1.0, 20.0, 1.0), Vec3::new(2.0, 20.0, 1.0), Vec3::new(1.0, 20.0, 2.0)]];
        assert!(!is_water(&tiny, &levels));
        assert!(!is_water(&sheet(20.0, 0.0), &[]));
        // Only exact levels: a sheet on a bed level may be the bed.
        assert!(!is_water(&sheet(20.0, 0.0), &[Level { exact: false, ..levels[0] }]));
    }

    #[test]
    fn water_colour_from_texture_and_corners() {
        assert_eq!(mean_rgba(&[0, 100, 200, 255, 100, 100, 0, 1]), [50, 100, 100, 128]);
        // Vertex colours as MODULATE2X (0x80 is 1), alpha as is.
        assert_eq!(water_color([100, 50, 200, 255], [[0x80, 0x80, 0x80, 0xff]; 3]), [100, 50, 200, 255]);
        assert_eq!(water_color([100, 100, 100, 200], [[0x40, 0xff, 0x80, 0x80]; 3]), [50, 199, 100, 100]);
    }

    #[test]
    fn tiles_cull_by_view_and_reach() {
        let s = WaterSurface::from_faces(
            [flat(0.0, 0.0), flat(40.0, 0.0), flat(1000.0, 0.0)].into_iter().map(|t| face(t, SEE_THROUGH)).collect(),
        );
        assert_eq!(s.tiles.len(), 3);
        // Looking down +x from above (y is down): the first two tiles are in view, the far one is past REACH.
        let eye = Vec3::new(-10.0, -5.0, 0.5);
        let view = glam::camera::rh::view::look_at_mat4(eye, Vec3::new(20.0, 0.0, 0.5), Vec3::NEG_Y);
        let proj = glam::camera::rh::proj::directx::perspective(1.0, 1.5, 0.1, 2000.0);
        let vis = s.visible(&(proj * view), eye);
        assert_eq!(vis, vec![0..6]);
        // Looking the other way: none.
        let back = glam::camera::rh::view::look_at_mat4(eye, Vec3::new(-40.0, 0.0, 0.5), Vec3::NEG_Y);
        assert!(s.visible(&(proj * back), eye).is_empty());
    }

    fn read(gpu: &Gpu, tex: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
        let row = (w * 4).div_ceil(256) * 256;
        let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (row * h) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        enc.copy_texture_to_buffer(
            tex.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) },
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        gpu.queue.submit([enc.finish()]);
        buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        gpu.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).unwrap();
        let d = buf.slice(..).get_mapped_range().unwrap();
        (0..h).flat_map(|y| d[(y * row) as usize..(y * row + w * 4) as usize].to_vec()).collect()
    }

    /// A large pond under the camera, the frame black: with nothing in the depth (far) the water takes the sky's
    /// colour (red here) at the Fresnel weight; with scenery in front of it everywhere (depth 0), nothing changes; with ground
    /// just under the surface (a shore) it is faint.
    #[test]
    fn gpu_sky_on_open_water_none_behind_scenery() {
        let Ok(gpu) = Gpu::headless() else { return };
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let pond = [Vec3::new(-500.0, 0.0, -500.0), Vec3::new(500.0, 0.0, -500.0), Vec3::new(0.0, 0.0, 500.0)];
        let water = WaterSurface::from_faces(vec![face(pond, SEE_THROUGH)]);
        let mut r = Reflections::new(&gpu, format, std::sync::Arc::new(water), Default::default(), Default::default());
        let n = 32u32;
        let cam = crate::render::Camera {
            jitter: glam::Vec2::ZERO,
            eye: Vec3::new(0.0, -3.0, -20.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::NEG_Y,
            fov_y: 0.8,
        };
        let mut params = params(&cam, [1.0, 0.0, 0.0, 1.0]);
        // The depth of ground 0.2 under the water at the pixel checked: shallow, so the reflection there is faint.
        let vp = cam.view_proj(1.0);
        let ndc = glam::Vec2::new(0.0 + 1.0 / n as f32, -0.5 - 1.0 / n as f32);
        let (a, b) = (vp.inverse().project_point3(ndc.extend(0.0)), vp.inverse().project_point3(ndc.extend(1.0)));
        let ground = a + (b - a) * ((0.2 - a.y) / (b.y - a.y));
        let shallow = vp.project_point3(ground).z;
        let mut centre = Vec::new();
        for d in [1.0f32, 0.0, shallow] {
            let depth = crate::render::depth_view(&gpu.device, n, n);
            let color = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = color.create_view(&Default::default());
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(d), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            params.info[3] = 0.0;
            // No frame copy: every ray takes the sky, so the colour shows where the water drew.
            r.draw(&gpu, &mut enc, &view, None, &depth, params, true, None);
            gpu.queue.submit([enc.finish()]);
            let px = read(&gpu, &color, n, n);
            // The lower middle of the screen looks at the water.
            let at = ((n * 3 / 4) * n + n / 2) as usize * 4;
            centre.push([px[at], px[at + 1], px[at + 2]]);
        }
        assert!(centre[0][0] > 20 && centre[0][1] == 0 && centre[0][2] == 0, "open water: {:?}", centre[0]);
        assert_eq!(centre[1], [0, 0, 0], "behind scenery");
        assert!(centre[2][0] * 4 < centre[0][0], "shallow water at the shore: {:?}", centre[2]);
    }

    #[test]
    fn floors_face_up_and_keep_their_strength() {
        let up = [[0.0, -1.0, 0.0]; 3];
        assert!(faces_up(&Mat4::IDENTITY, up));
        assert!(!faces_up(&Mat4::IDENTITY, [[0.0, 1.0, 0.0]; 3]));
        // A placement turned over: its "up" normals now point down.
        assert!(!faces_up(&Mat4::from_rotation_x(std::f32::consts::PI), up));
        let s = WaterSurface::from_faces(vec![face(flat(0.0, 0.0), 0.3)]);
        assert!(s.verts.iter().all(|v| v.kind == 0.3));
        assert_eq!(floor_textures(243).len(), 4);
        assert!(floor_textures(100).is_empty());
    }

    /// A floor under the camera with a wall of scenery standing across the view (constant depth): the floor in front
    /// of the wall reflects the wall (the copy is red), by its strength; without a copy it draws nothing (no sky).
    #[test]
    fn gpu_floor_reflects_the_scene_only() {
        let Ok(gpu) = Gpu::headless() else { return };
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let n = 32u32;
        let cam = crate::render::Camera {
            jitter: glam::Vec2::ZERO,
            eye: Vec3::new(0.0, -1.6, -6.0),
            target: Vec3::new(0.0, -1.0, 0.0),
            up: Vec3::NEG_Y,
            fov_y: 0.9,
        };
        let floor =
            vec![face([Vec3::new(-50.0, 0.0, -50.0), Vec3::new(50.0, 0.0, -50.0), Vec3::new(0.0, 0.0, 50.0)], 0.5)];
        let floors = std::sync::Arc::new(WaterSurface::from_faces(floor));
        let mut r = Reflections::new(&gpu, format, Default::default(), floors, Default::default());
        r.floors = true;
        let vp = cam.view_proj(1.0);
        // The wall: everything 5 units in front of the eye.
        let wall = vp.project_point3(cam.eye + (cam.target - cam.eye).normalize() * 5.0).z;
        let params = params(&cam, [0.0, 0.0, 1.0, 1.0]);
        let tex = |clear: wgpu::Color, enc: &mut wgpu::CommandEncoder| {
            let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let v = t.create_view(&Default::default());
            enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &v,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            (t, v)
        };
        let mut got = Vec::new();
        for copy in [true, false] {
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            let (target, view) = tex(wgpu::Color::BLACK, &mut enc);
            let (red, _) = tex(wgpu::Color::RED, &mut enc);
            let depth = crate::render::depth_view(&gpu.device, n, n);
            enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(wall), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            r.draw(&gpu, &mut enc, &view, copy.then_some(&red), &depth, params, true, None);
            gpu.queue.submit([enc.finish()]);
            let px = read(&gpu, &target, n, n);
            // Near the bottom of the screen: floor close to the eye, in front of the wall.
            let at = ((n - 3) * n + n / 2) as usize * 4;
            got.push([px[at], px[at + 1], px[at + 2]]);
        }
        assert!(got[0][0] > 20 && got[0][2] == 0, "floor reflecting the wall: {:?}", got[0]);
        assert_eq!(got[1], [0, 0, 0], "no copy: nothing, not the sky");
    }

    #[test]
    fn planes_under_feet_and_the_mirror() {
        let water = WaterSurface::from_faces(vec![face(flat(0.0, 2.0), SEE_THROUGH)]);
        let floors = WaterSurface::from_faces(vec![face(flat(0.0, 0.0), 0.3)]);
        let g = PlaneGrid::build(&water, &floors, &[]);
        // Standing on the floor at y 0: the floor; with floors off, the water 2 below.
        assert_eq!(g.under(Vec3::new(0.2, 0.0, 0.2), true), Some(0.0));
        assert_eq!(g.under(Vec3::new(0.2, 0.0, 0.2), false), Some(2.0));
        // Far above both (y grows downward), or away from them: none.
        assert_eq!(g.under(Vec3::new(0.2, -10.0, 0.2), true), None);
        assert_eq!(g.under(Vec3::new(40.0, 0.0, 0.2), true), None);
        // In the faces' cell but outside the triangles (a dry bank by a diagonal shore): none.
        assert_eq!(g.under(Vec3::new(0.9, 0.0, 0.9), true), None);
        // A sloped face: the height under the feet, not the corners' mean.
        let slope = WaterSurface::from_faces(vec![face(
            [Vec3::new(0.0, 0.0, 0.0), Vec3::new(4.0, 0.0, 0.0), Vec3::new(0.0, 2.0, 4.0)],
            0.3,
        )]);
        let g = PlaneGrid::build(&Default::default(), &slope, &[]);
        let h = g.under(Vec3::new(1.0, 0.0, 0.5), true).unwrap();
        assert!((h - 0.25).abs() < 1e-5, "{h}");
        // Over effect water (no faces): an exact level whose box holds the feet, within reach; not a bed level.
        let sea = Level { y: 20.0, exact: true, lo: [0.0, 0.0], hi: [100.0, 100.0], faces: 1 };
        let bed = Level { y: 10.0, exact: false, ..sea };
        let g = PlaneGrid::build(&Default::default(), &Default::default(), &[sea, bed]);
        assert_eq!(g.under(Vec3::new(50.0, 19.5, 50.0), false), Some(20.0));
        assert_eq!(g.under(Vec3::new(50.0, 10.0, 50.0), false), None);
        assert_eq!(g.under(Vec3::new(150.0, 19.5, 50.0), false), None);
        // The mirror in y = 1 takes a point 1 above the plane (y 0) to 1 below it (y 2), x and z kept.
        assert!(mirror(1.0).transform_point3(Vec3::new(3.0, 0.0, -4.0)).abs_diff_eq(Vec3::new(3.0, 2.0, -4.0), 1e-6));
    }

    /// The mirrored characters' layer (here filled green over its whole size) shows on a floor even where the march
    /// finds nothing (no frame copy), and is left out when the frame has none (`mirrored` false).
    #[test]
    fn gpu_mirrored_layer_on_the_floor() {
        let Ok(gpu) = Gpu::headless() else { return };
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let n = 32u32;
        let cam = crate::render::Camera {
            jitter: glam::Vec2::ZERO,
            eye: Vec3::new(0.0, -1.6, -6.0),
            target: Vec3::new(0.0, -1.0, 0.0),
            up: Vec3::NEG_Y,
            fov_y: 0.9,
        };
        let floor =
            vec![face([Vec3::new(-50.0, 0.0, -50.0), Vec3::new(50.0, 0.0, -50.0), Vec3::new(0.0, 0.0, 50.0)], 0.5)];
        let floors = std::sync::Arc::new(WaterSurface::from_faces(floor));
        let mut r = Reflections::new(&gpu, format, Default::default(), floors, Default::default());
        r.floors = true;
        let params = params(&cam, [0.0, 0.0, 1.0, 1.0]);
        let mut got = Vec::new();
        for mirrored in [true, false] {
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = target.create_view(&Default::default());
            let depth = crate::render::depth_view(&gpu.device, n, n);
            let layers = r.layer(&gpu, [n, n], format).0;
            let (green, red) = (layers[0].clone(), layers[1].clone());
            for (v, c) in [(&view, wgpu::Color::BLACK), (&green, wgpu::Color::GREEN), (&red, wgpu::Color::RED)] {
                enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: v,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(c), store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            }
            enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            // The floor (y 0) takes layer 0's characters, not layer 1's (another plane's).
            let heights = mirrored.then_some([0.0, -4.0, NO_PLANE, NO_PLANE]);
            r.draw(&gpu, &mut enc, &view, None, &depth, params, true, heights);
            gpu.queue.submit([enc.finish()]);
            let px = read(&gpu, &target, n, n);
            let at = ((n - 3) * n + n / 2) as usize * 4;
            got.push([px[at], px[at + 1], px[at + 2]]);
        }
        assert!(got[0][1] > 40 && got[0][0] == 0, "the floor's own mirrored character only: {:?}", got[0]);
        assert_eq!(got[1], [0, 0, 0], "no layer this frame");
    }

    /// The water faces of some zones and the time to find them (by hand: `cargo test --release water_census --
    /// --ignored --nocapture`).
    /// Per zone: its water levels and the scenery textures with upward faces on one (`cargo test --release
    /// level_census -- --ignored --nocapture`; FFXI_ZONES=a,b,.. for others).
    /// The effect-water pipelines build against the effects' own texture layout and output buffer (both backends'
    /// shader translation), and only in a zone with a water level.
    #[test]
    fn gpu_fx_water_pipelines() {
        let Ok(gpu) = Gpu::headless() else { return };
        let format = wgpu::TextureFormat::Rgba8Unorm;
        // A GPU short of what the effects need has no effect water either.
        let Ok(eg) = crate::render::effects::EffectsGpu::new(&gpu, format) else { return };
        let depth = crate::render::depth_view(&gpu.device, 8, 8);
        let mut none = Reflections::new(&gpu, format, Default::default(), Default::default(), Default::default());
        assert!(none.fx_water(&gpu, eg.tex_layout(), eg.outs(), &depth).is_none());
        let sea = Level { y: 20.0, exact: true, lo: [0.0; 2], hi: [1.0; 2], faces: 1 };
        let mut r =
            Reflections::new(&gpu, format, Default::default(), Default::default(), std::sync::Arc::new(vec![sea]));
        let w = r.fx_water(&gpu, eg.tex_layout(), eg.outs(), &depth).expect("pipelines");
        assert!(w.near_level(25.0) && !w.near_level(40.0));
    }

    #[test]
    #[ignore]
    fn level_census() {
        use crate::dat::zone::{Keys, file_id, load_scene};
        let cfg = crate::config::Config::load();
        let Ok(index) = crate::dat::DatIndex::open(&cfg.ffxi_dir()) else { return };
        let Some(keys) =
            std::fs::read(crate::dat::ffximain(&cfg.ffxi_dir())).ok().and_then(|b| Keys::from_ffximain(&b).ok())
        else {
            return;
        };
        let zones: Vec<u32> = std::env::var("FFXI_ZONES").map_or(
            vec![4, 26, 50, 102, 103, 104, 106, 115, 118, 230, 231, 232, 234, 236, 238, 240, 243, 245, 246, 248],
            |v| v.split(',').filter_map(|z| z.parse().ok()).collect(),
        );
        for zone in zones {
            let Ok(file) = index.read(file_id(zone)) else { continue };
            let Ok(scene) = load_scene(&keys, &file) else { continue };
            let Some(col) = &scene.collision else { continue };
            let t = std::time::Instant::now();
            let levels = water_levels(col);
            let lt = t.elapsed().as_secs_f64() * 1000.0;
            let desc: Vec<String> = levels
                .iter()
                .map(|l| {
                    format!(
                        "{}{:.2}x{} [{:.0},{:.0}..{:.0},{:.0}]",
                        if l.exact { "=" } else { "~" },
                        l.y,
                        l.faces,
                        l.lo[0],
                        l.lo[1],
                        l.hi[0],
                        l.hi[1]
                    )
                })
                .collect();
            println!("zone {zone}: {} levels ({lt:.1} ms): {}", levels.len(), desc.join(" "));
            let names: std::collections::HashSet<&str> = scene.meshes.iter().map(|m| m.name.as_str()).collect();
            let mut by_tex: std::collections::BTreeMap<String, (u32, u16, f32)> = Default::default();
            let mut water: std::collections::BTreeMap<String, u32> = Default::default();
            for p in scene.placements.iter().filter(|p| p.drawn()) {
                let [_, _, h] = p.lod_names(|n| names.contains(n));
                let Some(mesh) = scene.meshes.iter().find(|m| m.name == h) else { continue };
                let m = Mat4::from_cols_array_2d(&p.matrix());
                for model in &mesh.models {
                    let li =
                        if model.strip { crate::render::strip_to_list(&model.indices) } else { model.indices.clone() };
                    let tris: Vec<[Vec3; 3]> = li
                        .as_chunks::<3>()
                        .0
                        .iter()
                        .filter(|t| t.iter().all(|&k| (k as usize) < model.vertices.len()))
                        .map(|t| t.map(|k| m.transform_point3(Vec3::from(model.vertices[k as usize].pos))))
                        .collect();
                    if is_water(&tris, &levels) {
                        *water.entry(format!("{} {}", mesh.name, model.texture)).or_default() += tris.len() as u32;
                    }
                    let idx =
                        if model.strip { crate::render::strip_to_list(&model.indices) } else { model.indices.clone() };
                    for tri in idx.as_chunks::<3>().0 {
                        let Some(v) = tri.iter().map(|&k| model.vertices.get(k as usize)).collect::<Option<Vec<_>>>()
                        else {
                            continue;
                        };
                        if !faces_up(&m, [v[0].normal, v[1].normal, v[2].normal]) {
                            continue;
                        }
                        let c = v.iter().map(|c| m.transform_point3(Vec3::from(c.pos))).sum::<Vec3>() / 3.0;
                        if let Some(l) = level_at(&levels, c, 4.0) {
                            let e = by_tex.entry(model.texture.clone()).or_insert((0, model.blending, 0.0));
                            e.0 += 1;
                            e.2 = c.y - l.y;
                        }
                    }
                }
            }
            for (tex, (n, blend, dy)) in by_tex.iter().filter(|e| e.1.0 >= 4) {
                let mean = scene.textures.iter().find(|t| &t.name == tex).map(|t| mean_rgba(t.rgba()));
                println!("    {tex:?} blend {blend:#06x}: {n} faces, last dy {dy:.2}, mean {mean:?}");
            }
            for (k, n) in &water {
                println!("    water: {k:?} {n} faces");
            }
        }
    }
}
