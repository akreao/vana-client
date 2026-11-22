//! Depth of field (ours, not retail; Settings > Scene > Depth of Field, config `dof`): what the camera looks at stays
//! sharp and the scenery behind it softens with distance. The focus is the camera's target (our character in play, the
//! event camera's target in a cutscene), so it follows the third-person camera's zoom; nothing nearer than the focus
//! blurs (the third-person camera keeps the ground at our feet in view, and blurring it reads as a smeared screen).
//! Three passes: the scene halved with each pixel's blur size (from depth) in alpha, a 16-tap disc gather of that at
//! half size weighted by each tap's own blur size (so sharp things in front don't bleed into the blurred background
//! behind them), and the result blended over the full-size scene by the full-size pixel's own blur size.

use super::Gpu;
use super::scenefx::{Frame, pass, pipeline, soft_format, target};

/// (where the blur starts past the focus, the distance over which it grows to full, the widest blur in half-size
/// pixels) for Subtle (1) and Strong (2), in world units.
pub fn params(strength: u32) -> [f32; 3] {
    match strength {
        2 => [6.0, 60.0, 7.0],
        _ => [12.0, 140.0, 4.5],
    }
}

// The sky (depth at the clear value) blurs as the farthest scenery does.
const WGSL: &str = r#"
struct U {
    // x: the focus distance; y: blur start past it; z: the ramp to full; w: the widest blur (half-size pixels).
    f: vec4<f32>,
    // x, y: the projection's near and far planes.
    lin: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var depth: texture_2d<f32>;

fn linear(d: f32) -> f32 { return u.lin.x * u.lin.y / (u.lin.y - d * (u.lin.y - u.lin.x)); }
// The blur size 0..1 of a depth: none up to the focus plus the start, full past the ramp.
fn coc(d: f32) -> f32 {
    return clamp((linear(d) - u.f.x - u.f.y) / u.f.z, 0.0, 1.0);
}
fn depth_at(px: vec2<i32>) -> f32 {
    let size = vec2<i32>(textureDimensions(depth));
    return textureLoad(depth, clamp(px, vec2<i32>(0), size - 1), 0).x;
}

// Half size: the four full-size pixels under this one (the same four for colour and depth, clamped at an odd edge),
// averaged by their own blur sizes so a sharp silhouette's colour doesn't go into the blurred background beside it,
// with the largest blur size of the four in alpha.
@fragment fn fs_down(i: VOut) -> @location(0) vec4<f32> {
    let size = vec2<i32>(textureDimensions(depth));
    let px = vec2<i32>(floor(i.pos.xy)) * 2;
    var sum = vec3<f32>(0.0);
    var total = 0.0;
    var k = 0.0;
    for (var j = 0; j < 4; j++) {
        let q = clamp(px + vec2<i32>(j & 1, j >> 1u), vec2<i32>(0), size - 1);
        let c = coc(textureLoad(depth, q, 0).x);
        let w = c + 1e-3;
        sum += textureLoad(src, q, 0).rgb * w;
        total += w;
        k = max(k, c);
    }
    return vec4<f32>(sum / total, k);
}

// Half size: 16 taps on a golden-angle spiral out to this pixel's blur size, each weighted by its own.
@fragment fn fs_blur(i: VOut) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(src));
    let centre = textureSampleLevel(src, samp, i.uv, 0.0);
    let r = centre.a * u.f.w;
    var sum = centre.rgb * max(centre.a, 0.05);
    var total = max(centre.a, 0.05);
    for (var k = 1; k < 16; k++) {
        let t = f32(k) / 15.0;
        let a = f32(k) * 2.39996;
        let o = vec2<f32>(cos(a), sin(a)) * sqrt(t) * r * texel;
        let s = textureSampleLevel(src, samp, i.uv + o, 0.0);
        let w = s.a;
        sum += s.rgb * w;
        total += w;
    }
    return vec4<f32>(sum / total, centre.a);
}

// Full size: the blurred scene over the sharp one by this pixel's own blur size.
@fragment fn fs_apply(i: VOut) -> @location(0) vec4<f32> {
    let k = coc(depth_at(vec2<i32>(floor(i.pos.xy))));
    if (k <= 0.0) { discard; }
    return vec4<f32>(textureSampleLevel(src, samp, i.uv, 0.0).rgb, smoothstep(0.0, 1.0, k));
}
"#;

pub struct Dof {
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform: wgpu::Buffer,
    down: wgpu::RenderPipeline,
    blur: wgpu::RenderPipeline,
    apply: wgpu::RenderPipeline,
    format: wgpu::TextureFormat,
    size: [u32; 2],
    half: Option<[wgpu::TextureView; 2]>,
    /// The groups down (reads the scene), blur (the first half) and apply (the second), by the scene and depth views
    /// they were made for.
    groups: std::sync::Mutex<Option<(wgpu::TextureView, wgpu::TextureView, [wgpu::BindGroup; 3])>>,
    strength: u32,
}

/// The uniform for `frame` at `strength`: the focus is the eye's distance to the camera's target.
pub fn uniform(frame: &Frame, strength: u32) -> [f32; 8] {
    let [start, ramp, widest] = params(strength);
    let focus = frame.eye.distance(frame.target);
    [focus, start, ramp, widest, super::NEAR, super::FAR, 0.0, 0.0]
}

impl Dof {
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
            label: Some("depth of field"),
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
            label: Some("depth of field"),
            source: wgpu::ShaderSource::Wgsl(format!("{}{WGSL}", super::scenefx::VERTEX).into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("depth of field"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let format = soft_format(&gpu.adapter);
        let over = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: super::scenefx::ADD.alpha,
        };
        let pipe = |entry, format, blend| pipeline(device, "depth of field", &pl, &module, entry, format, blend);
        Self {
            down: pipe("fs_down", format, None),
            blur: pipe("fs_blur", format, None),
            apply: pipe("fs_apply", scene, Some(over)),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("depth of field"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("depth of field"),
                size: 32,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            layout,
            format,
            size: [0, 0],
            half: None,
            groups: Default::default(),
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
        self.half = Some([0, 1].map(|_| target(&gpu.device, "depth of field", half, self.format)));
        if let Ok(mut g) = self.groups.lock() {
            *g = None;
        }
    }

    pub fn set_strength(&mut self, strength: u32) {
        self.strength = strength;
    }

    fn group(&self, gpu: &Gpu, src: &wgpu::TextureView, depth: &wgpu::TextureView) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("depth of field"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(src) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(depth) },
            ],
        })
    }

    /// The background softened on `scene` by its `depth` (the size given to `resize`).
    pub fn draw(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        scene: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        frame: &Frame,
    ) {
        let Some([a, b]) = &self.half else { return };
        super::Put::put(&gpu.queue, &self.uniform, 0, bytemuck::cast_slice(&uniform(frame, self.strength)));
        // The three groups, made again only when the scene or depth view changes (resize or another target).
        let Ok(mut cache) = self.groups.lock() else { return };
        if cache.as_ref().is_none_or(|(s, d, _)| s != scene || d != depth) {
            let gs = [self.group(gpu, scene, depth), self.group(gpu, a, depth), self.group(gpu, b, depth)];
            *cache = Some((scene.clone(), depth.clone(), gs));
        }
        let Some((_, _, [g_down, g_blur, g_apply])) = cache.as_ref() else { return };
        pass(encoder, "depth of field down", a, false, &self.down, g_down);
        pass(encoder, "depth of field blur", b, false, &self.blur, g_blur);
        pass(encoder, "depth of field apply", scene, true, &self.apply, g_apply);
    }
}

#[cfg(test)]
mod tests {
    use super::super::scenefx::test_util::{filled, read};
    use super::*;
    use glam::Vec3;

    /// The focus follows the camera's distance to its target.
    #[test]
    fn focus_is_the_target_distance() {
        let cam = crate::render::Camera {
            eye: Vec3::new(0.0, -2.0, 0.0),
            target: Vec3::new(0.0, -2.0, 8.0),
            up: Vec3::NEG_Y,
            fov_y: 0.8,
        };
        let f = Frame { view_proj: cam.view_proj(1.0), eye: cam.eye, target: cam.target, fog: None, sun: None };
        let u = uniform(&f, 1);
        assert!((u[0] - 8.0).abs() < 1e-5 && u[1] == 12.0);
    }

    /// Stripes at the focus stay as they are; the same stripes far behind are averaged toward grey by the gather.
    #[test]
    fn near_sharp_far_soft() {
        let Ok(gpu) = Gpu::headless() else { return };
        let n = 32u32;
        let cam =
            crate::render::Camera { eye: Vec3::ZERO, target: Vec3::new(0.0, 0.0, 5.0), up: Vec3::NEG_Y, fov_y: 0.8 };
        let f = Frame { view_proj: cam.view_proj(1.0), eye: cam.eye, target: cam.target, fog: None, sun: None };
        let mut dof = Dof::new(&gpu, wgpu::TextureFormat::Rgba8Unorm);
        dof.resize(&gpu, [n, n]);
        dof.set_strength(2);
        // Four pixels wide: halving keeps them, so only the gather can soften them.
        let stripes = |x: u32, _| if (x / 4).is_multiple_of(2) { [255, 255, 255, 255] } else { [0, 0, 0, 255] };
        let mut spread = Vec::new();
        for dist in [5.0f32, 400.0] {
            let (tex, view) = filled(&gpu, [n, n], stripes);
            let depth = crate::render::depth_view(&gpu.device, n, n);
            let z = cam.view_proj(1.0).project_point3(Vec3::new(0.0, 0.0, dist)).z;
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
            dof.draw(&gpu, &mut enc, &view, &depth, &f);
            gpu.queue.submit([enc.finish()]);
            let px = read(&gpu, &tex);
            let row: Vec<i32> = (8..24).map(|x| px[(16 * n + x) as usize][0] as i32).collect();
            spread.push(row.iter().max().unwrap() - row.iter().min().unwrap());
        }
        assert_eq!(spread[0], 255, "in focus: untouched");
        assert!(spread[1] < 100, "far: softened ({spread:?})");
    }
}
