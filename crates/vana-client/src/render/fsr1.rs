//! AMD FidelityFX Super Resolution 1 (not retail): the spatial upscaler for devices the temporal one cannot run on
//! (OpenGL / GLES, small GPUs). The world draws at a lower render size; EASU (edge-adaptive spatial upsampling)
//! takes it to the window's size here, and RCAS (robust contrast-adaptive sharpening) runs in the scaling pass
//! (render/scale.rs) on the way to the window. Both are plain fragment passes with texel loads only (no compute,
//! no storage, no gathers), so they run on GLES 3.0.
//!
//! The shaders are a WGSL port of `ffxFsrEasuFloat` and `FsrRcasF` from AMD's ffx_fsr1.h (FidelityFX SDK, MIT
//! licence, copyright Advanced Micro Devices, Inc.): the 32-bit paths, the gathers written as the twelve loads
//! they stand for, and the approximate reciprocals as exact ones with the divide-by-zero cases guarded.
//! The input is the gamma-space scene, which is what FSR 1 expects (perceptual, after tonemapping).

use super::Gpu;
use super::Put;

/// Config `upscaler`'s value for FSR 1. 1 and 2 are the temporal upscaler's (FSR 3.1, the external engine).
pub const FSR1: u32 = 3;

/// The output-to-render ratio of each quality mode (config `upscale_quality`): 0 Native (unused by FSR 1),
/// 1 Quality, 2 Balanced, 3 Performance, 4 Ultra Performance. AMD's FSR 1 ratios.
pub const RATIOS: [f32; 5] = [1.0, 1.5, 1.7, 2.0, 3.0];

/// The size the world draws at for an `output` (window) size and a quality mode.
pub fn render_size(output: [u32; 2], quality: u32) -> [u32; 2] {
    let r = RATIOS[quality.min(4) as usize];
    [((output[0] as f32 / r) as u32).max(1), ((output[1] as f32 / r) as u32).max(1)]
}

/// RCAS's sharpness for config `upscale_sharpness` (0 off .. 100): FsrRcasCon's linear value, exp2(-stops), with
/// 100 % = 0 stops (the most) and 1 % about 2 stops. 0 when off (the pass is skipped).
pub fn rcas_sharpness(percent: u32) -> f32 {
    if percent == 0 {
        return 0.0;
    }
    let s = percent.min(100) as f32 / 100.0;
    (-(2.0 - 2.0 * s)).exp2()
}

/// FsrEasuCon's first constant: the output pixel to the input position, (in / out, 0.5 * in / out - 0.5) per axis.
/// The others only place the gathers, which the loads do not need.
pub fn easu_constants(input: [u32; 2], output: [u32; 2]) -> [f32; 4] {
    let kx = input[0] as f32 / output[0] as f32;
    let ky = input[1] as f32 / output[1] as f32;
    [kx, ky, 0.5 * kx - 0.5, 0.5 * ky - 0.5]
}

const EASU: &str = r#"
struct E { con0: vec4<f32> };
@group(0) @binding(0) var<uniform> e: E;
@group(0) @binding(1) var src: texture_2d<f32>;
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}
// One texel, clamped to the edge (as the gathers' clamp sampler).
fn ld(p: vec2<i32>) -> vec3<f32> {
    let hi = vec2<i32>(textureDimensions(src)) - 1;
    return textureLoad(src, clamp(p, vec2<i32>(0), hi), 0).rgb;
}
// Luma times 2.
fn l2(c: vec3<f32>) -> f32 { return c.b * 0.5 + (c.r * 0.5 + c.g); }
fn rcp(x: f32) -> f32 { return 1.0 / max(x, 1e-20); }
// fsrEasuSetFloat: the '+' around c, weighted by w (the bilinear weight of this quad corner).
fn easu_set(dir: ptr<function, vec2<f32>>, len: ptr<function, f32>, w: f32, la: f32, lb: f32, lc: f32, ld_: f32, le: f32) {
    let dc = ld_ - lc;
    let cb = lc - lb;
    let dx = ld_ - lb;
    var lx = clamp(abs(dx) * rcp(max(abs(dc), abs(cb))), 0.0, 1.0);
    lx = lx * lx;
    let ec = le - lc;
    let ca = lc - la;
    let dy = le - la;
    var ly = clamp(abs(dy) * rcp(max(abs(ec), abs(ca))), 0.0, 1.0);
    ly = ly * ly;
    *dir = *dir + vec2<f32>(dx, dy) * w;
    *len = *len + (lx + ly) * w;
}
// fsrEasuTapFloat: one tap of the rotated, stretched lanczos-2 approximation.
fn easu_tap(ac: ptr<function, vec3<f32>>, aw: ptr<function, f32>, off: vec2<f32>, dir: vec2<f32>, len2: vec2<f32>, lob: f32, clp: f32, c: vec3<f32>) {
    var v = vec2<f32>(off.x * dir.x + off.y * dir.y, off.x * -dir.y + off.y * dir.x);
    v = v * len2;
    let d2 = min(v.x * v.x + v.y * v.y, clp);
    var wb = 2.0 / 5.0 * d2 - 1.0;
    var wa = lob * d2 - 1.0;
    wb = wb * wb;
    wa = wa * wa;
    wb = 25.0 / 16.0 * wb - (25.0 / 16.0 - 1.0);
    let w = wb * wa;
    *ac = *ac + c * w;
    *aw = *aw + w;
}
@fragment fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    var pp = floor(pos.xy) * e.con0.xy + e.con0.zw;
    let fp = floor(pp);
    pp = pp - fp;
    let f0 = vec2<i32>(fp);
    //    b c
    //  e f g h
    //  i j k l
    //    n o
    let b = ld(f0 + vec2<i32>(0, -1));
    let c = ld(f0 + vec2<i32>(1, -1));
    let ee = ld(f0 + vec2<i32>(-1, 0));
    let f = ld(f0);
    let g = ld(f0 + vec2<i32>(1, 0));
    let h = ld(f0 + vec2<i32>(2, 0));
    let i = ld(f0 + vec2<i32>(-1, 1));
    let j = ld(f0 + vec2<i32>(0, 1));
    let k = ld(f0 + vec2<i32>(1, 1));
    let l = ld(f0 + vec2<i32>(2, 1));
    let n = ld(f0 + vec2<i32>(0, 2));
    let o = ld(f0 + vec2<i32>(1, 2));
    let bL = l2(b);
    let cL = l2(c);
    let eL = l2(ee);
    let fL = l2(f);
    let gL = l2(g);
    let hL = l2(h);
    let iL = l2(i);
    let jL = l2(j);
    let kL = l2(k);
    let lL = l2(l);
    let nL = l2(n);
    let oL = l2(o);
    var dir = vec2<f32>(0.0);
    var len = 0.0;
    easu_set(&dir, &len, (1.0 - pp.x) * (1.0 - pp.y), bL, eL, fL, gL, jL);
    easu_set(&dir, &len, pp.x * (1.0 - pp.y), cL, fL, gL, hL, kL);
    easu_set(&dir, &len, (1.0 - pp.x) * pp.y, fL, iL, jL, kL, nL);
    easu_set(&dir, &len, pp.x * pp.y, gL, jL, kL, lL, oL);
    // Normalise, with near-zero directions as (1, 0).
    let dir2 = dir * dir;
    var dirR = dir2.x + dir2.y;
    let zro = dirR < 1.0 / 32768.0;
    dirR = select(inverseSqrt(max(dirR, 1e-20)), 1.0, zro);
    dir.x = select(dir.x, 1.0, zro);
    dir = dir * dirR;
    len = len * 0.5;
    len = len * len;
    let stretch = (dir.x * dir.x + dir.y * dir.y) * rcp(max(abs(dir.x), abs(dir.y)));
    let len2 = vec2<f32>(1.0 + (stretch - 1.0) * len, 1.0 - 0.5 * len);
    let lob = 0.5 + ((1.0 / 4.0 - 0.04) - 0.5) * len;
    let clp = 1.0 / lob;
    let mn4 = min(min(min(f, g), j), k);
    let mx4 = max(max(max(f, g), j), k);
    var ac = vec3<f32>(0.0);
    var aw = 0.0;
    easu_tap(&ac, &aw, vec2<f32>(0.0, -1.0) - pp, dir, len2, lob, clp, b);
    easu_tap(&ac, &aw, vec2<f32>(1.0, -1.0) - pp, dir, len2, lob, clp, c);
    easu_tap(&ac, &aw, vec2<f32>(-1.0, 1.0) - pp, dir, len2, lob, clp, i);
    easu_tap(&ac, &aw, vec2<f32>(0.0, 1.0) - pp, dir, len2, lob, clp, j);
    easu_tap(&ac, &aw, vec2<f32>(0.0, 0.0) - pp, dir, len2, lob, clp, f);
    easu_tap(&ac, &aw, vec2<f32>(-1.0, 0.0) - pp, dir, len2, lob, clp, ee);
    easu_tap(&ac, &aw, vec2<f32>(1.0, 1.0) - pp, dir, len2, lob, clp, k);
    easu_tap(&ac, &aw, vec2<f32>(2.0, 1.0) - pp, dir, len2, lob, clp, l);
    easu_tap(&ac, &aw, vec2<f32>(2.0, 0.0) - pp, dir, len2, lob, clp, h);
    easu_tap(&ac, &aw, vec2<f32>(1.0, 0.0) - pp, dir, len2, lob, clp, g);
    easu_tap(&ac, &aw, vec2<f32>(1.0, 2.0) - pp, dir, len2, lob, clp, o);
    easu_tap(&ac, &aw, vec2<f32>(0.0, 2.0) - pp, dir, len2, lob, clp, n);
    // Normalise and dering against the four nearest.
    return vec4<f32>(min(mx4, max(mn4, ac / aw)), 1.0);
}
"#;

/// RCAS for the scaling pass's shader (render/scale.rs), which binds the EASU output as `tex` and gives the
/// sharpness in `u.fsr.x`: FsrRcasF without the noise term (FSR_RCAS_DENOISE off, AMD's default).
pub const RCAS: &str = r#"
fn rcas_ld(p: vec2<i32>) -> vec3<f32> {
    let hi = vec2<i32>(textureDimensions(tex)) - 1;
    return textureLoad(tex, clamp(p, vec2<i32>(0), hi), 0).rgb;
}
//    b
//  d e f
//    h
fn rcas(p: vec2<i32>, sharp: f32) -> vec3<f32> {
    let b = rcas_ld(p + vec2<i32>(0, -1));
    let d = rcas_ld(p + vec2<i32>(-1, 0));
    let e = rcas_ld(p);
    let f = rcas_ld(p + vec2<i32>(1, 0));
    let h = rcas_ld(p + vec2<i32>(0, 1));
    let mn4 = min(min(min(b, d), f), h);
    let mx4 = max(max(max(b, d), f), h);
    // The lobe that takes the ring to 0 or to 1, whichever comes first; the divides guarded at black and white.
    let hit_min = mn4 / max(4.0 * mx4, vec3<f32>(1e-20));
    let hit_max = (1.0 - mx4) / min(4.0 * mn4 - 4.0, vec3<f32>(-1e-20));
    let lobe3 = max(-hit_min, hit_max);
    let lobe = max(-(0.25 - 1.0 / 16.0), min(max(max(lobe3.r, lobe3.g), lobe3.b), 0.0)) * sharp;
    return (lobe * (b + d + h + f) + e) / (4.0 * lobe + 1.0);
}
"#;

struct Target {
    size: [u32; 2],
    view: wgpu::TextureView,
}

/// The EASU pass and its window-size target.
pub struct Easu {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    format: wgpu::TextureFormat,
    target: Option<Target>,
    /// The group over the source view, by that view.
    group: super::BindCache<wgpu::TextureView>,
}

impl Easu {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fsr1 easu"),
            source: wgpu::ShaderSource::Wgsl(EASU.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fsr1 easu"),
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
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fsr1 easu"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fsr1 easu"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fsr1 easu"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self { pipeline, layout, uniform, format, target: None, group: Default::default() }
    }

    /// Upscales `src` (at `src_size`) to `dst_size`; the result's view, at `dst_size`. The target is made on first
    /// use and on a size change.
    pub fn run(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        src: &wgpu::TextureView,
        src_size: [u32; 2],
        dst_size: [u32; 2],
    ) -> wgpu::TextureView {
        if self.target.as_ref().is_none_or(|t| t.size != dst_size) {
            let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fsr1 easu"),
                size: wgpu::Extent3d { width: dst_size[0], height: dst_size[1], depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            self.target = Some(Target { size: dst_size, view: tex.create_view(&Default::default()) });
        }
        gpu.queue.put(&self.uniform, 0, bytemuck::cast_slice(&easu_constants(src_size, dst_size)));
        let group = self.group.get(src.clone(), || {
            gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fsr1 easu"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(src) },
                ],
            })
        });
        let out = self.target.as_ref().unwrap().view.clone();
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fsr1 easu"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &out,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AMD's ratios: 1920x1080 draws at 1280x720 (Quality), 1129x635 (Balanced), 960x540 (Performance), 640x360
    /// (Ultra Performance).
    #[test]
    fn render_sizes() {
        assert_eq!(render_size([1920, 1080], 1), [1280, 720]);
        assert_eq!(render_size([1920, 1080], 2), [1129, 635]);
        assert_eq!(render_size([1920, 1080], 3), [960, 540]);
        assert_eq!(render_size([1920, 1080], 4), [640, 360]);
        assert_eq!(render_size([1, 1], 4), [1, 1]);
    }

    /// FsrRcasCon: 100 % is 0 stops (1.0), 50 % one stop (0.5); off is 0.
    #[test]
    fn sharpness() {
        assert_eq!(rcas_sharpness(0), 0.0);
        assert_eq!(rcas_sharpness(100), 1.0);
        assert!((rcas_sharpness(50) - 0.5).abs() < 1e-6);
        assert!(rcas_sharpness(1) > 0.25 && rcas_sharpness(1) < 0.26);
    }

    /// FsrEasuCon's con0 at 2x: output pixel 0 maps to input -0.25 (between texels 0 and -1, nearer 0) and
    /// pixel 1 to 0.25.
    #[test]
    fn easu_con0() {
        let c = easu_constants([960, 540], [1920, 1080]);
        assert_eq!(c, [0.5, 0.5, -0.25, -0.25]);
    }
}
