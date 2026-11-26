//! DAT overlays (not retail: XIPivot's job built in). Mods are folders laid out like the game (ROM*/<dir>/<file>.DAT and
//! game-root files), listed highest priority first. An overlaid DAT is rebuilt from the current stock file: its
//! textures (D3s), menu sprite sheets (Msb) and bump maps (Bp) come from the highest-priority mod that has them, and
//! so does each spell record (mgc_) a mod changed; every other chunk stays stock, so SE's updates survive packs made
//! years ago. A file that is not a chunk container is replaced whole. Merged files are cached, keyed by every source's
//! size and time.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Chunk kinds taken from mods (names from FFXiMain's table at 0x103542a8).
const MERGED: [u8; 3] = [0x20, 0x31, 0x5d];
/// File 81's spell records (mgc_, see `actions`): merged record by record, the index being the spell id. Retail reads
/// a spell's jobs and levels from here alone (FUN_1022cde0), so a server that changes them ships this file (XIPivot
/// replaces it whole); a record is taken where the mod's differs from stock, so the spells SE adds or changes since
/// stay stock.
const SPELLS: u8 = 0x49;
const SPELL_SIZE: usize = 100;

pub struct Overlays {
    /// File id -> the mods' copies, highest priority first.
    by_id: HashMap<u32, Vec<PathBuf>>,
    dirs: Vec<PathBuf>,
    cache: PathBuf,
}

impl Overlays {
    /// Every ROM*/<dir>/<file>.DAT of the mods that the stock tables know (`ids`: (rom, dir << 7 | file) -> id).
    pub fn scan(dirs: &[PathBuf], ids: &HashMap<(u8, u16), u32>, cache: PathBuf) -> Self {
        let mut by_id: HashMap<u32, Vec<PathBuf>> = HashMap::new();
        let mut unknown = 0;
        for dir in dirs {
            let Ok(roms) = std::fs::read_dir(dir) else { continue };
            for rom in roms.flatten() {
                let name = rom.file_name().to_string_lossy().to_ascii_uppercase();
                let Some(n) =
                    name.strip_prefix("ROM").and_then(|s| if s.is_empty() { Some(1u8) } else { s.parse().ok() })
                else {
                    continue;
                };
                let Ok(subs) = std::fs::read_dir(rom.path()) else { continue };
                for sub in subs.flatten() {
                    let Some(d) = sub.file_name().to_str().and_then(|s| s.parse::<u16>().ok()) else { continue };
                    let Ok(files) = std::fs::read_dir(sub.path()) else { continue };
                    for f in files.flatten() {
                        let fname = f.file_name().to_string_lossy().to_ascii_uppercase();
                        let Some(file) = fname.strip_suffix(".DAT").and_then(|s| s.parse::<u16>().ok()) else {
                            continue;
                        };
                        match ids.get(&(n, d << 7 | file)) {
                            Some(&id) => by_id.entry(id).or_default().push(f.path()),
                            None => unknown += 1,
                        }
                    }
                }
            }
        }
        log::info!(
            "overlays: {} mods, {} files overlaid, {unknown} files at paths the game has no id for",
            dirs.len(),
            by_id.len()
        );
        Self { by_id, dirs: dirs.to_vec(), cache }
    }

    pub fn has(&self, id: u32) -> bool {
        self.by_id.contains_key(&id)
    }

    /// A game-root file (cursors and the like) from the highest-priority mod that has it.
    pub fn root_file(&self, name: &str) -> Option<PathBuf> {
        self.dirs.iter().map(|d| d.join(name)).find(|p| p.is_file())
    }

    /// The file to read for `id`: a merged build in the cache, a mod's whole file, or None (stock).
    pub fn resolve(&self, id: u32, stock: &Path) -> Option<PathBuf> {
        let mods = self.by_id.get(&id)?;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for p in std::iter::once(stock).chain(mods.iter().map(PathBuf::as_path)) {
            p.hash(&mut h);
            let m = std::fs::metadata(p).ok()?;
            (m.len(), m.modified().ok()).hash(&mut h);
        }
        let stem = format!("{id}-{:016x}", h.finish());
        let out = self.cache.join(format!("{stem}.DAT"));
        if out.is_file() {
            return Some(out);
        }
        let stock_bytes = std::fs::read(stock).ok()?;
        let mod_bytes: Vec<Vec<u8>> = mods.iter().filter_map(|p| std::fs::read(p).ok()).collect();
        let Some(merged) = merge(&stock_bytes, &mod_bytes) else {
            // Not chunk containers: the highest-priority mod's file as it is.
            return mods.first().cloned();
        };
        let _ = std::fs::create_dir_all(&self.cache);
        self.prune(id, &stem);
        // A name of this writer's: threads or clients building the same file never share one.
        static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tmp = self.cache.join(format!("{stem}.{}-{n}.tmp", std::process::id()));
        if std::fs::write(&tmp, &merged).and_then(|_| std::fs::rename(&tmp, &out)).is_err() {
            let _ = std::fs::remove_file(&tmp);
            // Another writer may have put it in place first.
            return if out.is_file() { Some(out) } else { mods.first().cloned() };
        }
        Some(out)
    }

    /// Older builds of `id` go but the newest other one (another config's, a client running beside this one, may still
    /// read it), never this build's (`stem`; another thread may be writing it); a temporary file left 10 min goes.
    fn prune(&self, id: u32, stem: &str) {
        let Ok(rd) = std::fs::read_dir(&self.cache) else { return };
        let prefix = format!("{id}-");
        let mut builds = Vec::new();
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if !name.starts_with(&prefix) || (name.starts_with(stem) && name.ends_with(".DAT")) {
                continue;
            }
            let age = e.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok());
            if name.ends_with(".tmp") {
                if age.is_some_and(|a| a.as_secs() > 600) {
                    let _ = std::fs::remove_file(e.path());
                }
            } else {
                builds.push((age.unwrap_or_default(), e.path()));
            }
        }
        builds.sort();
        for (_, p) in builds.into_iter().skip(1) {
            let _ = std::fs::remove_file(p);
        }
    }
}

/// A chunk's whole bytes (header included) and its match key.
struct Raw<'a> {
    bytes: &'a [u8],
    kind: u8,
    key: (Vec<[u8; 4]>, [u8; 4], u8, Vec<u8>, usize),
}

/// Chunks with keys: the folder path, the 4cc, the kind, a texture's name (D3s data + 1, 16 bytes: what the game
/// looks textures up by) and the n-th such chunk. None when it is not a well-formed chunk container.
fn raw(b: &[u8]) -> Option<Vec<Raw<'_>>> {
    let (mut out, mut at, mut stack) = (Vec::new(), 0, Vec::new());
    let mut seen: HashMap<(Vec<[u8; 4]>, [u8; 4], u8, Vec<u8>), usize> = HashMap::new();
    while at < b.len() {
        let w = u32::from_le_bytes(b.get(at + 4..at + 8)?.try_into().ok()?);
        let size = ((w >> 7) & 0x7ffff) as usize * 16;
        if size < 16 || at + size > b.len() {
            return None;
        }
        let (name, kind) = (b[at..at + 4].try_into().ok()?, (w & 0x7f) as u8);
        if kind == 1 {
            stack.push(name);
        }
        let tex =
            if kind == 0x20 { b.get(at + 17..at + 33).map(<[u8]>::to_vec).unwrap_or_default() } else { Vec::new() };
        let k = (stack.clone(), name, kind, tex);
        let n = seen.entry(k.clone()).or_default();
        *n += 1;
        out.push(Raw { bytes: &b[at..at + size], kind, key: (k.0, k.1, k.2, k.3, *n) });
        if kind == 0 {
            stack.pop();
        }
        at += size;
    }
    Some(out)
}

/// A D3s chunk holds its image: a sane size and at least half a byte a pixel (DXT1's) after the 57-byte header. A
/// broken one (a size field wrapped past 8 MB left an 80-byte stub in our pack) is passed over, so the next mod's
/// or the stock image stays.
fn image_ok(chunk: &[u8]) -> bool {
    let p = chunk.get(16..).unwrap_or_default();
    let (Some(w), Some(h)) = (p.get(21..25), p.get(25..29)) else { return false };
    let (w, h) =
        (u32::from_le_bytes(w.try_into().unwrap()) as usize, u32::from_le_bytes(h.try_into().unwrap()) as usize);
    (1..=4096).contains(&w) && (1..=4096).contains(&h) && p.len() >= 57 + w * h / 2
}

/// The stock file with the mods' D3s / Msb / Bp chunks in place of stock ones of the same key, and their changed
/// spell records in place of stock's; `mods` highest priority first. Chunks a mod has that stock lacks are left out
/// (the stock file was restructured since).
pub fn merge(stock: &[u8], mods: &[Vec<u8>]) -> Option<Vec<u8>> {
    let base = raw(stock)?;
    let parsed: Vec<Vec<Raw>> = mods.iter().map(|m| raw(m)).collect::<Option<_>>()?;
    let mut take: HashMap<&(Vec<[u8; 4]>, [u8; 4], u8, Vec<u8>, usize), &[u8]> = HashMap::new();
    let mut spells: HashMap<&(Vec<[u8; 4]>, [u8; 4], u8, Vec<u8>, usize), Vec<&[u8]>> = HashMap::new();
    for m in parsed.iter().rev() {
        for c in m.iter() {
            if c.kind == SPELLS {
                spells.entry(&c.key).or_default().push(c.bytes);
            } else if MERGED.contains(&c.kind) && (c.kind != 0x20 || image_ok(c.bytes)) {
                take.insert(&c.key, c.bytes);
            }
        }
    }
    let mut out = Vec::with_capacity(stock.len());
    for c in &base {
        if let Some(ms) = spells.get(&c.key).filter(|_| c.kind == SPELLS) {
            out.extend_from_slice(&spell_records(c.bytes, ms));
            continue;
        }
        out.extend_from_slice(take.get(&c.key).copied().filter(|_| MERGED.contains(&c.kind)).unwrap_or(c.bytes));
    }
    Some(out)
}

/// A stock spell chunk (header included) with each record that a mod's chunk changes taken from it; `mods` lowest
/// priority first, so the highest one's change wins. Records past either chunk's end stay as they are.
fn spell_records(stock: &[u8], mods: &[&[u8]]) -> Vec<u8> {
    let mut out = stock.to_vec();
    let n = (stock.len() - 16) / SPELL_SIZE;
    for m in mods {
        for k in 0..n.min((m.len() - 16) / SPELL_SIZE) {
            let at = 16 + k * SPELL_SIZE;
            let r = &m[at..at + SPELL_SIZE];
            if r != &stock[at..at + SPELL_SIZE] {
                out[at..at + SPELL_SIZE].copy_from_slice(r);
            }
        }
    }
    out
}

/// A pack's `full` copy of a stock file cut to what `merge` takes from it: the D3s and spell chunks that differ from
/// stock's of the same key, inside the folder markers that key them. Also the number of those chunks kept.
pub fn slim(stock: &[u8], full: &[u8]) -> Option<(Vec<u8>, usize)> {
    let base: HashMap<_, &[u8]> = raw(stock)?.into_iter().map(|c| (c.key, c.bytes)).collect();
    let (mut out, mut kept) = (Vec::new(), 0);
    for c in raw(full)? {
        let keep = match c.kind {
            0 | 1 => true,
            0x20 => image_ok(c.bytes) && base.get(&c.key).is_some_and(|b| *b != c.bytes),
            SPELLS => base.get(&c.key).is_some_and(|b| *b != c.bytes),
            _ => false,
        };
        if keep {
            kept += matches!(c.kind, 0x20 | SPELLS) as usize;
            out.extend_from_slice(c.bytes);
        }
    }
    Some((out, kept))
}

#[cfg(test)]
mod tests {
    use super::{Overlays, merge, slim};

    fn chunk(name: &[u8; 4], kind: u8, data: &[u8]) -> Vec<u8> {
        let size = (16 + data.len()).div_ceil(16) * 16;
        let mut c = name.to_vec();
        c.extend_from_slice(&(kind as u32 | ((size / 16) as u32) << 7).to_le_bytes());
        c.extend_from_slice(&[0; 8]);
        c.extend_from_slice(data);
        c.resize(size, 0);
        c
    }

    /// Textures come from the mods (the higher one first), a mod's stale model does not.
    #[test]
    fn textures_only_highest_first() {
        let tex = |n: u8| {
            let mut d = vec![0u8; 64];
            d[1..5].copy_from_slice(b"tex1");
            d[21..25].copy_from_slice(&1u32.to_le_bytes());
            d[25..29].copy_from_slice(&1u32.to_le_bytes());
            d[30] = n;
            d
        };
        let stock = [
            chunk(b"fold", 1, &[]),
            chunk(b"img1", 0x20, &tex(1)),
            chunk(b"mdl1", 0x2e, &[1; 16]),
            chunk(b"end\0", 0, &[]),
        ]
        .concat();
        let low = [
            chunk(b"fold", 1, &[]),
            chunk(b"img1", 0x20, &tex(2)),
            chunk(b"mdl1", 0x2e, &[9; 16]),
            chunk(b"end\0", 0, &[]),
        ]
        .concat();
        let high = [chunk(b"fold", 1, &[]), chunk(b"img1", 0x20, &tex(3)), chunk(b"end\0", 0, &[])].concat();
        let m = merge(&stock, &[high.clone(), low]).unwrap();
        let expect = [
            chunk(b"fold", 1, &[]),
            chunk(b"img1", 0x20, &tex(3)),
            chunk(b"mdl1", 0x2e, &[1; 16]),
            chunk(b"end\0", 0, &[]),
        ]
        .concat();
        assert_eq!(m, expect);
        assert!(merge(b"not a dat", &[high]).is_none());
    }

    /// A mod's image too small for its size is passed over: the next mod's (or stock) stays.
    #[test]
    fn broken_image_skipped() {
        let tex = |n: u8, w: u32, h: u32, len: usize| {
            let mut d = vec![0u8; len];
            d[1..5].copy_from_slice(b"tex1");
            d[21..25].copy_from_slice(&w.to_le_bytes());
            d[25..29].copy_from_slice(&h.to_le_bytes());
            d[30] = n;
            d
        };
        let stock = [chunk(b"img1", 0x20, &tex(1, 8, 8, 57 + 32))].concat();
        let low = [chunk(b"img1", 0x20, &tex(2, 16, 16, 57 + 128))].concat();
        let high = [chunk(b"img1", 0x20, &tex(3, 2048, 4096, 80))].concat();
        assert_eq!(merge(&stock, &[high.clone(), low.clone()]).unwrap(), low);
        assert_eq!(merge(&stock, &[high]).unwrap(), stock);
    }

    /// Two configs (other mods) and several threads resolving one id over one cache each read their build whole.
    #[test]
    fn shared_cache_builds_stay_readable() {
        let dir = std::env::temp_dir().join(format!("vana-overlay-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let tex = |n: u8| {
            let mut d = vec![0u8; 4096];
            d[1..5].copy_from_slice(b"tex1");
            d[21..25].copy_from_slice(&1u32.to_le_bytes());
            d[25..29].copy_from_slice(&1u32.to_le_bytes());
            d[30] = n;
            d
        };
        let file = |name: &str, n: u8| {
            let p = dir.join(name);
            std::fs::write(&p, [chunk(b"img1", 0x20, &tex(n)), chunk(b"end\0", 0, &[])].concat()).unwrap();
            p
        };
        let (stock, a, b) = (file("stock.DAT", 1), file("a.DAT", 2), file("b.DAT", 3));
        let cache = dir.join("cache");
        let config = |m: &std::path::Path| Overlays {
            by_id: [(7, vec![m.to_path_buf()])].into(),
            dirs: Vec::new(),
            cache: cache.clone(),
        };
        let want = |m: &std::path::Path| merge(&std::fs::read(&stock).unwrap(), &[std::fs::read(m).unwrap()]).unwrap();
        std::thread::scope(|s| {
            for k in 0..8 {
                let m = if k % 2 == 0 { &a } else { &b };
                let (o, want, stock) = (config(m), want(m), &stock);
                s.spawn(move || {
                    for _ in 0..40 {
                        let p = o.resolve(7, stock).unwrap();
                        assert_eq!(std::fs::read(&p).unwrap(), want);
                    }
                });
            }
        });
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A slim copy drops the stock chunks and the unchanged images, and merges to the same bytes as the full one.
    #[test]
    fn slim_merges_the_same() {
        let tex = |name: &[u8; 4], n: u8| {
            let mut d = vec![0u8; 64];
            d[1..5].copy_from_slice(name);
            d[21..25].copy_from_slice(&1u32.to_le_bytes());
            d[25..29].copy_from_slice(&1u32.to_le_bytes());
            d[30] = n;
            d
        };
        let stock = [
            chunk(b"fold", 1, &[]),
            chunk(b"img1", 0x20, &tex(b"tex1", 1)),
            chunk(b"img2", 0x20, &tex(b"tex2", 1)),
            chunk(b"mdl1", 0x2e, &[1; 16]),
            chunk(b"end\0", 0, &[]),
        ]
        .concat();
        let full = [
            chunk(b"fold", 1, &[]),
            chunk(b"img1", 0x20, &tex(b"tex1", 7)),
            chunk(b"img2", 0x20, &tex(b"tex2", 1)),
            chunk(b"mdl1", 0x2e, &[1; 16]),
            chunk(b"end\0", 0, &[]),
        ]
        .concat();
        let (s, kept) = slim(&stock, &full).unwrap();
        assert_eq!(kept, 1);
        assert_eq!(
            s,
            [chunk(b"fold", 1, &[]), chunk(b"img1", 0x20, &tex(b"tex1", 7)), chunk(b"end\0", 0, &[])].concat()
        );
        assert_eq!(merge(&stock, &[s]).unwrap(), merge(&stock, &[full]).unwrap());
    }

    /// Spell records (chunk 0x49, 100 bytes each) are taken one by one where a mod changed them: the higher mod wins
    /// a record both change, a record no mod changes stays stock (SE's later change kept), a mod's short chunk (made
    /// before SE added spells) leaves the new ones stock, and the rest of the file stays stock.
    #[test]
    fn spell_records_merged_one_by_one() {
        let spells = |rs: &[u8]| rs.iter().flat_map(|&r| [r; 100]).collect::<Vec<u8>>();
        let file = |rs: &[u8], model: u8| {
            [chunk(b"mgc_", 0x49, &spells(rs)), chunk(b"mdl1", 0x2e, &[model; 16]), chunk(b"end\0", 0, &[])].concat()
        };
        let stock = file(&[1, 2, 3, 4], 1);
        let low = file(&[7, 2, 8, 4], 9);
        let high = file(&[1, 2, 9], 9);
        assert_eq!(merge(&stock, &[high.clone(), low]).unwrap(), file(&[7, 2, 9, 4], 1));
        assert_eq!(merge(&stock, std::slice::from_ref(&high)).unwrap(), file(&[1, 2, 9, 4], 1));
        let (s, kept) = slim(&stock, &high).unwrap();
        assert_eq!(kept, 1);
        assert_eq!(merge(&stock, &[s]).unwrap(), merge(&stock, &[high]).unwrap());
        assert_eq!(slim(&stock, &stock).unwrap().1, 0);
    }
}
