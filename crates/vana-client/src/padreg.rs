//! Retail's gamepad settings, as FFXiPadConfig.exe saves them (the tool Config > Gamepad starts, FUN_101bccb0): string
//! values padmode000, padsin000, padexsin000 and padguid000 under HKLM\SOFTWARE\[Wow6432Node\]PlayOnline{US,,EU}\
//! SquareEnix\FinalFantasyXI, read once at start by FUN_10002840 (MODLOG "Gamepad"). On Windows from the registry; on
//! Linux and macOS from a Wine prefix's system.reg (Steam's Proton prefixes, WINEPREFIX, ~/.wine, or the prefix the
//! game folder sits in).

use std::path::{Path, PathBuf};

/// The values found, and where.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Retail {
    /// padmode000: six flags "a,b,c,d,e,f" (FUN_10005490): [0] pad on, [1] force feedback, [2] sliders, [3] hat
    /// switches, [4] input while the game is in the background, [5] XInput.
    pub padmode: Option<String>,
    pub padsin: Option<String>,
    pub padexsin: Option<String>,
    pub padguid: Option<String>,
    /// Where they were read: the registry key, or the system.reg file.
    pub source: String,
}

impl Retail {
    /// padmode000's flag `i`, when it is there.
    pub fn mode(&self, i: usize) -> Option<bool> {
        let s = self.padmode.as_deref()?;
        s.split([',', ' ']).filter(|t| !t.is_empty()).nth(i).map(|t| t.trim().parse::<i32>().unwrap_or(0) != 0)
    }
}

/// The product keys in the order looked at: US first (the English client), then Japanese, then European.
const PRODUCTS: [&str; 3] = ["PlayOnlineUS", "PlayOnline", "PlayOnlineEU"];
const NAMES: [&str; 4] = ["padmode000", "padsin000", "padexsin000", "padguid000"];

/// Retail's settings, if a tool ever saved some. `game_dir` is the config's game folder (a Wine prefix it sits in is
/// looked at first).
pub fn find(game_dir: &Path) -> Option<Retail> {
    #[cfg(windows)]
    {
        let _ = game_dir;
        windows::find()
    }
    #[cfg(not(windows))]
    {
        for reg in wine_files(game_dir) {
            let Ok(bytes) = std::fs::read(&reg) else { continue };
            if let Some(mut r) = parse_reg(&String::from_utf8_lossy(&bytes)) {
                r.source = format!("{} ({})", reg.display(), r.source);
                return Some(r);
            }
        }
        None
    }
}

/// The system.reg files that may hold the key, most likely first: the prefix the game folder is in, WINEPREFIX,
/// ~/.wine, Steam's Final Fantasy XI prefix (app 230330), then any other Steam prefix with PlayOnline installed (a
/// non-Steam shortcut's, as on the user's machine).
#[cfg(not(windows))]
fn wine_files(game_dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut push = |p: PathBuf| {
        if p.is_file() && !out.contains(&p) {
            out.push(p);
        }
    };
    for a in game_dir.ancestors() {
        push(a.join("system.reg"));
    }
    if let Some(p) = std::env::var_os("WINEPREFIX") {
        push(PathBuf::from(p).join("system.reg"));
    }
    let home = dirs::home_dir().unwrap_or_default();
    push(home.join(".wine/system.reg"));
    let mut compat = Vec::new();
    for steam in [
        ".steam/steam",
        ".local/share/Steam",
        ".var/app/com.valvesoftware.Steam/.local/share/Steam",
        "Library/Application Support/Steam",
    ] {
        let root = home.join(steam);
        compat.push(root.join("steamapps/compatdata"));
        // Other Steam libraries (libraryfolders.vdf's "path" lines).
        if let Ok(vdf) = std::fs::read_to_string(root.join("steamapps/libraryfolders.vdf")) {
            for line in vdf.lines() {
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("\"path\"") {
                    let p = rest.trim().trim_matches('"').replace("\\\\", "\\");
                    compat.push(PathBuf::from(p).join("steamapps/compatdata"));
                }
            }
        }
    }
    for c in &compat {
        push(c.join("230330/pfx/system.reg"));
    }
    for c in &compat {
        let Ok(rd) = std::fs::read_dir(c) else { continue };
        let mut dirs: Vec<PathBuf> = rd.flatten().map(|e| e.path().join("pfx")).collect();
        dirs.sort();
        for pfx in dirs {
            // Only a prefix PlayOnline was installed in: system.reg files are megabytes each.
            let c = pfx.join("drive_c");
            if c.join("Program Files (x86)/PlayOnline").is_dir()
                || c.join("Program Files/PlayOnline").is_dir()
                || c.join("Program Files (x86)/Steam/steamapps/common/FINAL FANTASY XI Online").is_dir()
            {
                push(pfx.join("system.reg"));
            }
        }
    }
    out
}

/// A Wine registry file's FFXI key (system.reg: `[Software\\Wow6432Node\\PlayOnlineUS\\SquareEnix\\FinalFantasyXI]
/// stamp` then `"name"="value"` lines). The key with a pad value in the product order wins.
pub fn parse_reg(text: &str) -> Option<Retail> {
    let mut found: Vec<(usize, String, Retail)> = Vec::new();
    let mut cur: Option<(usize, String)> = None;
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(rest) = line.strip_prefix('[') {
            let key = rest.split(']').next().unwrap_or("");
            cur = product_of(key).map(|p| (p, key.replace("\\\\", "\\")));
            continue;
        }
        let Some((p, key)) = &cur else { continue };
        let Some((name, value)) = reg_value(line) else { continue };
        let Some(n) = NAMES.iter().position(|x| x.eq_ignore_ascii_case(&name)) else { continue };
        let at = match found.iter().position(|f| f.1 == *key) {
            Some(i) => i,
            None => {
                found.push((*p, key.clone(), Retail { source: key.clone(), ..Default::default() }));
                found.len() - 1
            }
        };
        let r = &mut found[at].2;
        *[&mut r.padmode, &mut r.padsin, &mut r.padexsin, &mut r.padguid][n] = Some(value);
    }
    // 64-bit prefixes keep the 32-bit game's key under Wow6432Node; either is taken, by product.
    found.sort_by_key(|f| (f.0, !f.1.to_ascii_lowercase().contains("wow6432node")));
    found.into_iter().map(|f| f.2).find(|r| r.padsin.is_some() || r.padmode.is_some())
}

/// The product order of a key `Software\\[Wow6432Node\\]<product>\\SquareEnix\\FinalFantasyXI`, if it is one.
fn product_of(key: &str) -> Option<usize> {
    let parts: Vec<String> = key.split("\\\\").map(|s| s.to_ascii_lowercase()).collect();
    let parts: Vec<&str> = parts.iter().map(String::as_str).filter(|s| *s != "wow6432node").collect();
    match parts.as_slice() {
        ["software", product, "squareenix", "finalfantasyxi"] => {
            PRODUCTS.iter().position(|p| p.eq_ignore_ascii_case(product))
        }
        _ => None,
    }
}

/// A `"name"="value"` line (string values only; Wine escapes `\\` and `\"`).
fn reg_value(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix('"')?;
    let (name, rest) = rest.split_once("\"=")?;
    let v = rest.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::new();
    let mut chars = v.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            out.extend(chars.next());
        } else {
            out.push(c);
        }
    }
    Some((name.to_string(), out))
}

#[cfg(windows)]
mod windows {
    use super::{NAMES, PRODUCTS, Retail};
    use windows_sys::Win32::System::Registry::{
        HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RRF_SUBKEY_WOW6432KEY, RegGetValueW,
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn value(key: &str, name: &str) -> Option<String> {
        let (k, n) = (wide(key), wide(name));
        let mut buf = [0u16; 512];
        let mut len = (buf.len() * 2) as u32;
        // The game is 32-bit: its key is in the 32-bit view (Wow6432Node on a 64-bit system).
        let r = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                k.as_ptr(),
                n.as_ptr(),
                RRF_RT_REG_SZ | RRF_SUBKEY_WOW6432KEY,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast(),
                &mut len,
            )
        };
        if r != 0 {
            return None;
        }
        let n = (len as usize / 2).min(buf.len());
        Some(String::from_utf16_lossy(&buf[..n]).trim_end_matches('\0').to_string())
    }

    pub fn find() -> Option<Retail> {
        for p in PRODUCTS {
            let key = format!("SOFTWARE\\{p}\\SquareEnix\\FinalFantasyXI");
            let [padmode, padsin, padexsin, padguid] = NAMES.map(|n| value(&key, n));
            if padsin.is_some() || padmode.is_some() {
                return Some(Retail { padmode, padsin, padexsin, padguid, source: format!("HKLM\\{key}") });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The user's Steam prefix (ffxi-pad-re registry_steam_prefix.txt), with a Japanese key before it.
    const REG: &str = r#"WINE REGISTRY Version 2
[Software\\Wow6432Node\\PlayOnline\\SquareEnix\\FinalFantasyXI] 1785805000
"0001"=dword:00000500

[Software\\Wow6432Node\\PlayOnlineUS\\SquareEnix\\FinalFantasyXI] 1785805585
#time=1dd23ad79020c32
"padexsin000"="-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-33,-33,32,32,-36,-36,35,35,6,7,5,4,-1,-1,-1,-1,-1,-1,-1"
"padguid000"="{55DDF31E-8F9C-11F1-8001-000044455354}"
"padmode000"="1,1,0,0,0,1"
"padsin000"="8,9,13,12,10,0,1,3,2,15,-1,-1,14,-33,-33,32,32,-36,-36,35,35,6,7,5,4,11,-1,-1,-1,-1,-1,-1"

[Software\\Wow6432Node\\PlayOnlineUS\\SquareEnix\\FinalFantasyXI\\Other] 1
"padsin000"="0"
"#;

    #[test]
    fn reads_wine_registry() {
        let r = parse_reg(REG).unwrap();
        assert!(r.padsin.as_deref().unwrap().starts_with("8,9,13,12,10,0,1,3,2,15,"));
        assert!(r.padexsin.as_deref().unwrap().starts_with("-1,-1,"));
        assert_eq!(r.padguid.as_deref(), Some("{55DDF31E-8F9C-11F1-8001-000044455354}"));
        assert_eq!(
            (r.mode(0), r.mode(1), r.mode(4), r.mode(5), r.mode(9)),
            (Some(true), Some(true), Some(false), Some(true), None)
        );
        assert!(r.source.contains("PlayOnlineUS"), "{}", r.source);
        // No pad values: nothing.
        assert!(
            parse_reg("[Software\\\\PlayOnlineUS\\\\SquareEnix\\\\FinalFantasyXI] 1\n\"0001\"=dword:1\n").is_none()
        );
        // A 32-bit prefix (no Wow6432Node) and escapes.
        let r = parse_reg("[Software\\\\PlayOnlineEU\\\\SquareEnix\\\\FinalFantasyXI] 1\n\"padsin000\"=\"a\\\\b\"\n")
            .unwrap();
        assert_eq!(r.padsin.as_deref(), Some("a\\b"));
    }
}
