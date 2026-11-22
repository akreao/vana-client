//! Height fog (ours, not retail; Settings > Scene > Fog, config `height_fog`): mist that lies low, over the game's own
//! fog. Retail's fog (the zone's or the weather's colour, start and end, drawn by the zone shader) is left exactly as it
//! is; this adds a layer whose colour is that same fog colour this frame (so it follows the time of day, the weather and
//! the zone) and whose density comes from the same range: a zone whose retail fog ends close is mistier. The layer is
//! thickest at and below the camera's target (our character's feet in play) and thins exponentially above it, so
//! valleys and low ground fill while hill tops and the sky stay clear. One full-size pass from depth: the exponential
//! height density integrated in closed form along the eye ray, blended over the scene. The sky (depth at the clear
//! value) is never fogged.

use super::Gpu;
use super::scenefx::{Frame, pipeline};

/// (density at the reference height per retail fog end, falloff per unit of height, most of the scene it may cover)
/// for Height (1) and Dense (2).
pub fn params(strength: u32) -> [f32; 3] {
    match strength {
        2 => [8.0, 0.04, 0.85],
        _ => [4.0, 0.05, 0.7],
    }
}

/// The fade-in distance from the eye: retail's fog start, held within these (units).
const NEAR_CLEAR: [f32; 2] = [15.0, 60.0];

/// The least retail fog end used for the density (a zone with fog right at the eye would otherwise fill the screen).
const MIN_END: f32 = 60.0;

const WGSL: &str = r#"
struct U {
    inv_view_proj: mat4x4<f32>,
    // xyz: the eye; w: the reference height (world y, which points down).
    eye: vec4<f32>,
    // rgb: the fog colour (gamma space, as the scene); a: the most it covers.
    color: vec4<f32>,
    // x: density per unit at the reference height; y: falloff per unit of height above it; z: the distance over
    // which it fades in from the eye.
    p: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var depth: texture_2d<f32>;

@fragment fn fs_fog(i: VOut) -> @location(0) vec4<f32> {
    let d = textureLoad(depth, vec2<i32>(i.pos.xy), 0).x;
    if (d >= 1.0) { discard; }
    let h = u.inv_view_proj * vec4<f32>(i.uv.x * 2.0 - 1.0, 1.0 - i.uv.y * 2.0, d, 1.0);
    let p = h.xyz / h.w;
    let dist = distance(p, u.eye.xyz);
    let b = u.p.y;
    // Heights above the reference (y points down), held at -1/b below it so the density there tops out at e.
    let he = max(u.eye.w - u.eye.y, -1.0 / b);
    let hp = max(u.eye.w - p.y, -1.0 / b);
    let dh = hp - he;
    // The mean of exp(-b h) along the ray from the eye to the point.
    var mean = exp(-b * he);
    if (abs(dh * b) > 1e-3) {
        mean = (exp(-b * he) - exp(-b * hp)) / (b * dh);
    }
    // Eased in over the first stretch from the eye (to retail's fog start), so the ground at our feet stays clear.
    let amount = (1.0 - exp(-u.p.x * dist * mean)) * smoothstep(0.0, u.p.z, dist);
    return vec4<f32>(u.color.rgb, min(amount, u.color.a));
}
"#;

pub struct Fog {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    /// The group, by the depth view it reads.
    group: super::BindCache<wgpu::TextureView>,
    strength: u32,
}

/// The fog's uniform for `frame` at `strength`, or None when the frame has no zone fog (no pass then).
pub fn uniform(frame: &Frame, strength: u32) -> Option<[f32; 28]> {
    let f = frame.fog?;
    let [density, falloff, most] = params(strength);
    let mut u = [0.0f32; 28];
    u[..16].copy_from_slice(&frame.view_proj.inverse().to_cols_array());
    u[16..20].copy_from_slice(&[frame.eye.x, frame.eye.y, frame.eye.z, frame.target.y]);
    u[20..24].copy_from_slice(&[f.color[0], f.color[1], f.color[2], most]);
    u[24..28].copy_from_slice(&[
        density / f.end.max(MIN_END),
        falloff,
        f.start.clamp(NEAR_CLEAR[0], NEAR_CLEAR[1]),
        0.0,
    ]);
    Some(u)
}

impl Fog {
    /// `scene`: the scene target's format.
    pub fn new(gpu: &Gpu, scene: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("height fog"),
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
            label: Some("height fog"),
            source: wgpu::ShaderSource::Wgsl(format!("{}{WGSL}", super::scenefx::VERTEX).into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("height fog"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        // Over the scene by the fog's amount, alpha kept.
        let over = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: super::scenefx::ADD.alpha,
        };
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("height fog"),
            size: 28 * 4,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline: pipeline(device, "height fog", &pl, &module, "fs_fog", scene, Some(over)),
            layout,
            uniform,
            group: Default::default(),
            strength: 0,
        }
    }

    pub fn set_strength(&mut self, strength: u32) {
        self.strength = strength;
    }

    /// The fog over `scene` by `depth` (the scene's), when the frame has zone fog.
    pub fn draw(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        scene: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        frame: &Frame,
    ) {
        let Some(u) = uniform(frame, self.strength) else { return };
        super::Put::put(&gpu.queue, &self.uniform, 0, bytemuck::cast_slice(&u));
        let g = self.group.get(depth.clone(), || {
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("height fog"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(depth) },
                ],
            })
        });
        super::scenefx::pass(encoder, "height fog", scene, true, &self.pipeline, &g);
    }
}

#[cfg(test)]
mod tests {
    use super::super::scenefx::test_util::{filled, read};
    use super::super::scenefx::{FogFrame, Frame};
    use super::*;
    use glam::Vec3;

    fn frame(fog: Option<FogFrame>) -> (Frame, crate::render::Camera) {
        // Looking level along +z from 2 units above the reference (y down: eye y = -2, target y = 0).
        let cam = crate::render::Camera {
            eye: Vec3::new(0.0, -2.0, 0.0),
            target: Vec3::new(0.0, 0.0, 10.0),
            up: Vec3::NEG_Y,
            fov_y: 0.8,
        };
        (Frame { view_proj: cam.view_proj(1.0), eye: cam.eye, target: cam.target, fog }, cam)
    }

    #[test]
    fn no_zone_fog_no_uniform() {
        assert!(uniform(&frame(None).0, 1).is_none());
        let f = FogFrame { color: [0.5; 3], start: 20.0, end: 200.0 };
        let u = uniform(&frame(Some(f)).0, 2).unwrap();
        assert_eq!(u[19], 0.0, "reference height = the target's y");
        assert!((u[24] - 8.0 / 200.0).abs() < 1e-6);
    }

    /// The sky is left alone; geometry far away is fogged more than near, toward the fog colour, never past the cap.
    #[test]
    fn sky_untouched_far_fogged_more() {
        let Ok(gpu) = Gpu::headless() else { return };
        let n = 16u32;
        let f = FogFrame { color: [1.0, 1.0, 1.0], start: 20.0, end: 100.0 };
        let (fr, cam) = frame(Some(f));
        let mut fog = Fog::new(&gpu, wgpu::TextureFormat::Rgba8Unorm);
        fog.set_strength(2);
        let mut out = Vec::new();
        for dist in [None, Some(5.0f32), Some(80.0)] {
            let (tex, view) = filled(&gpu, [n, n], |_, _| [0, 0, 0, 255]);
            let depth = crate::render::depth_view(&gpu.device, n, n);
            let z = dist.map_or(1.0, |d| cam.view_proj(1.0).project_point3(cam.eye + Vec3::Z * d).z);
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(z), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            fog.draw(&gpu, &mut enc, &view, &depth, &fr);
            gpu.queue.submit([enc.finish()]);
            out.push(read(&gpu, &tex)[(n * n / 2 + n / 2) as usize]);
        }
        assert_eq!(out[0], [0, 0, 0, 255], "sky");
        assert!(out[1][0] > 0 && out[2][0] > out[1][0], "{out:?}");
        assert!(out[2][0] <= (0.85f32 * 255.0).ceil() as u8 && out[2][3] == 255, "{out:?}");
    }
}
