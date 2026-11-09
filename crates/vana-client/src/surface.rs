//! Surface Detail's height pack (not retail; Settings' Surface Detail, render zone.wgsl `detail_normal_pack`).
//!
//! `tex-heights` writes one greyscale PNG per zone texture into a mod folder, at `surface/<zone>/<name>.png`
//! (`file_name`). Each is the texture's brightness with its painted shading taken out (a wide blur subtracted) and its
//! contrast evened out, so every texture gets about the same relief: retail's flat-painted textures get some, the
//! contrasty ones no more than the rest. The zones are read through the enabled mods, so an upscaled texture pack
//! gives heights at its own size. The PNGs can be edited by hand.
//!
//! At a zone's load with Surface Detail on, `load` finds the heights of the zone's textures in the enabled mods
//! (highest priority first) and turns each into slopes (`slopes`), which the renderer uploads with mips: one read a
//! pixel gives the slope averaged over the pixel's footprint. Textures without one keep the shader's estimate.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The pack's folder in a mod.
const DIR: &str = "surface";
/// The largest zone id looked at.
const LAST_ZONE: u32 = 1000;

/// A texture's file name in the pack: its name trimmed, bytes other than letters, digits, '_' and '-' as %xx.
pub fn file_name(texture: &str) -> String {
    let mut out = String::new();
    for b in texture.trim_end_matches([' ', '\0']).bytes() {
        if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02x}"));
        }
    }
    out + ".png"
}

/// Box blur of radius `r` in place, wrapping at the edges (the textures tile), along x then y. Linear in the texels.
fn blur(v: &mut [f32], w: usize, h: usize, r: usize) {
    if r == 0 || w == 0 || h == 0 {
        return;
    }
    let mut line = Vec::new();
    let mut pass = |v: &mut [f32], n: usize, lines: usize, at: &dyn Fn(usize, usize) -> usize| {
        let r = r.min(n / 2);
        let k = (2 * r + 1) as f32;
        for l in 0..lines {
            line.clear();
            line.extend((0..n).map(|i| v[at(l, i)]));
            let mut sum: f32 = (0..=2 * r).map(|j| line[(j + n - r) % n]).sum();
            for i in 0..n {
                v[at(l, i)] = sum / k;
                sum += line[(i + r + 1) % n] - line[(i + n - r) % n];
            }
        }
    };
    pass(v, w, h, &|l, i| l * w + i);
    pass(v, h, w, &|l, i| i * w + l);
}

/// A texture's heights (0..255, 128 level): brightness, its block noise softened by a blur of a 256th of the larger
/// side (so features keep their size on an upscaled copy), its painted shading removed by subtracting a blur of a
/// 16th of it (twice, for a smooth falloff), then scaled so 98% of texels lie within the range.
pub fn heights(w: u32, h: u32, rgba: &[u8]) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let n = w * h;
    if n == 0 || rgba.len() < n * 4 {
        return vec![128; n];
    }
    let mut lum: Vec<f32> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .take(n)
        .map(|p| (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0)
        .collect();
    let side = w.max(h);
    blur(&mut lum, w, h, side / 512);
    let mut base = lum.clone();
    blur(&mut base, w, h, (side / 32).max(1));
    blur(&mut base, w, h, (side / 32).max(1));
    let hp: Vec<f32> = lum.iter().zip(&base).map(|(a, b)| a - b).collect();
    let mut mags: Vec<f32> = hp.iter().map(|d| d.abs()).collect();
    let at = (mags.len() * 98 / 100).min(mags.len() - 1);
    let p98 = *mags.select_nth_unstable_by(at, f32::total_cmp).1;
    if p98 < 1e-4 {
        return vec![128; n];
    }
    hp.iter().map(|d| (128.0 + 115.0 * (d / p98).clamp(-1.1, 1.1)).round() as u8).collect()
}

/// Heights to slopes, RG8 (128 level): each texel's height change per texel along u and v (central differences,
/// wrapping), x2 and clamped, so a slope of +-0.25 a texel fills the range. The shader undoes it.
pub fn slopes(w: u32, h: u32, hts: &[u8]) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let mut out = Vec::with_capacity(w * h * 2);
    let at = |x: usize, y: usize| hts[y * w + x] as f32 / 255.0;
    for y in 0..h {
        for x in 0..w {
            let du = (at((x + 1) % w, y) - at((x + w - 1) % w, y)) * 0.5;
            let dv = (at(x, (y + 1) % h) - at(x, (y + h - 1) % h)) * 0.5;
            let enc = |s: f32| (128.0 + 255.0 * (s * 2.0).clamp(-0.5, 0.5)).round().clamp(0.0, 255.0) as u8;
            out.push(enc(du));
            out.push(enc(dv));
        }
    }
    out
}

/// One texture's slopes (`slopes`): width, height, RG8.
pub type Slopes = (u32, u32, Vec<u8>);

/// The pack's slopes for `textures` (names) of `zone` from the first of `mods` (highest priority first) that has
/// each, by name. Textures without a PNG, or with one that doesn't read as greyscale or RGBA, are left out.
pub fn load(zone: u32, textures: &[String], mods: &[PathBuf]) -> HashMap<String, Slopes> {
    let dirs: Vec<PathBuf> = mods.iter().map(|m| m.join(DIR).join(zone.to_string())).filter(|d| d.is_dir()).collect();
    let mut out = HashMap::new();
    if dirs.is_empty() {
        return out;
    }
    for t in textures {
        let f = file_name(t);
        let Some(path) = dirs.iter().map(|d| d.join(&f)).find(|p| p.is_file()) else { continue };
        match load_grey(&path) {
            Ok((w, h, hts)) => {
                out.insert(t.clone(), (w, h, slopes(w, h, &hts)));
            }
            Err(e) => log::warn!("surface detail {}: {e:#}", path.display()),
        }
    }
    out
}

fn load_grey(path: &Path) -> Result<(u32, u32, Vec<u8>)> {
    let mut d = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path)?));
    d.set_transformations(png::Transformations::normalize_to_color8());
    let mut r = d.read_info()?;
    let mut buf = vec![0u8; r.output_buffer_size().context("png size")?];
    let info = r.next_frame(&mut buf)?;
    buf.truncate(info.buffer_size());
    let step = match info.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        c => anyhow::bail!("colour type {c:?}"),
    };
    let n = (info.width * info.height) as usize;
    anyhow::ensure!(buf.len() >= n * step, "short image");
    Ok((info.width, info.height, buf.chunks_exact(step).take(n).map(|p| p[0]).collect()))
}

fn save_grey(path: &Path, w: u32, h: u32, v: &[u8]) -> Result<()> {
    let tmp = path.with_extension("part");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(&tmp)?), w, h);
    enc.set_color(png::ColorType::Grayscale);
    enc.set_compression(png::Compression::Fast);
    enc.write_header()?.write_image_data(v)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// `tex-heights`: every zone's textures (through the enabled mods) as height PNGs under `out`/surface/<zone>/, or only
/// `zones`. Existing PNGs are kept unless `redo` (hand edits survive a rerun).
pub fn pack(out: &Path, zones: Option<&[u32]>, redo: bool) -> Result<()> {
    let cfg = crate::config::Config::load();
    let index = crate::dat::DatIndex::open(&cfg.ffxi_dir())?;
    let keys = crate::dat::zone::Keys::from_ffximain(&std::fs::read(crate::dat::ffximain(&cfg.ffxi_dir()))?)?;
    let list: Vec<u32> = zones.map_or_else(|| (0..=LAST_ZONE).collect(), |z| z.to_vec());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let written = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, 12) {
            s.spawn(|| {
                loop {
                    let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(&zone) = list.get(k) else { break };
                    let Ok(file) = index.read(crate::dat::zone::file_id(zone)) else { continue };
                    let Ok(scene) = crate::dat::zone::load_scene(&keys, &file) else { continue };
                    if scene.textures.is_empty() {
                        continue;
                    }
                    let dir = out.join(DIR).join(zone.to_string());
                    if std::fs::create_dir_all(&dir).is_err() {
                        continue;
                    }
                    let mut n = 0;
                    for t in &scene.textures {
                        let path = dir.join(file_name(&t.name));
                        if path.exists() && !redo {
                            continue;
                        }
                        match save_grey(&path, t.width, t.height, &heights(t.width, t.height, t.rgba())) {
                            Ok(()) => n += 1,
                            Err(e) => eprintln!("zone {zone} {}: {e:#}", t.name.trim()),
                        }
                    }
                    written.fetch_add(n, std::sync::atomic::Ordering::Relaxed);
                    println!("zone {zone}: {n} height maps");
                }
            });
        }
    });
    println!("{} height maps in {}", written.into_inner(), out.join(DIR).display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_escape_odd_bytes() {
        assert_eq!(file_name("tex_wall 01   "), "tex_wall%2001.png");
        assert_eq!(file_name("a/b\0\0"), "a%2fb.png");
    }

    #[test]
    fn heights_drop_shading_and_even_contrast() {
        // A faint stripe pattern on a dark-to-light ramp (painted shading): the ramp goes, the stripes fill the range.
        let (w, h) = (64u32, 64u32);
        let faint: Vec<u8> = (0..w * h)
            .flat_map(|i| {
                let (x, _) = (i % w, i / w);
                let v = (x * 2 + if x % 8 < 4 { 6 } else { 0 }) as u8;
                [v, v, v, 255]
            })
            .collect();
        let hts = heights(w, h, &faint);
        let (lo, hi) = (*hts.iter().min().unwrap(), *hts.iter().max().unwrap());
        assert!(hi - lo > 150, "stripes stretched over the range: {lo}..{hi}");
        // A flat texture: level.
        assert!(heights(8, 8, &[90u8; 8 * 8 * 4]).iter().all(|&v| v == 128));
    }

    #[test]
    fn slopes_follow_the_height() {
        // Heights rising 4 a texel along u, level along v (wrapping only at the ends).
        let (w, h) = (8u32, 2u32);
        let hts: Vec<u8> = (0..w * h).map(|i| 100 + 4 * (i % w) as u8).collect();
        let s = slopes(w, h, &hts);
        let mid = (3 * 2) as usize;
        let du = (s[mid] as f32 - 128.0) / 255.0 / 2.0;
        assert!((du - 4.0 / 255.0).abs() < 0.003, "{du}");
        assert_eq!(s[mid + 1], 128);
    }
}
