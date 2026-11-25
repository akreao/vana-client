//! Retail FINAL FANTASY XI installs on this computer: HKLM\SOFTWARE\PlayOnline{,US,EU}\InstallFolder "0001" (the 32-bit
//! view, as FFXiMain's FUN_101bccb0 reads it), from the registry on Windows and from Wine prefixes' system.reg
//! elsewhere (the prefixes padreg looks in).

use std::path::{Path, PathBuf};

/// An install found, and where it was read.
#[derive(Clone, Debug, PartialEq)]
pub struct Install {
    pub path: PathBuf,
    pub source: String,
}

const PRODUCTS: [&str; 3] = ["PlayOnlineUS", "PlayOnline", "PlayOnlineEU"];

/// The installs found, each folder once (the one `game_dir` names too, when it is one).
pub fn find(game_dir: &Path) -> Vec<Install> {
    let mut out: Vec<Install> = Vec::new();
    let mut push = |path: PathBuf, source: String| {
        let same = |a: &Path| std::fs::canonicalize(a).ok() == std::fs::canonicalize(&path).ok();
        if path.is_dir() && !out.iter().any(|i| same(&i.path)) {
            out.push(Install { path, source });
        }
    };
    #[cfg(windows)]
    {
        let _ = game_dir;
        for p in PRODUCTS {
            let key = format!("SOFTWARE\\{p}\\InstallFolder");
            if let Some(v) = crate::padreg::windows::value(&key, "0001") {
                push(PathBuf::from(v.trim_end_matches('\\')), format!("HKLM\\{key}"));
            }
        }
    }
    #[cfg(not(windows))]
    {
        wine(game_dir, &mut push);
    }
    out
}

#[cfg(not(windows))]
fn wine(game_dir: &Path, push: &mut impl FnMut(PathBuf, String)) {
    for reg in crate::padreg::wine_files(game_dir) {
        let Ok(bytes) = std::fs::read(&reg) else { continue };
        let prefix = reg.parent().unwrap_or(Path::new("."));
        for (key, win) in install_values(&String::from_utf8_lossy(&bytes)) {
            if let Some(p) = unix_path(prefix, &win) {
                push(p, format!("{} ({key})", reg.display()));
            }
        }
        // The installer's default folders, for a prefix whose registry lost the key.
        let c = prefix.join("drive_c");
        for d in ["Program Files (x86)", "Program Files"] {
            let p = c.join(d).join("PlayOnline/SquareEnix/FINAL FANTASY XI");
            push(p, format!("{} (default folder)", prefix.display()));
        }
    }
}

#[cfg_attr(windows, allow(dead_code))]
/// A system.reg's InstallFolder "0001" values: (key, Windows path), US first.
pub fn install_values(text: &str) -> Vec<(String, String)> {
    let mut found: Vec<(usize, String, String)> = Vec::new();
    let mut cur: Option<(usize, String)> = None;
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(rest) = line.strip_prefix('[') {
            let key = rest.split(']').next().unwrap_or("");
            cur = product_of(key).map(|p| (p, key.replace("\\\\", "\\")));
            continue;
        }
        let Some((p, key)) = &cur else { continue };
        if let Some((name, value)) = crate::padreg::reg_value(line)
            && name == "0001"
            && !value.is_empty()
        {
            found.push((*p, key.clone(), value));
        }
    }
    found.sort_by_key(|f| f.0);
    found.into_iter().map(|f| (f.1, f.2)).collect()
}

#[cfg_attr(windows, allow(dead_code))]
/// The product order of `Software\\[Wow6432Node\\]<product>\\InstallFolder`, if the key is one.
fn product_of(key: &str) -> Option<usize> {
    let parts: Vec<String> = key.split("\\\\").map(|s| s.to_ascii_lowercase()).collect();
    let parts: Vec<&str> = parts.iter().map(String::as_str).filter(|s| *s != "wow6432node").collect();
    match parts.as_slice() {
        ["software", product, "installfolder"] => PRODUCTS.iter().position(|p| p.eq_ignore_ascii_case(product)),
        _ => None,
    }
}

#[cfg_attr(windows, allow(dead_code))]
/// A Windows path in a Wine prefix: its drive through dosdevices (c: is drive_c).
pub fn unix_path(prefix: &Path, win: &str) -> Option<PathBuf> {
    let (drive, rest) = win.split_once(':')?;
    if drive.len() != 1 {
        return None;
    }
    let d = drive.to_ascii_lowercase();
    let dev = prefix.join("dosdevices").join(format!("{d}:"));
    let mut p = if dev.exists() || d != "c" { dev } else { prefix.join("drive_c") };
    for part in rest.split(['\\', '/']).filter(|s| !s.is_empty()) {
        p.push(part);
    }
    Some(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_and_paths() {
        let reg = "[Software\\\\Wow6432Node\\\\PlayOnlineEU\\\\InstallFolder] 1\n\"0001\"=\"D:\\\\FFXI\"\n\
                   [Software\\\\Wow6432Node\\\\PlayOnlineUS\\\\InstallFolder] 1\n\
                   \"0001\"=\"C:\\\\Program Files (x86)\\\\PlayOnline\\\\SquareEnix\\\\FINAL FANTASY XI\\\\\"\n\
                   \"1000\"=\"C:\\\\Program Files (x86)\\\\PlayOnline\\\\SquareEnix\\\\PlayOnlineViewer\"\n\
                   [Software\\\\Wow6432Node\\\\PlayOnlineUS\\\\SquareEnix\\\\FinalFantasyXI] 1\n\"0001\"=\"x\"\n";
        let v = install_values(reg);
        assert_eq!(v.len(), 2, "{v:?}");
        assert!(v[0].0.contains("PlayOnlineUS") && v[0].1.ends_with("FINAL FANTASY XI\\"));
        let pfx = Path::new("/nowhere/pfx");
        assert_eq!(
            unix_path(pfx, &v[0].1).unwrap(),
            Path::new("/nowhere/pfx/drive_c/Program Files (x86)/PlayOnline/SquareEnix/FINAL FANTASY XI")
        );
        assert_eq!(unix_path(pfx, "D:\\FFXI").unwrap(), Path::new("/nowhere/pfx/dosdevices/d:/FFXI"));
        assert_eq!(unix_path(pfx, "FFXI"), None);
    }
}
