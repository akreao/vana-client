//! NVIDIA DLSS Super Resolution (config `upscaler` 4, not retail): the temporal upscaler of NVIDIA RTX cards, through
//! the NGX SDK over Vulkan (crate dlss_wgpu). Fed the inputs FSR 3.1 takes: the linear colour (so DLSS's HDR mode:
//! its LDR mode wants sRGB-encoded colour and bands on linear), depth, render-size motion vectors without the
//! jitter, the jitter, reset. Output linear at the output size, as FSR 3.1's, for the scaling pass to grade and encode.
//!
//! Built with the cargo feature `dlss` only (the NGX static library comes from NVIDIA's DLSS SDK, DLSS_SDK at build
//! time). On an NVIDIA GPU on the Vulkan backend the device is made with NGX's extensions (`device`); every other
//! GPU, GL / GLES and builds without the feature take the usual device and never see it. The DLSS runtime
//! (nvngx_dlss.dll / libnvidia-ngx-dlss.so.310.9.1, about 59 MB) is not shipped: the first time DLSS is chosen it is
//! downloaded from NVIDIA's DLSS repository on GitHub at the SDK's tag, checked against its SHA-256, into the data
//! folder (`runtime_dir`), and NGX starts once it is there.

/// Config `upscaler`'s value for DLSS. 1 FSR 3.1, 2 the external engine, 3 FSR 1.
pub const DLSS: u32 = 4;

/// NVIDIA's PCI vendor id.
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
pub const NVIDIA: u32 = 0x10de;

/// Whether DLSS can be offered on this device (an NVIDIA GPU on Vulkan with NGX's extensions), else why not. Set when
/// the device is made.
static STATUS: std::sync::Mutex<Option<Result<(), String>>> = std::sync::Mutex::new(None);

pub fn set_status(s: Result<(), String>) {
    if let Ok(mut n) = STATUS.lock() {
        *n = Some(s);
    }
}

pub fn status() -> Result<(), String> {
    if !cfg!(feature = "dlss") {
        return Err("this build has no DLSS".into());
    }
    STATUS.lock().ok().and_then(|n| n.clone()).unwrap_or_else(|| Err("no device yet".into()))
}

/// NVIDIA's notices for DLSS and the third-party code in its runtime (docs/notices/NVIDIA-DLSS.txt), written beside
/// the downloaded runtime.
#[cfg(feature = "dlss")]
const NOTICES: &str = include_str!("../../../../../docs/notices/NVIDIA-DLSS.txt");

/// An output size and quality mode NGX's render range left out (dispatch): DLSS is not used for that pair.
static REJECTED: std::sync::Mutex<Option<([u32; 2], usize)>> = std::sync::Mutex::new(None);

#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
fn reject(output: [u32; 2], mode: usize) {
    if let Ok(mut r) = REJECTED.lock() {
        *r = Some((output, mode));
    }
}

/// Whether the frame upscales with DLSS at window size `output` and `upscale_quality` `quality`: NGX runs here, and
/// that pair was not left out of its render range.
pub fn usable(output: [u32; 2], quality: u32) -> bool {
    status().is_ok() && REJECTED.lock().map_or(true, |r| *r != Some((output, quality.min(4) as usize)))
}

/// The credit the title screen shows above the version where DLSS can run (a `dlss` build on an NVIDIA RTX GPU), as
/// the SDK licence asks of applications using DLSS; None elsewhere, so other builds and GPUs keep the retail screen.
pub fn credit() -> Option<&'static str> {
    status().is_ok().then_some("Uses NVIDIA DLSS")
}

/// The DLSS SDK release whose runtime is fetched (the one dlss_wgpu 6.0 and its NGX library are made for).
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
const SDK_TAG: &str = "v310.9.1";

/// The runtime for this platform: its folder in the SDK's layout, its file name and SHA-256 (of the SDK at SDK_TAG).
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
fn runtime_file() -> (&'static str, &'static str, &'static str) {
    if cfg!(windows) {
        ("Windows_x86_64", "nvngx_dlss.dll", "3975567b8943c53acce397f2b72380092f84f162d00b0d2c7d08a1025c563983")
    } else {
        (
            "Linux_x86_64",
            "libnvidia-ngx-dlss.so.310.9.1",
            "7561f16cab74e2b7ccf791bf44bb9cc43abb44bcf78f33c5663520ad4e857e02",
        )
    }
}

/// The runtime's size in bytes at SDK_TAG: files of another size are not hashed in the search for a copy.
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
fn runtime_size() -> u64 {
    if cfg!(windows) { 58_956_912 } else { 59_481_944 }
}

/// Folders other software keeps game files in, where a copy of the same runtime may already sit: Steam's libraries
/// (from its libraryfolders.vdf), Epic's and GOG's game folders, NVIDIA's NGX update cache (its files carry other
/// names), and Heroic / Lutris's ~/Games on Linux. Each is (folder, any file name counts).
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
fn search_roots() -> Vec<(std::path::PathBuf, bool)> {
    use std::path::PathBuf;
    let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
    let home = dirs::home_dir();
    let mut steam: Vec<PathBuf> = if cfg!(windows) {
        [env("ProgramFiles(x86)"), env("ProgramFiles")].into_iter().flatten().map(|p| p.join("Steam")).collect()
    } else {
        home.iter()
            .flat_map(|h| {
                [".steam/steam", ".local/share/Steam", ".var/app/com.valvesoftware.Steam/.local/share/Steam"]
                    .map(|r| h.join(r))
            })
            .collect()
    };
    // Each Steam install's other libraries ("path" lines of steamapps/libraryfolders.vdf).
    let mut libs = Vec::new();
    for s in &steam {
        if let Ok(t) = std::fs::read_to_string(s.join("steamapps").join("libraryfolders.vdf")) {
            for l in t.lines() {
                let mut q = l.split('"').filter(|x| !x.trim().is_empty());
                if q.next() == Some("path")
                    && let Some(p) = q.next()
                {
                    libs.push(PathBuf::from(p.replace("\\\\", "\\")));
                }
            }
        }
    }
    steam.extend(libs);
    let mut roots: Vec<(PathBuf, bool)> = steam.iter().map(|s| (s.join("steamapps").join("common"), false)).collect();
    if cfg!(windows) {
        roots.extend(env("ProgramFiles").map(|p| (p.join("Epic Games"), false)));
        roots.extend(env("ProgramFiles(x86)").map(|p| (p.join("GOG Galaxy").join("Games"), false)));
        roots.extend(env("ProgramData").map(|p| (p.join("NVIDIA").join("NGX").join("models"), true)));
    } else {
        roots.extend(home.iter().map(|h| (h.join("Games"), false)));
    }
    roots.sort();
    roots.dedup();
    roots
}

/// The first file under `roots` (at most `depth` folders down, symlinks not followed, at most 500,000 entries in all; Unreal games keep it about 8 down)
/// named `name` (any name under a root marked so), of `size` bytes, whose SHA-256 is `sha`. Only reads.
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
fn find_copy(
    roots: &[(std::path::PathBuf, bool)],
    name: &str,
    size: u64,
    sha: &str,
    depth: usize,
) -> Option<std::path::PathBuf> {
    let mut budget = 500_000usize;
    for (root, any_name) in roots {
        let mut stack = vec![(root.clone(), 0usize)];
        while let Some((dir, d)) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            for e in rd.flatten() {
                budget = budget.checked_sub(1)?;
                let Ok(ft) = e.file_type() else { continue };
                let path = e.path();
                if ft.is_dir() {
                    if d < depth {
                        stack.push((path, d + 1));
                    }
                } else if ft.is_file()
                    && (*any_name || e.file_name().to_str().is_some_and(|n| n.eq_ignore_ascii_case(name)))
                    && e.metadata().is_ok_and(|m| m.len() == size)
                    && std::fs::read(&path)
                        .is_ok_and(|b| hex::encode(ring::digest::digest(&ring::digest::SHA256, &b)) == sha)
                {
                    return Some(path);
                }
            }
        }
    }
    None
}

/// The folder NGX searches for the runtime: DLSS_SDK's `lib/<platform>/rel` (dlss_wgpu hands NGX only that path). The
/// client sets DLSS_SDK to `<data folder>/dlss` at start when it is unset (`point_ngx_at_data`), so the downloaded
/// runtime goes there; a build run against a real SDK keeps that SDK's.
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
pub fn runtime_dir() -> std::path::PathBuf {
    let base = std::env::var_os("DLSS_SDK").map_or_else(|| crate::config::data_dir().join("dlss"), Into::into);
    base.join("lib").join(runtime_file().0).join("rel")
}

/// At the very start of main, before any other thread: DLSS_SDK, when unset, names `<data folder>/dlss`, where the
/// runtime is downloaded to. A `dlss` build only.
pub fn point_ngx_at_data() {
    if cfg!(feature = "dlss") && std::env::var_os("DLSS_SDK").is_none() {
        // Safety: called first thing in main, while the process has one thread.
        unsafe { std::env::set_var("DLSS_SDK", crate::config::data_dir().join("dlss")) };
    }
}

/// Whether the runtime is in place (beside the executable, where NGX also looks, or in `runtime_dir`).
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
pub fn runtime_present() -> bool {
    let name = runtime_file().1;
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.join(name)));
    exe_dir.is_some_and(|p| p.is_file()) || runtime_dir().join(name).is_file()
}

/// The runtime download: not started, running, done, or failed (why). One try a run.
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
#[derive(Clone, Debug, PartialEq)]
pub enum Fetch {
    Idle,
    Running,
    Done,
    Failed(String),
}

#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
static FETCH: std::sync::Mutex<Fetch> = std::sync::Mutex::new(Fetch::Idle);

/// Where the runtime download stands.
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
pub fn fetch_state() -> Fetch {
    FETCH.lock().map(|f| f.clone()).unwrap_or(Fetch::Idle)
}

/// Starts the runtime download on a thread of its own, unless it already ran this session.
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
fn start_fetch() {
    let Ok(mut f) = FETCH.lock() else { return };
    if *f != Fetch::Idle {
        return;
    }
    *f = Fetch::Running;
    drop(f);
    std::thread::spawn(|| {
        let r = fetch_runtime();
        match &r {
            Ok(()) => log::info!("DLSS: runtime downloaded to {}", runtime_dir().display()),
            Err(e) => log::warn!("DLSS: runtime download failed: {e:#}"),
        }
        if let Ok(mut f) = FETCH.lock() {
            *f = r.map_or_else(|e| Fetch::Failed(format!("{e:#}")), |()| Fetch::Done);
        }
    });
}

/// Puts the runtime in `runtime_dir`: copied from a copy already on this computer with the SDK's exact SHA-256 (the
/// same file, so the same version; found by `find_copy`, left where it is), else downloaded from NVIDIA's DLSS
/// repository at SDK_TAG and kept only when its SHA-256 is the SDK's.
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
fn fetch_runtime() -> anyhow::Result<()> {
    let (plat, name, sha) = runtime_file();
    let dir = runtime_dir();
    std::fs::create_dir_all(&dir)?;
    let tmp = dir.join(format!("{name}.download"));
    match find_copy(&search_roots(), name, runtime_size(), sha, 9) {
        Some(found) => {
            log::info!("DLSS: the runtime is already at {}; copying it", found.display());
            std::fs::copy(&found, &tmp)?;
        }
        None => {
            let url = format!("https://raw.githubusercontent.com/NVIDIA/DLSS/{SDK_TAG}/lib/{plat}/rel/{name}");
            log::info!("DLSS: downloading the runtime from {url}");
            crate::client::updater::download(&url, &tmp, &|_, _| {})?;
        }
    }
    let got = hex::encode(ring::digest::digest(&ring::digest::SHA256, &std::fs::read(&tmp)?));
    if got != sha {
        let _ = std::fs::remove_file(&tmp);
        anyhow::bail!("the runtime's SHA-256 is {got}, expected {sha}");
    }
    // The notices first: the runtime counts as installed once it is in place, and comes with them.
    #[cfg(feature = "dlss")]
    std::fs::write(dir.join("NVIDIA-DLSS-NOTICES.txt"), NOTICES)?;
    std::fs::rename(&tmp, dir.join(name))?;
    Ok(())
}

/// The quality mode for the configured ratio (`upscale_quality` 0..4: Native AA (DLAA), Quality, Balanced,
/// Performance, Ultra Performance).
#[cfg_attr(not(feature = "dlss"), allow(dead_code))]
pub fn mode_of(render: [u32; 2], output: [u32; 2]) -> usize {
    let r = output[0] as f32 / render[0].max(1) as f32;
    let near = |i: &usize| ((super::RATIOS[*i] - r).abs() * 1000.0) as u32;
    (0..super::RATIOS.len()).min_by_key(near).unwrap_or(1)
}

#[cfg(not(feature = "dlss"))]
pub fn device(_adapter: &wgpu::Adapter, _desc: &wgpu::DeviceDescriptor) -> Option<(wgpu::Device, wgpu::Queue)> {
    set_status(Err("this build has no DLSS".into()));
    None
}

#[cfg(not(feature = "dlss"))]
pub fn new(_gpu: &crate::render::Gpu) -> Result<Box<dyn super::Upscaler>, String> {
    Err("this build has no DLSS".into())
}

#[cfg(feature = "dlss")]
pub use imp::{device, new};

#[cfg(feature = "dlss")]
mod imp {
    use super::super::{Inputs, Upscaler};
    use super::{NVIDIA, set_status};
    use crate::render::Gpu;
    use dlss_wgpu::super_resolution::{
        DlssSuperResolution, DlssSuperResolutionExposure, DlssSuperResolutionRenderParameters,
    };
    use dlss_wgpu::{DlssFeatureFlags, DlssPerfQualityMode, DlssSdk, FeatureSupport};
    use std::sync::{Arc, Mutex};

    /// The application id NGX is started with (a project id of our own; NVIDIA hands out ids only to titles that
    /// ask for one).
    const PROJECT: uuid::Uuid = uuid::uuid!("5b8f2a1e-6c0d-4b7e-9f3a-2d71c4e8a960");

    /// The SDK once started (on the device, when the runtime is there).
    static SDK: Mutex<Option<Arc<Mutex<DlssSdk>>>> = Mutex::new(None);

    /// The device with NGX's Vulkan extensions when the adapter is an NVIDIA GPU on Vulkan; None (with the status
    /// saying why) for the usual device. NGX itself starts when the runtime is there (`sdk`).
    pub fn device(adapter: &wgpu::Adapter, desc: &wgpu::DeviceDescriptor) -> Option<(wgpu::Device, wgpu::Queue)> {
        let info = adapter.get_info();
        if info.backend != wgpu::Backend::Vulkan {
            set_status(Err("DLSS needs the Vulkan backend".into()));
            return None;
        }
        if info.vendor != NVIDIA {
            set_status(Err("DLSS needs an NVIDIA RTX graphics card".into()));
            return None;
        }
        // NGX's calls are not thread-safe (parallel devices, as the tests make, double-freed inside it): one at a time.
        static NGX: Mutex<()> = Mutex::new(());
        let _one = NGX.lock();
        let mut support = FeatureSupport::default();
        let made = dlss_wgpu::request_device(PROJECT, adapter, desc, &mut support, Some(desc.required_limits.clone()));
        let (device, queue) = match made {
            Ok(d) => d,
            Err(e) => {
                log::warn!("DLSS: the device with NGX's extensions failed ({e}); the usual device instead");
                set_status(Err(format!("{e}")));
                return None;
            }
        };
        if !support.super_resolution_supported {
            set_status(Err("the GPU or driver lacks DLSS's Vulkan extensions".into()));
        } else if super::runtime_present() {
            set_status(sdk(&device).map(|_| ()));
        } else {
            set_status(Ok(()));
        }
        Some((device, queue))
    }

    /// NGX, started on first need once the runtime is there. Err: it can't run here (not an RTX card, an old driver).
    fn sdk(device: &wgpu::Device) -> Result<Arc<Mutex<DlssSdk>>, String> {
        let mut s = SDK.lock().map_err(|e| e.to_string())?;
        if let Some(sdk) = s.as_ref() {
            return Ok(sdk.clone());
        }
        let sdk = DlssSdk::new(PROJECT, device.clone()).map_err(|e| {
            log::info!("DLSS unavailable: {e}");
            format!("{e}")
        })?;
        log::info!("DLSS: NGX started");
        *s = Some(sdk.clone());
        Ok(sdk)
    }

    const MODES: [DlssPerfQualityMode; 5] = [
        DlssPerfQualityMode::Dlaa,
        DlssPerfQualityMode::Quality,
        DlssPerfQualityMode::Balanced,
        DlssPerfQualityMode::Performance,
        DlssPerfQualityMode::UltraPerformance,
    ];

    /// The DLSS feature for one output size and mode, its output and the exposure (1.0: the colour is the scene's
    /// own linear light, nothing to expose).
    struct Made {
        output: [u32; 2],
        mode: usize,
        sr: DlssSuperResolution,
        out: wgpu::Texture,
    }

    pub struct Dlss {
        /// NGX, once the runtime is there and it started; Err once it failed (not tried again this run).
        sdk: Option<Result<Arc<Mutex<DlssSdk>>, String>>,
        made: Option<Made>,
        exposure: wgpu::Texture,
        /// One texel the output is copied into after NGX (see dispatch).
        touch: wgpu::Texture,
    }

    pub fn new(gpu: &Gpu) -> Result<Box<dyn Upscaler>, String> {
        super::status()?;
        let exposure = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("dlss exposure"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        gpu.queue.write_texture(
            exposure.as_image_copy(),
            &1.0f32.to_le_bytes(),
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: None },
            exposure.size(),
        );
        let touch = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("dlss touch"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: super::super::COLOR_FORMAT,
            usage: wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        Ok(Box::new(Dlss { sdk: None, made: None, exposure, touch }))
    }

    impl Upscaler for Dlss {
        fn name(&self) -> &'static str {
            "DLSS"
        }

        fn dispatch(
            &mut self,
            gpu: &Gpu,
            encoder: &mut wgpu::CommandEncoder,
            inp: &Inputs,
        ) -> Option<wgpu::TextureView> {
            // Until the runtime is downloaded and NGX started, the frame is scaled plainly.
            if self.sdk.is_none() {
                if !super::runtime_present() {
                    super::start_fetch();
                    return None;
                }
                let started = sdk(&gpu.device);
                // NGX refused (not an RTX card, an old driver): Settings stops offering DLSS.
                if let Err(e) = &started {
                    set_status(Err(e.clone()));
                }
                self.sdk = Some(started);
            }
            let sdk = self.sdk.as_ref()?.as_ref().ok()?.clone();
            let mode = super::mode_of(inp.render, inp.output);
            if self.made.as_ref().is_none_or(|m| m.output != inp.output || m.mode != mode) {
                // The old feature goes first (its drop waits for the device to idle).
                self.made = None;
                let flags = DlssFeatureFlags::HighDynamicRange | DlssFeatureFlags::LowResolutionMotionVectors;
                let sr = DlssSuperResolution::new(inp.output, MODES[mode], flags, sdk, &gpu.device, &gpu.queue)
                    .inspect_err(|e| log::warn!("DLSS: creating the feature at {:?}: {e}", inp.output))
                    .ok()?;
                let range = sr.render_resolution_range();
                let (lo, hi) = (*range.start(), *range.end());
                if (0..2).any(|i| inp.render[i] < lo[i] || inp.render[i] > hi[i]) {
                    // Outside what NGX takes for this mode at this output size: the world draws at full size until
                    // either changes (`usable`); the other sizes and modes stay offered.
                    log::warn!("DLSS: render size {:?} outside NGX's {lo:?}..={hi:?}", inp.render);
                    super::reject(inp.output, mode);
                    return None;
                }
                log::info!(
                    "DLSS {:?} at {}x{}: render {}x{} (DLSS takes {:?}..={:?})",
                    MODES[mode],
                    inp.output[0],
                    inp.output[1],
                    inp.render[0],
                    inp.render[1],
                    range.start(),
                    range.end()
                );
                let out = gpu.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("dlss output"),
                    size: wgpu::Extent3d { width: inp.output[0], height: inp.output[1], depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: super::super::COLOR_FORMAT,
                    usage: wgpu::TextureUsages::STORAGE_BINDING
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                // DLSS writes the output outside wgpu's sight: cleared once here, so wgpu counts it as written and
                // never zero-fills it before a read.
                encoder.clear_texture(&out, &wgpu::ImageSubresourceRange::default());
                self.made = Some(Made { output: inp.output, mode, sr, out });
            }
            let m = self.made.as_mut()?;
            let v = |t: &wgpu::Texture| t.create_view(&Default::default());
            let depth = inp.depth.create_view(&wgpu::TextureViewDescriptor {
                aspect: wgpu::TextureAspect::DepthOnly,
                ..Default::default()
            });
            let (color, motion, exposure, out) = (v(inp.color), v(inp.motion), v(&self.exposure), v(&m.out));
            let params = DlssSuperResolutionRenderParameters {
                color: &color,
                depth: &depth,
                motion_vectors: &motion,
                exposure: DlssSuperResolutionExposure::Manual {
                    exposure: &exposure,
                    exposure_scale: Some(1.0),
                    pre_exposure: Some(1.0),
                },
                bias: None,
                dlss_output: &out,
                reset: inp.reset,
                jitter_offset: inp.jitter,
                partial_texture_size: Some(inp.render),
                motion_vector_scale: None,
            };
            let cb =
                m.sr.render(params, encoder, &gpu.adapter).inspect_err(|e| log::warn!("DLSS: evaluate: {e}")).ok()?;
            // DLSS's commands go in the same submit, right after the frame so far; the frame goes on in a fresh
            // encoder.
            let prior = std::mem::replace(encoder, gpu.device.create_command_encoder(&Default::default()));
            gpu.queue.submit([prior.finish(), cb]);
            // wgpu sees only the textures (in the barriers), not these views: they live until the GPU is done with
            // DLSS's commands, or wgpu would destroy them under NGX.
            let keep = (color, motion, depth, exposure, out.clone());
            gpu.queue.on_submitted_work_done(move || drop(keep));
            // A transfer use of the output between NGX and the scaling pass: without it the pass reads black on
            // NVIDIA's Linux driver (620.73; a full memory barrier after NGX's commands does not help, a copy out of the
            // output does). One texel into a scratch texture.
            encoder.copy_texture_to_texture(
                m.out.as_image_copy(),
                self.touch.as_image_copy(),
                wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            );
            Some(out)
        }
    }
}

#[cfg(test)]
mod tests {
    /// The quality mode comes back from the render size the configured quality gives, at common window sizes.
    #[test]
    fn mode_from_render_size() {
        for out in [[1280, 720], [1600, 900], [1920, 1080], [2560, 1440], [3840, 2160], [1024, 768]] {
            for q in 0..5 {
                assert_eq!(super::mode_of(super::super::render_size(out, q), out), q as usize, "{out:?} quality {q}");
            }
        }
    }

    /// The search finds a file of the right name, size and hash a few folders down, skips one whose bytes differ, and
    /// takes any name under a root marked so (NGX's cache).
    #[test]
    fn finds_a_matching_copy() {
        let base = std::env::temp_dir().join(format!("vana-dlss-find-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let good = b"runtime bytes".to_vec();
        let sha = hex::encode(ring::digest::digest(&ring::digest::SHA256, &good));
        let put = |rel: &str, b: &[u8]| {
            let p = base.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, b).unwrap();
            p
        };
        put("steam/GameA/bin/nvngx_dlss.dll", b"other bytes!!");
        let want = put("steam/GameB/x64/bin/NVNGX_DLSS.DLL", &good);
        let cached = put("ngx/dlss/versions/1/files/160_x.bin", &good);
        let n = good.len() as u64;
        let roots = |r: &[(&str, bool)]| r.iter().map(|(p, a)| (base.join(p), *a)).collect::<Vec<_>>();
        assert_eq!(super::find_copy(&roots(&[("steam", false)]), "nvngx_dlss.dll", n, &sha, 6), Some(want));
        assert_eq!(super::find_copy(&roots(&[("steam", false)]), "nvngx_dlss.dll", n, &sha, 1), None);
        assert_eq!(super::find_copy(&roots(&[("ngx", true)]), "nvngx_dlss.dll", n, &sha, 6), Some(cached));
        assert_eq!(super::find_copy(&roots(&[("ngx", false)]), "nvngx_dlss.dll", n, &sha, 6), None);
        let _ = std::fs::remove_dir_all(&base);
    }

    /// A size NGX's range left out stops DLSS for that window size and mode only: another size, or the same size in
    /// another mode, is still used, and going back to a good size works again.
    #[test]
    fn range_rejection_is_per_size_and_mode() {
        super::set_status(Ok(()));
        let ok = |o, q| !cfg!(feature = "dlss") || super::usable(o, q);
        super::reject([640, 360], 4);
        assert!(!super::usable([640, 360], 4));
        assert!(ok([640, 360], 1));
        assert!(ok([1600, 900], 4));
        super::reject([700, 400], 4);
        assert!(ok([640, 360], 4));
    }
}
