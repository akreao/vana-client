//! Upscale Grain (ours, not retail): where an upscale pack's copy of a texture lost the stock image's fine grain (the
//! 4x upscale smears noisy ground such as Mhaura's sand into a blur), the stock grain goes back on top of the HD copy,
//! and into its material maps (material.rs) as fine relief and roughness, so those squares match their neighbours.
//! Offered only while Vana Upscale is enabled and the material pack is open; runs on the loader thread after
//! `material::attach`, so the maps are found by the HD copy's own key.

use crate::dat::zone::Texture;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};

/// The upscale pack's mod folder name (catalogue/dats.json).
pub const UPSCALE_MOD: &str = "vana-upscale";

static LEVEL: AtomicU32 = AtomicU32::new(0);
static UPSCALE: AtomicBool = AtomicBool::new(false);

/// Config `upscale_grain`: 0 off, 1..3 Low / Medium / High.
pub fn set_level(level: u32) {
    LEVEL.store(level.min(3), Relaxed);
}

/// Whether the enabled mods include the upscale pack (set wherever the mod list is scanned).
pub fn set_upscale(on: bool) {
    UPSCALE.store(on, Relaxed);
}

/// The setting exists: the upscale pack is enabled and the material pack is open.
pub fn offered() -> bool {
    UPSCALE.load(Relaxed) && crate::material::available()
}

fn strength() -> f32 {
    match LEVEL.load(Relaxed) {
        0 => 0.0,
        1 => 0.5,
        2 => 0.8,
        _ => 1.0,
    }
}

/// Puts the grain back into `textures` (file `id`'s, as read with the mods) from the stock copy of the same file.
pub fn apply(index: &crate::dat::DatIndex, id: u32, textures: &mut [Texture]) {
    let k = strength();
    if k == 0.0 || !offered() {
        return;
    }
    let Ok(bytes) = index.read_stock(id) else { return };
    let Ok(chunks) = crate::dat::chunks(&bytes) else { return };
    let stock: Vec<Texture> =
        chunks.iter().filter(|c| c.kind == 0x20).filter_map(|c| crate::dat::zone::parse_img(c.data).ok()).collect();
    let t0 = std::time::Instant::now();
    // World textures only (those the material pack covers): sky, weather and effect images are left alone.
    let mut jobs: Vec<(&mut Texture, &Texture)> = textures
        .iter_mut()
        .filter(|t| t.material.is_some())
        .filter_map(|t| {
            let s = stock.iter().find(|s| {
                s.name == t.name
                    && s.width > 0
                    && t.width > s.width
                    && t.width % s.width == 0
                    && t.height * s.width == s.height * t.width
            })?;
            Some((t, s))
        })
        .collect();
    let n = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, 6);
    let per = jobs.len().div_ceil(threads).max(1);
    std::thread::scope(|sc| {
        for part in jobs.chunks_mut(per) {
            let n = &n;
            sc.spawn(move || {
                for (t, s) in part {
                    if blend(t, s, k) {
                        n.fetch_add(1, Relaxed);
                    }
                }
            });
        }
    });
    let n = n.into_inner();
    if n > 0 {
        log::info!("upscale grain: {n} of {} textures in {:.0} ms", textures.len(), t0.elapsed().as_secs_f64() * 1e3);
    }
}

/// A plane of floats, w x h.
struct Plane {
    w: usize,
    h: usize,
    v: Vec<f32>,
}

impl Plane {
    /// Three box passes of radius r (about a gaussian), wrapping.
    fn blur(&self, r: usize) -> Plane {
        let (w, h) = (self.w, self.h);
        let mut cur = self.v.clone();
        let mut line = Vec::new();
        for _ in 0..3 {
            for y in 0..h {
                line.clear();
                line.extend_from_slice(&cur[y * w..(y + 1) * w]);
                box_line(&line, r, |x, v| cur[y * w + x] = v);
            }
            for x in 0..w {
                line.clear();
                line.extend((0..h).map(|y| cur[y * w + x]));
                box_line(&line, r, |y, v| cur[y * w + x] = v);
            }
        }
        Plane { w, h, v: cur }
    }
}

/// A wrapping box mean of radius r along `line`, written through `out`.
fn box_line(line: &[f32], r: usize, mut out: impl FnMut(usize, f32)) {
    let n = line.len() as isize;
    let at = |i: isize| line[i.rem_euclid(n) as usize];
    let r = r as isize;
    let d = (2 * r + 1) as f32;
    let mut s: f32 = (-r..=r).map(at).sum();
    for x in 0..n {
        out(x as usize, s / d);
        s += at(x + r + 1) - at(x - r);
    }
}

/// Bilinear taps from `n` texels to `m` (texel centres, wrapping): (first, second, weight of the second).
fn taps(n: usize, m: usize) -> Vec<(usize, usize, f32)> {
    let f = m as f32 / n as f32;
    (0..m)
        .map(|x| {
            let p = (x as f32 + 0.5) / f - 0.5;
            let p0 = p.floor();
            let i = (p0 as isize).rem_euclid(n as isize) as usize;
            (i, (i + 1) % n, p - p0)
        })
        .collect()
}

/// Planes of one size scaled up to `w` x `h` a row at a time: rows widened once, then mixed per output row.
struct Up {
    wide: Vec<Vec<f32>>,
    rows: Vec<(usize, usize, f32)>,
    w: usize,
}

impl Up {
    fn new(planes: &[&Plane], w: usize, h: usize) -> Up {
        let cols = taps(planes[0].w, w);
        let wide = planes
            .iter()
            .map(|p| {
                let mut out = vec![0f32; p.h * w];
                for y in 0..p.h {
                    let row = &p.v[y * p.w..(y + 1) * p.w];
                    for (x, &(a, b, t)) in cols.iter().enumerate() {
                        out[y * w + x] = row[a] + (row[b] - row[a]) * t;
                    }
                }
                out
            })
            .collect();
        Up { wide, rows: taps(planes[0].h, h), w }
    }

    /// Plane `k`'s output row `y` into `out`.
    fn row(&self, k: usize, y: usize, out: &mut [f32]) {
        let (a, b, t) = self.rows[y];
        let (ra, rb) = (&self.wide[k][a * self.w..(a + 1) * self.w], &self.wide[k][b * self.w..(b + 1) * self.w]);
        for x in 0..self.w {
            out[x] = ra[x] + (rb[x] - ra[x]) * t;
        }
    }
}

fn channels(rgba: &[u8], w: usize, h: usize) -> [Plane; 3] {
    std::array::from_fn(|c| Plane { w, h, v: rgba.as_chunks::<4>().0.iter().map(|p| p[c] as f32).collect() })
}

/// The grain weight at stock size: 1 where the HD copy kept none of the stock image's fine detail, 0 where it kept it
/// all; and the stock grain (the image less its blur), per channel.
fn grain(t: &Texture, s: &Texture) -> (Plane, [Plane; 3]) {
    let (sw, sh) = (s.width as usize, s.height as usize);
    let f = t.width as usize / sw;
    let sc = channels(s.rgba(), sw, sh);
    let g: [Plane; 3] = std::array::from_fn(|c| {
        let b = sc[c].blur(1);
        Plane { w: sw, h: sh, v: sc[c].v.iter().zip(&b.v).map(|(a, b)| a - b).collect() }
    });
    let gl = Plane { w: sw, h: sh, v: (0..sw * sh).map(|i| (g[0].v[i] + g[1].v[i] + g[2].v[i]).abs() / 3.0).collect() };
    // The HD copy's detail at the same scale: its average over each stock texel, less that image's blur.
    let (tw, th) = (t.width as usize, t.height as usize);
    let fy = th / sh;
    let mut small = Plane { w: sw, h: sh, v: vec![0.0; sw * sh] };
    for (i, p) in t.rgba().as_chunks::<4>().0.iter().enumerate() {
        let (x, y) = (i % tw / f, i / tw / fy);
        small.v[y * sw + x] += (p[0] as f32 + p[1] as f32 + p[2] as f32) / (3 * f * fy) as f32;
    }
    let sb = small.blur(1);
    let hd = Plane { w: sw, h: sh, v: small.v.iter().zip(&sb.v).map(|(a, b)| (a - b).abs()).collect() };
    let (eg, eh) = (gl.blur(4), hd.blur(4));
    let w = Plane {
        w: sw,
        h: sh,
        v: eg.v.iter().zip(&eh.v).map(|(g, h)| (1.0 - h / g.max(0.5)).clamp(0.0, 1.0)).collect(),
    };
    (w, g)
}

/// Blends `t` from `s` (its stock copy); false when the HD copy kept its grain (nothing changed).
fn blend(t: &mut Texture, s: &Texture, k: f32) -> bool {
    let (w, g) = grain(t, s);
    if w.v.iter().sum::<f32>() / (w.v.len() as f32) < 0.05 {
        return false;
    }
    let (tw, th) = (t.width as usize, t.height as usize);
    let up = Up::new(&[&w, &g[0], &g[1], &g[2]], tw, th);
    let mut rows = vec![vec![0f32; tw]; 4];
    let px = t.rgba_mut();
    for y in 0..th {
        for (j, r) in rows.iter_mut().enumerate() {
            up.row(j, y, r);
        }
        for x in 0..tw {
            let a = rows[0][x] * k;
            if a <= 0.0 {
                continue;
            }
            let p = &mut px[(y * tw + x) * 4..][..3];
            for c in 0..3 {
                p[c] = (p[c] as f32 + a * rows[c + 1][x]).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    // The GPU takes the RGBA now (level0 picks it when there are no blocks).
    t.dxt = None;
    if let Some(m) = t.material.take() {
        t.material = Some(material(&m, &w, &g, k));
    }
    true
}

/// The maps with the grain as fine relief (whiteout-blended into the normal map) and roughness.
fn material(m: &crate::material::Material, w: &Plane, g: &[Plane; 3], k: f32) -> Arc<crate::material::Material> {
    let (mw, mh) = (m.width as usize, m.height as usize);
    let size = texpresso::Format::Bc5.compressed_size(mw, mh);
    let (mut n, mut r) = (vec![0u8; mw * mh * 4], vec![0u8; mw * mh * 4]);
    texpresso::Format::Bc5.decompress(&m.normal[..size], mw, mh, &mut n);
    texpresso::Format::Bc5.decompress(&m.rough_metal[..size], mw, mh, &mut r);
    let gl = Plane {
        w: g[0].w,
        h: g[0].h,
        v: (0..g[0].v.len()).map(|i| (g[0].v[i] + g[1].v[i] + g[2].v[i]) / 765.0).collect(),
    };
    let up = Up::new(&[&gl, w], mw, mh);
    let (mut hgt, mut wt) = (vec![0f32; mw * mh], vec![0f32; mw * mh]);
    for y in 0..mh {
        up.row(0, y, &mut hgt[y * mw..(y + 1) * mw]);
        up.row(1, y, &mut wt[y * mw..(y + 1) * mw]);
    }
    wt.iter_mut().for_each(|a| *a *= k);
    let mean = hgt.iter().map(|h| h.abs()).sum::<f32>() / hgt.len() as f32;
    let h_at = |x: usize, y: usize| hgt[(y % mh) * mw + (x % mw)];
    let mut nrgba = vec![0u8; mw * mh * 4];
    let mut rrgba = vec![0u8; mw * mh * 4];
    for y in 0..mh {
        for x in 0..mw {
            let i = y * mw + x;
            let a = wt[i];
            let (nx, ny) = (n[i * 4] as f32 / 127.5 - 1.0, n[i * 4 + 1] as f32 / 127.5 - 1.0);
            let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
            // DirectX maps: y runs down the image.
            let dx = (h_at(x + 1, y) - h_at(x + mw - 1, y)) * 4.0 * a;
            let dy = (h_at(x, y + 1) - h_at(x, y + mh - 1)) * 4.0 * a;
            let gl = (dx * dx + dy * dy + 1.0).sqrt();
            let (gx, gy, gz) = (-dx / gl, -dy / gl, 1.0 / gl);
            let (bx, by, bz) = (nx + gx, ny + gy, nz * gz);
            let l = (bx * bx + by * by + bz * bz).sqrt().max(1e-6);
            nrgba[i * 4..i * 4 + 4].copy_from_slice(&[
                ((bx / l * 0.5 + 0.5) * 255.0).round() as u8,
                ((by / l * 0.5 + 0.5) * 255.0).round() as u8,
                ((bz / l * 0.5 + 0.5) * 255.0).round() as u8,
                255,
            ]);
            let rough = r[i * 4] as f32 / 255.0 + (hgt[i].abs() - mean) * 3.0 * a;
            rrgba[i * 4..i * 4 + 4].copy_from_slice(&[
                (rough.clamp(0.0, 1.0) * 255.0).round() as u8,
                r[i * 4 + 1],
                0,
                255,
            ]);
        }
    }
    Arc::new(crate::material::Material {
        width: m.width,
        height: m.height,
        levels: m.levels,
        normal: crate::texpack::bc5_chain(m.width, m.height, &nrgba, m.levels, true),
        rough_metal: crate::texpack::bc5_chain(m.width, m.height, &rrgba, m.levels, false),
        gpu: Default::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blur_keeps_a_flat_plane() {
        let p = Plane { w: 8, h: 4, v: vec![3.0; 32] };
        assert!(p.blur(2).v.iter().all(|v| (v - 3.0).abs() < 1e-4));
    }

    #[test]
    fn blur_keeps_the_sum() {
        let mut v = vec![0.0; 64];
        v[9] = 64.0;
        let p = Plane { w: 8, h: 8, v };
        assert!((p.blur(1).v.iter().sum::<f32>() - 64.0).abs() < 1e-3);
    }

    /// A noisy stock image and an HD copy that is the stock image's blur: the grain goes back, close to the stock
    /// noise; an HD copy that kept the noise is left as it is.
    #[test]
    fn blurred_copy_gets_its_grain_back() {
        let (sw, f) = (32usize, 4usize);
        let mut seed = 1u32;
        let mut rnd = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 24) as u8
        };
        let stock: Vec<u8> = (0..sw * sw)
            .flat_map(|_| {
                let v = 96 + rnd() / 4;
                [v, v, v, 255]
            })
            .collect();
        let s = Texture::new("t".into(), 0xa1, sw as u32, sw as u32, stock.clone());
        // Sharp copy: each stock texel as f x f.
        let hw = sw * f;
        let sharp: Vec<u8> = (0..hw * hw)
            .flat_map(|i| {
                let (x, y) = (i % hw / f, i / hw / f);
                let o = (y * sw + x) * 4;
                [stock[o], stock[o + 1], stock[o + 2], 255]
            })
            .collect();
        let mut kept = Texture::new("t".into(), 0xa1, hw as u32, hw as u32, sharp);
        assert!(!blend(&mut kept, &s, 1.0));
        let flat: Vec<u8> = (0..hw * hw).flat_map(|_| [112u8, 112, 112, 255]).collect();
        let mut smeared = Texture::new("t".into(), 0xa1, hw as u32, hw as u32, flat);
        assert!(blend(&mut smeared, &s, 1.0));
        let spread = |p: &[u8]| {
            let v: Vec<f32> = p.as_chunks::<4>().0.iter().map(|p| p[0] as f32).collect();
            let m = v.iter().sum::<f32>() / v.len() as f32;
            (v.iter().map(|x| (x - m).powi(2)).sum::<f32>() / v.len() as f32).sqrt()
        };
        assert!(spread(smeared.rgba()) > 0.5 * spread(&stock), "{} vs {}", spread(smeared.rgba()), spread(&stock));
    }
}
