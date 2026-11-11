//! The modern UI's setup per character and main job (not retail): its window places (each display's), folded edge
//! windows, mode presets, chat window settings and favourites, kept in <data>/ui/config/<World>-<Name>/<JOB>.json
//! (beside an FFXI_CONFIG copy for test runs). The config's own fields hold the one in use. A job new to a character
//! starts as the job it comes from (the character's last one, at a login), favourites aside: they are a job's spells
//! and abilities, as the hotbar's bars are (which it keeps by job itself). A character's first job takes the config's
//! fields as they were.

use super::World;
use crate::config::{Config, ModernLayout, ModernModes};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One character's job's setup.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// Window places by display: "" the latest (the template for a display not shown yet), else " @WxH S%".
    pub places: BTreeMap<String, ModernLayout>,
    pub folded: BTreeMap<String, bool>,
    pub modes: ModernModes,
    pub chat_font: f32,
    pub chat_hidden: Vec<String>,
    pub battle_in_all: bool,
    pub condense: bool,
    pub scan_kinds: u8,
    pub favourites: Vec<String>,
    pub fav_strip: bool,
    /// Fields of a newer client, kept as they are.
    #[serde(flatten)]
    pub unknown: serde_json::Map<String, serde_json::Value>,
}

fn root() -> PathBuf {
    match std::env::var_os("FFXI_CONFIG") {
        Some(p) => PathBuf::from(p).with_file_name("ui").join("config"),
        None => crate::config::data_dir().join("ui").join("config"),
    }
}

/// A character's folder name, as the Lua mods' storage names its file: "<World>-<Name>", letters and digits.
pub fn character(world: &str, name: &str) -> String {
    let clean = |s: &str| s.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>();
    format!("{}-{}", clean(world), clean(name))
}

fn path(character: &str, job: &str) -> PathBuf {
    root().join(character).join(format!("{job}.json"))
}

/// The main job as its setup is named (MON in Monstrosity); none until it is known.
fn job(w: &World) -> Option<&'static str> {
    if super::monstrosity::active(w) {
        return Some("MON");
    }
    super::items::JOBS.get(w.stats.as_ref()?.main.0 as usize).copied().filter(|j| !j.is_empty())
}

/// The character and main job whose setup the world wants, when it is not the one in use (each frame: a world is one
/// character's, so only the job is compared).
pub fn wanted(w: &World) -> Option<(String, String)> {
    let job = job(w).filter(|j| !w.name.is_empty() && w.ui_profile.as_ref().is_none_or(|p| p.1 != *j))?;
    Some((character(&w.world_name, &w.name), job.to_string()))
}

/// The character `name`'s setup as the config and the world hold it now.
fn take(cfg: &Config, w: &World) -> UiConfig {
    let places = cfg
        .modern_places
        .iter()
        .filter(|(k, _)| super::modern::layout_of(k, &w.name))
        .map(|(k, v)| (k[w.name.len()..].to_string(), v.clone()))
        .collect();
    UiConfig {
        places,
        folded: cfg.modern_folded.clone(),
        modes: cfg.modern_modes.clone(),
        chat_font: cfg.modern_chat_font,
        chat_hidden: cfg.modern_chat_hidden.clone(),
        battle_in_all: cfg.modern_battle_in_all,
        condense: cfg.modern_condense,
        scan_kinds: cfg.modern_scan_kinds,
        favourites: w.favourites.clone(),
        fav_strip: w.fav_strip,
        unknown: Default::default(),
    }
}

/// A setup into the config and the world; `favourites` false keeps the world's.
fn put(cfg: &mut Config, w: &mut World, u: &UiConfig, favourites: bool) {
    cfg.modern_places.retain(|k, _| !super::modern::layout_of(k, &w.name));
    for (k, v) in &u.places {
        cfg.modern_places.insert(format!("{}{k}", w.name), v.clone());
    }
    cfg.modern_folded = u.folded.clone();
    cfg.modern_modes = u.modes.clone();
    if u.chat_font > 0.0 {
        cfg.modern_chat_font = u.chat_font;
    }
    cfg.modern_chat_hidden = u.chat_hidden.clone();
    (cfg.modern_battle_in_all, cfg.modern_condense, cfg.modern_scan_kinds) =
        (u.battle_in_all, u.condense, u.scan_kinds);
    if favourites {
        (w.favourites, w.fav_strip) = (u.favourites.clone(), u.fav_strip);
        cfg.modern_favourites.insert(w.name.clone(), w.favourites.clone());
        cfg.modern_fav_strip = w.fav_strip;
    }
}

fn read(p: &Path) -> Option<UiConfig> {
    let text = std::fs::read(p).ok()?;
    serde_json::from_slice(&text).map_err(|e| log::warn!("ui config {}: {e}", p.display())).ok()
}

fn write(p: &Path, u: &UiConfig) {
    let tmp = p.with_extension(format!("json.{}.tmp", std::process::id()));
    let done = (|| -> anyhow::Result<()> {
        std::fs::create_dir_all(p.parent().unwrap_or(p))?;
        std::fs::write(&tmp, serde_json::to_vec_pretty(u)?)?;
        std::fs::rename(&tmp, p)?;
        Ok(())
    })();
    if let Err(e) = done {
        log::warn!("ui config {} not saved: {e:#}", p.display());
    }
}

/// The character's most recently saved job, other than `but`.
fn latest(character: &str, but: &str) -> Option<String> {
    std::fs::read_dir(root().join(character))
        .ok()?
        .flatten()
        .filter_map(|e| {
            let job = e.path().file_stem()?.to_str()?.to_string();
            let fresh = e.metadata().ok()?.modified().ok()?;
            (e.path().extension()? == "json" && job != but).then_some((fresh, job))
        })
        .max()
        .map(|x| x.1)
}

/// The world's setup to `to` (character, job): the one in use saved first; `to`'s loaded, else made from the job it
/// comes from (in this session, else the character's latest), else from the config as it is (the character's first).
pub fn switch(cfg: &mut Config, w: &mut World, to: (String, String)) {
    if let Some(from) = w.ui_profile.take() {
        write(&path(&from.0, &from.1), &take(cfg, w));
    }
    let (character, job) = &to;
    match read(&path(character, job)) {
        Some(u) => {
            log::info!("ui config: {character} {job}");
            put(cfg, w, &u, true);
        }
        None => {
            let source = latest(character, job);
            if let Some(src) = source.as_ref().and_then(|j| read(&path(character, j))) {
                put(cfg, w, &src, false);
            }
            let mut u = take(cfg, w);
            // A new job's actions are its own (as its hotbar): the favourites start empty.
            if source.is_some() {
                u.favourites.clear();
                put(cfg, w, &u, true);
            }
            log::info!("ui config: {character} {job} made from {}", source.as_deref().unwrap_or("the config"));
            write(&path(character, job), &u);
        }
    }
    w.ui_profile = Some(to);
}

/// The setup in use to its file (after the config saved a change).
pub fn save(cfg: &Config, w: &World) {
    if let Some((character, job)) = &w.ui_profile {
        write(&path(character, job), &take(cfg, w));
    }
}

/// The other setups kept (character folder, job), for Settings' UI Setup row, by character then job.
pub fn others(w: &World) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for c in std::fs::read_dir(root()).into_iter().flatten().flatten() {
        let Some(character) = c.file_name().to_str().map(String::from) else { continue };
        for j in std::fs::read_dir(c.path()).into_iter().flatten().flatten() {
            let p = j.path();
            if let (Some(job), Some("json")) =
                (p.file_stem().and_then(|s| s.to_str()), p.extension().and_then(|s| s.to_str()))
            {
                out.push((character.clone(), job.to_string()));
            }
        }
    }
    out.sort();
    out.retain(|o| w.ui_profile.as_ref() != Some(o));
    out
}

/// How a kept setup reads in Settings: "Nine WHM" (the world's name too when another world has a Nine).
pub fn label(all: &[(String, String)], (character, job): &(String, String)) -> String {
    let name = |c: &str| c.split_once('-').map_or(c.to_string(), |(_, n)| n.to_string());
    let shared = all.iter().any(|(c, _)| c != character && name(c) == name(character));
    match character.split_once('-') {
        Some((world, n)) if shared => format!("{n} {job} ({world})"),
        _ => format!("{} {job}", name(character)),
    }
}

/// Another character's or job's setup copied into the one in use: places, modes and the chat window's settings, not
/// the favourites (this job's own).
pub fn copy_from(cfg: &mut Config, w: &mut World, from: &(String, String)) {
    if let Some(u) = read(&path(&from.0, &from.1)) {
        log::info!("ui config: {} {} copied in", from.0, from.1);
        put(cfg, w, &u, false);
        save(cfg, w);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Characters on two worlds with one name read with their world; the rest by name.
    #[test]
    fn labels() {
        let all = [
            ("Asura-Tsetse".to_string(), "WHM".to_string()),
            ("Nameless-Nine".to_string(), "NIN".to_string()),
            ("Asura-Nine".to_string(), "WAR".to_string()),
        ];
        assert_eq!(label(&all, &all[0]), "Tsetse WHM");
        assert_eq!(label(&all, &all[1]), "Nine NIN (Nameless)");
        assert_eq!(character("Nameless", "Nine"), "Nameless-Nine");
    }
}
