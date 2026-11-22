//! Scene effects (ours, not retail; Settings > Scene, each one Off by default): full-screen passes over the finished
//! 3D scene at its own size, after the actors and the effect elements and before the mods' post passes, the screen
//! fades, the scaling pass and the 2D layer. One file each (render/bloom.rs, ...); this one holds what they share and
//! keeps only the ones turned on: an effect that is off has no pass, no target and no pipeline.
//!
//! They use plain render passes on Rgba8 / Rgba16Float targets (no compute, no storage buffers) and read depth as an
//! unfilterable float, so they run on the GL / GLES backend too.

use super::Gpu;
use super::bloom::Bloom;

/// What the settings turn on, each 0 = off (retail). `bloom`: config `bloom`, 1 Subtle, 2 Strong.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Want {
    pub bloom: u32,
}

impl Want {
    pub fn from_config(c: &crate::config::Config) -> Self {
        Self { bloom: c.bloom }
    }

    /// zone-shot's switches: FFXI_BLOOM=1|2.
    pub fn from_env() -> Self {
        let n = |k: &str| std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        Self { bloom: n("FFXI_BLOOM") }
    }

    /// Whether any effect is on (the stage is made only then).
    pub fn any(&self) -> bool {
        self.bloom != 0
    }
}

pub struct SceneFx {
    /// The scene target's format (the passes that write it are made for it).
    format: wgpu::TextureFormat,
    bloom: Option<Bloom>,
}

impl SceneFx {
    pub fn new(format: wgpu::TextureFormat) -> Self {
        Self { format, bloom: None }
    }

    /// Makes what `want` turns on for a scene of `size` (made on first use or a size change) and lets go of what it
    /// turns off.
    pub fn prepare(&mut self, gpu: &Gpu, want: Want, size: [u32; 2]) {
        if want.bloom == 0 {
            self.bloom = None;
        } else {
            let b = self.bloom.get_or_insert_with(|| Bloom::new(gpu, self.format));
            b.resize(gpu, size);
            b.set_strength(gpu, want.bloom);
        }
    }

    /// The effects turned on, over `scene` (the plain, not sRGB, view of the scene target, the size `prepare` was
    /// given).
    pub fn draw(&self, gpu: &Gpu, encoder: &mut wgpu::CommandEncoder, scene: &wgpu::TextureView) {
        if let Some(b) = &self.bloom {
            b.draw(gpu, encoder, scene);
        }
    }
}

/// The full-screen triangle every pass draws (`uv` 0..1, top left 0).
pub(super) const VERTEX: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) i: u32) -> VOut {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var o: VOut;
    o.pos = vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
    o.uv = vec2<f32>(p.x, 1.0 - p.y);
    return o;
}
"#;

/// Colour added to what is there, alpha kept.
pub(super) const ADD: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    },
    alpha: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    },
};

/// The intermediate targets' format: Rgba16Float where the device renders, filters and blends it (no banding in the
/// soft gradients), else Rgba8Unorm, which every backend can.
pub(super) fn soft_format(adapter: &wgpu::Adapter) -> wgpu::TextureFormat {
    use wgpu::TextureFormatFeatureFlags as F;
    let f = wgpu::TextureFormat::Rgba16Float;
    let ff = adapter.get_texture_format_features(f);
    let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
    if ff.allowed_usages.contains(usage) && ff.flags.contains(F::FILTERABLE | F::BLENDABLE) {
        f
    } else {
        wgpu::TextureFormat::Rgba8Unorm
    }
}

/// A full-screen pipeline of `module`'s `vs` and `entry`.
pub(super) fn pipeline(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    entry: &str,
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(entry),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState { format, blend, write_mask: wgpu::ColorWrites::ALL })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

/// A target the passes render into and then sample.
pub(super) fn target(
    device: &wgpu::Device,
    label: &str,
    size: [u32; 2],
    format: wgpu::TextureFormat,
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width: size[0].max(1), height: size[1].max(1), depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

/// One full-screen draw into `view`: cleared first (`load` None) or drawn over.
pub(super) fn pass(
    encoder: &mut wgpu::CommandEncoder,
    label: &str,
    view: &wgpu::TextureView,
    keep: bool,
    pipeline: &wgpu::RenderPipeline,
    group: &wgpu::BindGroup,
) {
    let load = if keep { wgpu::LoadOp::Load } else { wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT) };
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, group, &[]);
    pass.draw(0..3, 0..1);
}

#[cfg(test)]
pub(super) mod test_util {
    use super::Gpu;

    /// An Rgba8Unorm scene target filled with `rgba`, and its view.
    pub fn filled(gpu: &Gpu, size: [u32; 2], rgba: impl Fn(u32, u32) -> [u8; 4]) -> (wgpu::Texture, wgpu::TextureView) {
        let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let data: Vec<u8> =
            (0..size[1]).flat_map(|y| (0..size[0]).map(move |x| (x, y))).flat_map(|(x, y)| rgba(x, y)).collect();
        gpu.queue.write_texture(
            tex.as_image_copy(),
            &data,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(size[0] * 4), rows_per_image: Some(size[1]) },
            wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
        );
        let view = tex.create_view(&Default::default());
        (tex, view)
    }

    /// The texture's pixels, row by row.
    pub fn read(gpu: &Gpu, tex: &wgpu::Texture) -> Vec<[u8; 4]> {
        let (w, h) = (tex.width(), tex.height());
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
        (0..h).flat_map(|y| d[(y * row) as usize..(y * row + w * 4) as usize].as_chunks::<4>().0.to_vec()).collect()
    }
}
