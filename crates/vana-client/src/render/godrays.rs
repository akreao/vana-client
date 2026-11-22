//! God rays (ours, not retail; Settings > Scene > Light Shafts, config `god_rays`): shafts of light from the sun
//! through gaps in the scenery. Screen-space: at half size, the open sky (depth at the clear value) near the sun's
//! place on screen is cut out by its brightness, blurred radially toward that place (light scattering along the eye
//! ray, decaying with each step), and the result, tinted with the zone's sun colour this frame, added to the scene.
//! The sun is the zone's own light (the environment's sun direction and colour); at night the moon gives faint shafts
//! on Strong only. Nothing is drawn indoors (an environment without a sky dome or with a fixed light), with the sun
//! behind the camera, or with no sky on screen near it. Retail's lens flare is left as it is.

use super::Gpu;
use super::scenefx::{ADD, Frame, pass, pipeline, soft_format, target};
use glam::{Vec3, Vec4Swizzles};

/// Samples along each ray, toward the sun.
const STEPS: u32 = 48;

/// (intensity, how much of the way to the sun the rays reach, decay per step, the moon's share) for Subtle (1) and
/// Strong (2).
pub fn params(strength: u32) -> [f32; 4] {
    match strength {
        2 => [0.45, 0.85, 0.965, 0.35],
        _ => [0.25, 0.7, 0.955, 0.0],
    }
}

const WGSL: &str = r#"
struct U {
    // xy: the sun's place in uv (0..1, top left 0); z: how much of the way to it the rays reach; w: decay per step.
    sun: vec4<f32>,
    // rgb: the tint x intensity x the fade (off screen, behind); a: the sky brightness floor cut out.
    tint: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var depth: texture_2d<f32>;

fn luma(c: vec3<f32>) -> f32 { return dot(c, vec3<f32>(0.299, 0.587, 0.114)); }

// Half size: the sky (the full-size pixel under this one at the clear depth) near the sun on screen (aspect kept).
@fragment fn fs_mask(i: VOut) -> @location(0) vec4<f32> {
    let full = vec2<f32>(textureDimensions(depth));
    let px = vec2<i32>(clamp(i.uv * full, vec2<f32>(0.0), full - 1.0));
    if (textureLoad(depth, px, 0).x < 1.0) { return vec4<f32>(0.0); }
    // The colour of that same pixel (a filtered tap at a silhouette would bring in the scenery's colour).
    let c = textureLoad(src, px, 0).rgb;
    let over = max(luma(c) - u.tint.a, 0.0) / max(1.0 - u.tint.a, 1e-3);
    let d = (i.uv - u.sun.xy) * vec2<f32>(full.x / full.y, 1.0);
    let near = 1.0 - smoothstep(0.0, 0.4, length(d));
    let core = 1.0 - smoothstep(0.0, 0.18, length(d));
    // The sky's own brightness, and the sun's glow itself: the dome has no sun disc (the sun is an effect sprite, and
    // may be hidden or not yet drawn), so the open sky around its place shines whatever its colour.
    return vec4<f32>(c * over * near + vec3<f32>(core * core) * 0.35, 1.0);
}

// Half size: the mask marched toward the sun, each step weighted down by the decay.
@fragment fn fs_blur(i: VOut) -> @location(0) vec4<f32> {
    let step = (u.sun.xy - i.uv) * u.sun.z / f32(STEPS);
    var uv = i.uv;
    var w = 1.0;
    var sum = vec3<f32>(0.0);
    var total = 0.0;
    for (var k = 0u; k < STEPS; k++) {
        sum += textureSampleLevel(src, samp, uv, 0.0).rgb * w;
        total += w;
        w *= u.sun.w;
        uv += step;
    }
    return vec4<f32>(sum / total, 1.0);
}

@fragment fn fs_apply(i: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSampleLevel(src, samp, i.uv, 0.0).rgb * u.tint.rgb, 0.0);
}
"#;

/// The sky brightness (gamma-space luma) below which nothing shines.
const FLOOR: f32 = 0.55;

/// Where the light comes from this frame: the direction toward it and its colour x share, or None (no shafts).
pub fn light(frame: &Frame, strength: u32) -> Option<(Vec3, Vec3)> {
    let s = frame.sun?;
    let share = if s.moon { params(strength)[3] } else { 1.0 };
    let toward = (-Vec3::from(s.dir)).try_normalize()?;
    // Under the horizon (y points down: the light must travel downward).
    if toward.y >= 0.0 || share <= 0.0 {
        return None;
    }
    Some((toward, Vec3::from(s.color) * share))
}

/// The uniform for `frame` at `strength`: the sun in uv and the tint, or None when there is nothing to draw (no sun,
/// behind the camera, faded out).
pub fn uniform(frame: &Frame, strength: u32) -> Option<[f32; 8]> {
    let (toward, color) = light(frame, strength)?;
    let [intensity, reach, decay, _] = params(strength);
    // The sun at infinity: w = 0.
    let c = frame.view_proj * toward.extend(0.0);
    if c.w <= 1e-4 {
        return None;
    }
    let ndc = c.xy() / c.w;
    let uv = [ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5];
    // Faded out as the sun leaves the screen (gone half a screen past the edge; ndc spans 2 across).
    let off = (ndc.x.abs().max(ndc.y.abs()) - 1.0).max(0.0);
    let fade = 1.0 - off.clamp(0.0, 1.0);
    if fade <= 0.0 {
        return None;
    }
    let t = color * intensity * fade * fade;
    Some([uv[0], uv[1], reach, decay, t.x, t.y, t.z, FLOOR])
}

pub struct GodRays {
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform: wgpu::Buffer,
    mask: wgpu::RenderPipeline,
    blur: wgpu::RenderPipeline,
    apply: wgpu::RenderPipeline,
    format: wgpu::TextureFormat,
    /// The scene size, the two half-size targets and the groups reading the first (blur) and the second (apply).
    size: [u32; 2],
    half: Option<[wgpu::TextureView; 2]>,
    groups: Option<[wgpu::BindGroup; 2]>,
    /// A 1 x 1 unfilterable stand-in for the depth slot of the passes that don't read it.
    no_depth: wgpu::TextureView,
    /// The mask's group, by the scene and depth views it reads.
    scene_group: super::BindCache<(wgpu::TextureView, wgpu::TextureView)>,
    strength: u32,
}

impl GodRays {
    /// `scene`: the scene target's format.
    pub fn new(gpu: &Gpu, scene: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let tex = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("god rays"),
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
                tex(1, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                tex(3, false),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("god rays"),
            source: wgpu::ShaderSource::Wgsl(
                format!("const STEPS: u32 = {STEPS}u;\n{}{WGSL}", super::scenefx::VERTEX).into(),
            ),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("god rays"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let format = soft_format(&gpu.adapter);
        let pipe = |entry, format, blend| pipeline(device, "god rays", &pl, &module, entry, format, blend);
        Self {
            mask: pipe("fs_mask", format, None),
            blur: pipe("fs_blur", format, None),
            apply: pipe("fs_apply", scene, Some(ADD)),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("god rays"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("god rays"),
                size: 32,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            layout,
            format,
            size: [0, 0],
            half: None,
            groups: None,
            no_depth: target(device, "god rays", [1, 1], wgpu::TextureFormat::Rgba8Unorm),
            scene_group: Default::default(),
            strength: 0,
        }
    }

    /// The half-size targets for a scene of `size`, made on first use or a size change.
    pub fn resize(&mut self, gpu: &Gpu, size: [u32; 2]) {
        if self.size == size && self.half.is_some() {
            return;
        }
        self.size = size;
        let half = size.map(|v| v.div_ceil(2).max(1));
        let [a, b] = [0, 1].map(|_| target(&gpu.device, "god rays", half, self.format));
        self.groups = Some([self.group(gpu, &a, &self.no_depth), self.group(gpu, &b, &self.no_depth)]);
        self.half = Some([a, b]);
        self.scene_group.clear();
    }

    pub fn set_strength(&mut self, strength: u32) {
        self.strength = strength;
    }

    fn group(&self, gpu: &Gpu, src: &wgpu::TextureView, depth: &wgpu::TextureView) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("god rays"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(src) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(depth) },
            ],
        })
    }

    /// The shafts added to `scene` (with its `depth`, the size given to `resize`), when the frame has a sun to cast
    /// them.
    pub fn draw(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        scene: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        frame: &Frame,
    ) {
        let (Some([a, b]), Some([g_blur, g_apply]), Some(u)) =
            (&self.half, &self.groups, uniform(frame, self.strength))
        else {
            return;
        };
        super::Put::put(&gpu.queue, &self.uniform, 0, bytemuck::cast_slice(&u));
        let g_mask = self.scene_group.get((scene.clone(), depth.clone()), || self.group(gpu, scene, depth));
        pass(encoder, "god rays mask", a, false, &self.mask, &g_mask);
        pass(encoder, "god rays blur", b, false, &self.blur, g_blur);
        pass(encoder, "god rays apply", scene, true, &self.apply, g_apply);
    }
}

#[cfg(test)]
mod tests {
    use super::super::scenefx::test_util::{filled, read};
    use super::super::scenefx::{Frame, SunFrame};
    use super::*;

    /// Open sky with the sun in the middle of the screen brightens round it more than at the edge; the same view
    /// with scenery everywhere (no sky) is left exactly as it was.
    #[test]
    fn sky_round_the_sun_shines_scenery_alone_does_not() {
        let Ok(gpu) = Gpu::headless() else { return };
        let n = 32u32;
        // Looking straight at a sun up and ahead (y down).
        let toward = Vec3::new(0.0, -0.5, 1.0).normalize();
        let cam = crate::render::Camera { eye: Vec3::ZERO, target: toward * 10.0, up: Vec3::NEG_Y, fov_y: 0.8 };
        let sun = SunFrame { dir: (-toward).to_array(), color: [1.0; 3], moon: false };
        let fr = Frame { view_proj: cam.view_proj(1.0), eye: cam.eye, target: cam.target, fog: None, sun: Some(sun) };
        let mut rays = GodRays::new(&gpu, wgpu::TextureFormat::Rgba8Unorm);
        rays.resize(&gpu, [n, n]);
        rays.set_strength(2);
        let mut out = Vec::new();
        for z in [1.0f32, 0.5] {
            let (tex, view) = filled(&gpu, [n, n], |_, _| [60, 60, 60, 255]);
            let depth = crate::render::depth_view(&gpu.device, n, n);
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
            rays.draw(&gpu, &mut enc, &view, &depth, &fr);
            gpu.queue.submit([enc.finish()]);
            out.push(read(&gpu, &tex));
        }
        let at = |px: &Vec<[u8; 4]>, x: u32, y: u32| px[(y * n + x) as usize][0];
        assert!(at(&out[0], 16, 16) > at(&out[0], 0, 0) && at(&out[0], 16, 16) > 60, "sky");
        assert!(out[1].iter().all(|p| *p == [60, 60, 60, 255]), "scenery only");
    }

    fn frame(sun: Option<SunFrame>, target: Vec3) -> Frame {
        let cam = crate::render::Camera { eye: Vec3::ZERO, target, up: Vec3::NEG_Y, fov_y: 0.8 };
        Frame { view_proj: cam.view_proj(16.0 / 9.0), eye: cam.eye, target: cam.target, fog: None, sun }
    }

    /// The sun ahead and up (y down) lands in the upper half of the screen; behind the camera, under the horizon or
    /// with no sun there is nothing; the moon only on Strong.
    #[test]
    fn sun_on_screen_or_nothing() {
        // Travelling toward +z and down (+y): it is behind us and up, so looking toward -z we face it.
        let sun = SunFrame { dir: [0.0, 0.5, 1.0], color: [1.0, 0.9, 0.8], moon: false };
        let ahead = Vec3::new(0.0, 0.0, -10.0);
        let u = uniform(&frame(Some(sun), ahead), 1).unwrap();
        assert!((u[0] - 0.5).abs() < 1e-4 && u[1] < 0.5, "{u:?}");
        assert!(u[4] > 0.0);
        assert!(uniform(&frame(Some(sun), -ahead), 1).is_none(), "behind");
        let below = SunFrame { dir: [0.0, -0.5, 1.0], ..sun };
        assert!(uniform(&frame(Some(below), ahead), 2).is_none(), "under the horizon");
        assert!(uniform(&frame(None, ahead), 2).is_none());
        let moon = SunFrame { moon: true, ..sun };
        assert!(uniform(&frame(Some(moon), ahead), 1).is_none());
        let strong = uniform(&frame(Some(moon), ahead), 2).unwrap();
        assert!(strong[4] < uniform(&frame(Some(sun), ahead), 2).unwrap()[4]);
    }
}
