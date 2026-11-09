//! Water reflections (ours, not retail; Settings > Graphics > Reflections, config `reflections`).
//!
//! Where: the zone's water as its collision knows it, the type-9 faces fishing and wading use. Much of the water
//! drawn is an effect mesh or part of a larger scenery mesh, so the collision is the one place every zone marks it.
//! A face with a water height (its object's, `water_of`) is a riverbed: it is laid flat at that height, and the
//! scenery's depth hides the parts that stand above the water (banks). A face without one is the surface itself and
//! is kept as it is when it lies flat.
//!
//! How: after the actors, one copy of the frame is taken, and the water faces in view are drawn once over it. Each
//! water pixel marches its reflected ray through the scene's depth a fixed number of steps; a hit takes the copy's
//! colour there, a miss (or a ray leaving the screen) takes retail's sky gradient in that direction. The result is
//! blended by a Fresnel weight and faded by the fog, and faded out where the water is shallow (the scene's depth
//! lies little below the surface), so a face laid a little above the drawn water leaves no band along the shore.
//! Ripples bend the surface normal, worked out in the shader.
//!
//! Floors (the option's "Water and Floors"): polished floors chosen by hand, per zone and texture (`floor_textures`).
//! Their upward-facing triangles are copied out of the scenery at load, in world space, and drawn in the same pass
//! with the strength listed for them. A floor reflects only what the march finds on screen, fading with the ray's
//! length (a glossy look, and no sky under a roof); it has no ripples and no shoreline fade.
//!
//! Cost: one scene-sized copy and a draw of the water in view, nothing at all when none is. The faces go in tiles
//! at load (a sort, linear apart from it) and each frame tests only the tiles' boxes.

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

/// Reflecting faces as one triangle list, grouped by tile.
#[derive(Default)]
pub struct WaterSurface {
    /// World positions, three per face; w: the face's strength as a floor (0 for water).
    pub verts: Vec<[f32; 4]>,
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
    pub fn build(c: &Collision) -> Self {
        let mut faces: Vec<([Vec3; 3], f32)> = Vec::new();
        for (i, t) in c.tris.iter().enumerate() {
            if c.kinds.get(i) != Some(&9) {
                continue;
            }
            let Some(o) = c.object.get(i).and_then(|&o| c.objects.get(o as usize)) else { continue };
            let h = water_of(o.flags);
            let face = if h != 0.0 {
                // A riverbed: only where some of it lies under the surface (y grows downward).
                if t.iter().all(|v| v.y < h - 0.05) {
                    continue;
                }
                t.map(|v| Vec3::new(v.x, h, v.z))
            } else {
                let n = (t[1] - t[0]).cross(t[2] - t[0]);
                if n.length_squared() < 1e-8 || n.normalize().y.abs() < FLAT {
                    continue;
                }
                *t
            };
            faces.push((face, 0.0));
        }
        Self::from_faces(faces)
    }

    /// Faces (world corners, floor strength or 0 for water) grouped in tiles.
    pub fn from_faces(faces: Vec<([Vec3; 3], f32)>) -> Self {
        let mut faces: Vec<((i32, i32), [Vec3; 3], f32)> = faces
            .into_iter()
            .map(|(f, w)| {
                let centre = (f[0] + f[1] + f[2]) / 3.0;
                (((centre.x / TILE).floor() as i32, (centre.z / TILE).floor() as i32), f, w)
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
                for v in faces[k].1 {
                    lo = lo.min(v);
                    hi = hi.max(v);
                    out.verts.push(v.extend(faces[k].2).to_array());
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

const WGSL: &str = include_str!("reflect.wgsl");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Params {
    pub view_proj: [[f32; 4]; 4],
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
}

pub struct Reflections {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    /// The water's and the floors' faces, each with its vertex buffer.
    surfaces: [(std::sync::Arc<WaterSurface>, wgpu::Buffer); 2],
    /// Floors are drawn too (the option's "Water and Floors").
    pub floors: bool,
    /// The frame copy (same size and format as the scene) and the view its pixels are read through.
    copy: Option<(wgpu::Texture, wgpu::TextureView)>,
    /// A 1 x 1 stand-in when the scene can't be copied.
    blank: wgpu::TextureView,
}

impl Reflections {
    /// `format`: the scene's colour format (as the zone pass writes it).
    pub fn new(
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        water: std::sync::Arc<WaterSurface>,
        floors: std::sync::Arc<WaterSurface>,
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
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("reflections"),
            entries: &[
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
                tex(1, wgpu::TextureSampleType::Depth),
                tex(2, wgpu::TextureSampleType::Float { filterable: false }),
            ],
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
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x4],
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
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // Colour over the frame by the weight in alpha; the frame's alpha kept.
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
                })],
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
                size: (bytes.len() as u64).max(16),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            if !bytes.is_empty() {
                gpu.queue.write_buffer(&buf, 0, bytes);
            }
            buf
        };
        let (wb, fb) = (upload(&water), upload(&floors));
        let blank = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("reflections blank"),
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: format.remove_srgb_suffix(),
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());
        log::info!(
            "reflections: {} water faces in {} tiles, {} floor faces in {} tiles",
            water.verts.len() / 3,
            water.tiles.len(),
            floors.verts.len() / 3,
            floors.tiles.len()
        );
        Self { layout, pipeline, uniform, surfaces: [(water, wb), (floors, fb)], floors: false, copy: None, blank }
    }

    /// Draws the water in view over `color` (the scene, `color_tex` its texture when it can be copied), reading the
    /// scene's `depth`. `encode_gamma`: the zone pass writes gamma bytes (the copy is read the same way).
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
    ) {
        let vp = Mat4::from_cols_array_2d(&params.view_proj);
        let eye = Vec3::new(params.eye[0], params.eye[1], params.eye[2]);
        let ranges: Vec<Vec<std::ops::Range<u32>>> = self
            .surfaces
            .iter()
            .enumerate()
            .map(|(k, (s, _))| if k == 1 && !self.floors { Vec::new() } else { s.visible(&vp, eye) })
            .collect();
        if ranges.iter().all(Vec::is_empty) {
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
        let copied = match (src, &self.copy) {
            (Some(t), Some((c, _))) => {
                encoder.copy_texture_to_texture(t.as_image_copy(), c.as_image_copy(), t.size());
                true
            }
            _ => false,
        };
        params.info[3] = copied as u32 as f32;
        gpu.queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&params));
        let scene = if copied { self.copy.as_ref().map_or(&self.blank, |c| &c.1) } else { &self.blank };
        let group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("reflections"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(depth) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(scene) },
            ],
        });
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

    #[test]
    fn surface_faces_kept_flat_others_dropped() {
        let steep = [Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)];
        let c = collision(vec![flat(0.0, 2.0), steep, flat(100.0, 2.0)], vec![9, 9, 1], 0);
        let s = WaterSurface::build(&c);
        assert_eq!(s.verts.len(), 3);
        assert_eq!(s.verts[0][1], 2.0);
    }

    #[test]
    fn riverbeds_laid_at_the_water_height() {
        // Water height 1.5 (bits 6..25, 1/256 units, shifted as water_of reads it).
        let flags = ((1.5f32 * 256.0) as u32) << 6;
        assert_eq!(water_of(flags), 1.5);
        // A bed below the surface (y 3 > 1.5) and a bank wholly above it (y 0).
        let c = collision(vec![flat(0.0, 3.0), flat(50.0, 0.0)], vec![9, 9], flags);
        let s = WaterSurface::build(&c);
        assert_eq!(s.verts.len(), 3);
        assert!(s.verts.iter().all(|v| v[1] == 1.5));
    }

    #[test]
    fn tiles_cull_by_view_and_reach() {
        let c = collision(vec![flat(0.0, 0.0), flat(40.0, 0.0), flat(1000.0, 0.0)], vec![9; 3], 0);
        let s = WaterSurface::build(&c);
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
        let c = collision(vec![pond], vec![9], 0);
        let mut r = Reflections::new(&gpu, format, std::sync::Arc::new(WaterSurface::build(&c)), Default::default());
        let n = 32u32;
        let cam = crate::render::Camera {
            eye: Vec3::new(0.0, -3.0, -20.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::NEG_Y,
            fov_y: 0.8,
        };
        let mut params = Params {
            view_proj: cam.view_proj(1.0).to_cols_array_2d(),
            eye: [cam.eye.x, cam.eye.y, cam.eye.z, 1.0],
            forward: (cam.target - cam.eye).normalize().extend(0.0).to_array(),
            info: [0.0, 1.0, 0.0, 0.0],
            clear: [1.0, 0.0, 0.0, 1.0],
            fog_color: [0.0; 4],
            fog: [500.0, 1000.0, crate::render::NEAR, crate::render::FAR],
            sky_colors: [[0.0; 4]; 8],
            sky_stops: [[0.0; 4]; 2],
        };
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
            r.draw(&gpu, &mut enc, &view, None, &depth, params, true);
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
        let s = WaterSurface::from_faces(vec![(flat(0.0, 0.0), 0.3)]);
        assert!(s.verts.iter().all(|v| v[3] == 0.3));
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
            eye: Vec3::new(0.0, -1.6, -6.0),
            target: Vec3::new(0.0, -1.0, 0.0),
            up: Vec3::NEG_Y,
            fov_y: 0.9,
        };
        let floor = vec![([Vec3::new(-50.0, 0.0, -50.0), Vec3::new(50.0, 0.0, -50.0), Vec3::new(0.0, 0.0, 50.0)], 0.5)];
        let mut r =
            Reflections::new(&gpu, format, Default::default(), std::sync::Arc::new(WaterSurface::from_faces(floor)));
        r.floors = true;
        let vp = cam.view_proj(1.0);
        // The wall: everything 8 units in front of the eye.
        let wall = vp.project_point3(cam.eye + (cam.target - cam.eye).normalize() * 8.0).z;
        let params = Params {
            view_proj: vp.to_cols_array_2d(),
            eye: [cam.eye.x, cam.eye.y, cam.eye.z, 1.0],
            forward: (cam.target - cam.eye).normalize().extend(0.0).to_array(),
            info: [0.0, 1.0, 0.0, 0.0],
            clear: [0.0, 0.0, 1.0, 1.0],
            fog_color: [0.0; 4],
            fog: [500.0, 1000.0, crate::render::NEAR, crate::render::FAR],
            sky_colors: [[0.0; 4]; 8],
            sky_stops: [[0.0; 4]; 2],
        };
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
            r.draw(&gpu, &mut enc, &view, copy.then_some(&red), &depth, params, true);
            gpu.queue.submit([enc.finish()]);
            let px = read(&gpu, &target, n, n);
            // Near the bottom of the screen: floor close to the eye, in front of the wall.
            let at = ((n - 3) * n + n / 2) as usize * 4;
            got.push([px[at], px[at + 1], px[at + 2]]);
        }
        assert!(got[0][0] > 20 && got[0][2] == 0, "floor reflecting the wall: {:?}", got[0]);
        assert_eq!(got[1], [0, 0, 0], "no copy: nothing, not the sky");
    }

    /// The water faces of some zones and the time to find them (by hand: `cargo test --release water_census --
    /// --ignored --nocapture`).
    #[test]
    #[ignore]
    fn water_census() {
        use crate::dat::zone::{Keys, file_id, load_scene};
        let cfg = crate::config::Config::load();
        let Ok(index) = crate::dat::DatIndex::open(&cfg.ffxi_dir()) else { return };
        let Some(keys) =
            std::fs::read(crate::dat::ffximain(&cfg.ffxi_dir())).ok().and_then(|b| Keys::from_ffximain(&b).ok())
        else {
            return;
        };
        for zone in [4u32, 26, 50, 102, 103, 104, 106, 115, 230, 231, 232, 234, 236, 238, 240, 243, 245, 246] {
            let Ok(file) = index.read(file_id(zone)) else { continue };
            let Ok(mut scene) = load_scene(&keys, &file) else { continue };
            scene.floors = floor_textures(zone).iter().map(|(n, w)| (n.to_string(), *w)).collect();
            if !scene.floors.is_empty() {
                let t = std::time::Instant::now();
                let geo = crate::render::ZoneGeometry::build(&scene);
                println!(
                    "zone {zone}: {} floor faces in {} tiles (whole geometry {:.1} ms)",
                    geo.floors.verts.len() / 3,
                    geo.floors.tiles.len(),
                    t.elapsed().as_secs_f64() * 1000.0
                );
            }
            let Some(col) = &scene.collision else { continue };
            let t = std::time::Instant::now();
            let s = WaterSurface::build(col);
            let (lo, hi) = s.verts.iter().fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(v[1]), b.max(v[1])));
            println!(
                "zone {zone}: {} faces in {} tiles, y {lo:.1}..{hi:.1}, {:.2} ms",
                s.verts.len() / 3,
                s.tiles.len(),
                t.elapsed().as_secs_f64() * 1000.0
            );
        }
    }
}
