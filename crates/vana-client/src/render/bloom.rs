//! Bloom (ours, not retail; Settings > Scene > Bloom, config `bloom`): the bright parts of the scene glow. The scene is
//! the game's 8-bit, gamma-space picture (no HDR): a lamp at night is about as bright as sunlit rock at noon, so one
//! fixed threshold either misses the lamps or washes the day out. The threshold follows the neighbourhood instead: the
//! scene is first halved down the chain as it is, and the smallest level (1/64 of the scene's size), copied aside, is
//! the local mean; a pixel glows by how far its luma is over that mean times a ratio, held between a floor and a
//! ceiling (soft knee). Then the bright part is prefiltered into the half-size level (13 taps, each box weighted down by
//! its luma so a single bright pixel can't flicker), halved again up to five times with the same 13-tap filter, summed
//! back up the chain with a 3 x 3 tent, and the half-size sum added to the scene.

use super::Gpu;
use super::scenefx::{ADD, pass, pipeline, soft_format, target};

/// The chain's levels: the half-size one and up to this many less halvings after it.
const LEVELS: usize = 6;
/// The smallest level's least side in pixels.
const SMALLEST: u32 = 4;

/// (ratio, floor, ceiling, intensity, knee) for Subtle (1) and Strong (2): the threshold is the local mean luma times
/// the ratio, held within floor..ceiling (gamma-space luma 0..1), eased in over the knee; the intensity scales the
/// chain's sum (all levels added) as it goes onto the scene.
pub fn params(strength: u32) -> [f32; 5] {
    match strength {
        2 => [1.8, 0.28, 0.78, 1.4, 0.12],
        _ => [2.0, 0.3, 0.8, 0.8, 0.12],
    }
}

const WGSL: &str = r#"
struct P { ratio: f32, floor: f32, ceiling: f32, intensity: f32, knee: f32, pad0: f32, pad1: f32, pad2: f32 };
@group(0) @binding(0) var<uniform> p: P;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
// The local mean (the smallest level of the scene halved as it is).
@group(0) @binding(3) var mean: texture_2d<f32>;

fn texel() -> vec2<f32> { return 1.0 / vec2<f32>(textureDimensions(src)); }
fn tap(uv: vec2<f32>) -> vec3<f32> { return textureSampleLevel(src, samp, uv, 0.0).rgb; }
fn luma(c: vec3<f32>) -> f32 { return dot(c, vec3<f32>(0.299, 0.587, 0.114)); }

// What of c is over the threshold t, by its luma, eased in over the knee.
fn bright(c: vec3<f32>, t: f32) -> vec3<f32> {
    let l = luma(c);
    var soft = clamp(l - t + p.knee, 0.0, 2.0 * p.knee);
    soft = soft * soft / (4.0 * p.knee + 1e-4);
    return c * max(soft, l - t) / max(l, 1e-4);
}

// The 13 taps around uv in texels of src, as five 2 x 2 boxes: the inner one weighted 1/2, the four corner ones 1/8.
fn taps13(uv: vec2<f32>) -> array<vec3<f32>, 13> {
    let t = texel();
    return array<vec3<f32>, 13>(
        tap(uv + t * vec2<f32>(-2.0, -2.0)), tap(uv + t * vec2<f32>(0.0, -2.0)), tap(uv + t * vec2<f32>(2.0, -2.0)),
        tap(uv + t * vec2<f32>(-1.0, -1.0)), tap(uv + t * vec2<f32>(1.0, -1.0)),
        tap(uv + t * vec2<f32>(-2.0, 0.0)), tap(uv), tap(uv + t * vec2<f32>(2.0, 0.0)),
        tap(uv + t * vec2<f32>(-1.0, 1.0)), tap(uv + t * vec2<f32>(1.0, 1.0)),
        tap(uv + t * vec2<f32>(-2.0, 2.0)), tap(uv + t * vec2<f32>(0.0, 2.0)), tap(uv + t * vec2<f32>(2.0, 2.0)),
    );
}

// A box's average, weighted 1 / (1 + luma) with the others (Karis) so one hot pixel doesn't flicker.
fn karis(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, d: vec3<f32>) -> vec4<f32> {
    let m = (a + b + c + d) * 0.25;
    let w = 1.0 / (1.0 + luma(m));
    return vec4<f32>(m * w, w);
}

@fragment fn fs_prefilter(i: VOut) -> @location(0) vec4<f32> {
    var s = taps13(i.uv);
    let t = clamp(luma(textureSampleLevel(mean, samp, i.uv, 0.0).rgb) * p.ratio, p.floor, p.ceiling);
    for (var k = 0; k < 13; k++) { s[k] = bright(s[k], t); }
    let inner = karis(s[3], s[4], s[8], s[9]) * 0.5;
    let tl = karis(s[0], s[1], s[5], s[6]) * 0.125;
    let tr = karis(s[1], s[2], s[6], s[7]) * 0.125;
    let bl = karis(s[5], s[6], s[10], s[11]) * 0.125;
    let br = karis(s[6], s[7], s[11], s[12]) * 0.125;
    let sum = inner + tl + tr + bl + br;
    return vec4<f32>(sum.rgb / max(sum.w, 1e-4), 1.0);
}

@fragment fn fs_down(i: VOut) -> @location(0) vec4<f32> {
    let s = taps13(i.uv);
    let c = (s[3] + s[4] + s[8] + s[9]) * 0.125
        + (s[0] + s[1] + s[5] + s[6]) * 0.03125 + (s[1] + s[2] + s[6] + s[7]) * 0.03125
        + (s[5] + s[6] + s[10] + s[11]) * 0.03125 + (s[6] + s[7] + s[11] + s[12]) * 0.03125;
    return vec4<f32>(c, 1.0);
}

// The 3 x 3 tent of the smaller level, at its texel spacing.
fn tent(uv: vec2<f32>) -> vec3<f32> {
    let t = texel();
    var c = tap(uv) * 4.0;
    c += (tap(uv + vec2<f32>(t.x, 0.0)) + tap(uv - vec2<f32>(t.x, 0.0)) + tap(uv + vec2<f32>(0.0, t.y)) + tap(uv - vec2<f32>(0.0, t.y))) * 2.0;
    c += tap(uv + t) + tap(uv - t) + tap(uv + vec2<f32>(t.x, -t.y)) + tap(uv + vec2<f32>(-t.x, t.y));
    return c / 16.0;
}

@fragment fn fs_copy(i: VOut) -> @location(0) vec4<f32> { return vec4<f32>(tap(i.uv), 1.0); }

@fragment fn fs_up(i: VOut) -> @location(0) vec4<f32> { return vec4<f32>(tent(i.uv), 1.0); }

@fragment fn fs_apply(i: VOut) -> @location(0) vec4<f32> { return vec4<f32>(tent(i.uv) * p.intensity, 0.0); }
"#;

pub struct Bloom {
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform: wgpu::Buffer,
    prefilter: wgpu::RenderPipeline,
    down: wgpu::RenderPipeline,
    up: wgpu::RenderPipeline,
    apply: wgpu::RenderPipeline,
    copy: wgpu::RenderPipeline,
    /// The levels' format (render/scenefx.rs soft_format).
    format: wgpu::TextureFormat,
    /// The scene size the levels are for, the levels (half size first) and a group reading each.
    size: [u32; 2],
    levels: Vec<wgpu::TextureView>,
    groups: Vec<wgpu::BindGroup>,
    /// The local mean (the smallest level's size), the group copying the smallest level into it (bound with `blank`
    /// as its mean, since it writes `mean`) and a 1 x 1 stand-in.
    mean: Option<wgpu::TextureView>,
    copy_group: Option<wgpu::BindGroup>,
    blank: wgpu::TextureView,
    /// The prefilter's group, by the scene view it reads.
    scene_group: super::BindCache<wgpu::TextureView>,
    strength: u32,
}

impl Bloom {
    /// `scene`: the scene target's format.
    pub fn new(gpu: &Gpu, scene: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom"),
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom"),
            source: wgpu::ShaderSource::Wgsl(format!("{}{WGSL}", super::scenefx::VERTEX).into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bloom"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let format = soft_format(&gpu.adapter);
        let add_all = wgpu::BlendState {
            color: ADD.color,
            alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, ..ADD.alpha },
        };
        let pipe = |entry, format, blend| pipeline(device, "bloom", &pl, &module, entry, format, blend);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("bloom"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bloom"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            prefilter: pipe("fs_prefilter", format, None),
            down: pipe("fs_down", format, None),
            up: pipe("fs_up", format, Some(add_all)),
            apply: pipe("fs_apply", scene, Some(ADD)),
            copy: pipe("fs_copy", format, None),
            blank: target(device, "bloom", [1, 1], format),
            mean: None,
            copy_group: None,
            layout,
            sampler,
            uniform,
            format,
            size: [0, 0],
            levels: Vec::new(),
            groups: Vec::new(),
            scene_group: Default::default(),
            strength: 0,
        }
    }

    /// The levels' sizes for a scene of `size`: half of it, then halved while both sides stay at least SMALLEST.
    pub fn level_sizes(size: [u32; 2]) -> Vec<[u32; 2]> {
        let mut s = size.map(|v| v.div_ceil(2).max(1));
        let mut out = vec![s];
        while out.len() < LEVELS {
            s = s.map(|v| v.div_ceil(2));
            if s[0] < SMALLEST || s[1] < SMALLEST {
                break;
            }
            out.push(s);
        }
        out
    }

    /// The levels for a scene of `size`, made on first use or a size change.
    pub fn resize(&mut self, gpu: &Gpu, size: [u32; 2]) {
        if self.size == size && !self.levels.is_empty() {
            return;
        }
        self.size = size;
        let sizes = Self::level_sizes(size);
        self.levels = sizes.iter().map(|&s| target(&gpu.device, "bloom", s, self.format)).collect();
        let mean = target(&gpu.device, "bloom mean", sizes[sizes.len() - 1], self.format);
        self.groups = self.levels.iter().map(|v| self.group_on(&gpu.device, v, &mean)).collect();
        self.copy_group = Some(self.group_on(&gpu.device, &self.levels[self.levels.len() - 1], &self.blank));
        self.mean = Some(mean);
        self.scene_group.clear();
    }

    fn group_on(&self, device: &wgpu::Device, view: &wgpu::TextureView, mean: &wgpu::TextureView) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(mean) },
            ],
        })
    }

    /// Subtle (1) or Strong (2); the uniform is written only on a change.
    pub fn set_strength(&mut self, gpu: &Gpu, strength: u32) {
        if self.strength != strength {
            self.strength = strength;
            let [ratio, floor, ceiling, intensity, knee] = params(strength);
            let u = [ratio, floor, ceiling, intensity, knee, 0.0, 0.0, 0.0];
            super::Put::put(&gpu.queue, &self.uniform, 0, bytemuck::cast_slice(&u));
        }
    }

    /// The glow added to `scene` (the size given to `resize`).
    pub fn draw(&self, gpu: &Gpu, encoder: &mut wgpu::CommandEncoder, scene: &wgpu::TextureView) {
        let (Some(first), Some(mean), Some(copy)) = (self.levels.first(), &self.mean, &self.copy_group) else { return };
        let g = self.scene_group.get(scene.clone(), || self.group_on(&gpu.device, scene, mean));
        // The scene halved down the chain as it is; the smallest level is the local mean.
        pass(encoder, "bloom mean", first, false, &self.down, &g);
        for k in 1..self.levels.len() {
            pass(encoder, "bloom mean", &self.levels[k], false, &self.down, &self.groups[k - 1]);
        }
        pass(encoder, "bloom mean", mean, false, &self.copy, copy);
        // Then the bright part, by that mean.
        pass(encoder, "bloom prefilter", first, false, &self.prefilter, &g);
        for k in 1..self.levels.len() {
            pass(encoder, "bloom down", &self.levels[k], false, &self.down, &self.groups[k - 1]);
        }
        for k in (0..self.levels.len() - 1).rev() {
            pass(encoder, "bloom up", &self.levels[k], true, &self.up, &self.groups[k + 1]);
        }
        pass(encoder, "bloom apply", scene, true, &self.apply, &self.groups[0]);
    }
}

#[cfg(test)]
mod tests {
    use super::super::scenefx::test_util::{filled, read};
    use super::*;

    #[test]
    fn level_sizes_halve_down_to_the_smallest() {
        assert_eq!(
            Bloom::level_sizes([1600, 900]),
            vec![[800, 450], [400, 225], [200, 113], [100, 57], [50, 29], [25, 15]]
        );
        assert_eq!(Bloom::level_sizes([20, 10]), vec![[10, 5]]);
        assert_eq!(Bloom::level_sizes([1, 1]), vec![[1, 1]]);
    }

    /// A scene under the threshold is left exactly as it was; a bright square glows onto the dark pixels around it
    /// and the glow falls off with distance.
    #[test]
    fn dim_scene_untouched_bright_spot_glows() {
        let Ok(gpu) = Gpu::headless() else { return };
        let size = [64u32, 64];
        for strength in [1, 2] {
            let mut b = Bloom::new(&gpu, wgpu::TextureFormat::Rgba8Unorm);
            b.resize(&gpu, size);
            b.set_strength(&gpu, strength);
            let (tex, view) = filled(&gpu, size, |_, _| [90, 100, 110, 255]);
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            b.draw(&gpu, &mut enc, &view);
            gpu.queue.submit([enc.finish()]);
            assert!(read(&gpu, &tex).iter().all(|p| *p == [90, 100, 110, 255]), "strength {strength}");

            let spot = |x: u32, y: u32| (28..36).contains(&x) && (28..36).contains(&y);
            let (tex, view) = filled(&gpu, size, |x, y| if spot(x, y) { [255; 4] } else { [20, 20, 20, 255] });
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            b.draw(&gpu, &mut enc, &view);
            gpu.queue.submit([enc.finish()]);
            let px = read(&gpu, &tex);
            let at = |x: u32, y: u32| px[(y * size[0] + x) as usize];
            let (near, far, corner) = (at(38, 32)[0], at(46, 32)[0], at(0, 0)[0]);
            assert!(near > far && far >= corner && near > 30, "strength {strength}: {near} {far} {corner}");
            assert_eq!(at(0, 0)[3], 255, "alpha kept");
        }
    }
}
