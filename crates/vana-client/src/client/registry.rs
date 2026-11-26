//! Ours (the modern UI's action registry, plan D): one place that says, for a spell or an ability, who it may target,
//! how far it reaches, how far the current target is, and why it can't be used now. The checks are retail's
//! (magic::magic_check / ability_check, FUN_1022cf30 / FUN_10224b70, their group 7 lines) plus the effects the
//! modern UI already dims by (magic::blocked). The lists' cost box shows it.

use super::World;
use super::magic;

/// What the registry knows about one action.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Info {
    /// The kinds it may target, in the prompt's order ("Self", "Party", "Enemy" ...).
    pub targets: Vec<&'static str>,
    /// The reach in yalms (spell +0x45 or ability +0x10 through table 0x103654a8).
    pub range: Option<f32>,
    /// The current target's distance (flat, as the prompt's arrow measures it) when the action may take it.
    pub distance: Option<f32>,
    /// The prompt's arrow would be red on the current target (target::reach_tint with the action's range checks:
    /// both radii, sub-list 6 / 10 moves from the pet too).
    pub out: bool,
    /// Why the action can't be used now: retail's refusal line, or the effect that stops it.
    pub why: Option<String>,
}

impl Info {
    /// The current target is a valid pick but beyond the range (the prompt's red arrow).
    pub fn out_of_range(&self) -> bool {
        self.distance.is_some() && self.out
    }
}

/// DAT valid-target bits (spell +6, ability +0xa) by the kinds target::SUB_FLAGS maps them to.
const TARGETS: [(u16, &str); 8] = [
    (1, "Self"),
    (4, "Party"),
    (8, "Alliance"),
    (0x10, "Player"),
    (2, "Ally NPC"),
    (0x40, "NPC"),
    (0x80, "NPC"),
    (0x20, "Enemy"),
];

fn target_names(bits: u16) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for (b, name) in TARGETS {
        if bits & b != 0 && !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

/// The effect we carry that stops the action, as a reason (the ids magic::blocked tests).
fn effect_reason(w: &World, magic: bool) -> Option<&'static str> {
    w.buffs.iter().find_map(|&b| match b {
        2 | 19 | 193 => Some("Asleep."),
        7 => Some("Petrified."),
        10 => Some("Stunned."),
        14 | 17 => Some("Charmed."),
        28 => Some("Terrorized."),
        6 if magic => Some("Silenced."),
        29 if magic => Some("Muted."),
        16 if !magic => Some("Under amnesia."),
        _ => None,
    })
}

/// A refusal id's group 7 line, or a short line of ours when the DAT has none (a silent refusal).
fn line(w: &World, id: usize) -> String {
    w.battle
        .as_ref()
        .and_then(|bt| bt.group7(id))
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .unwrap_or_else(|| "Can't be used now.".into())
}

/// Whether the arrow would be red on the current target with `range`.
fn red(w: &World, range: &super::target::Reaches) -> bool {
    super::target::reach_tint(w, range, w.target, 0) == super::target::RED
}

/// The current target's flat distance when `bits` may take it (target::takes).
fn distance(w: &World, bits: u16) -> Option<f32> {
    let t = w.target.filter(|&t| t != w.my_id && super::target::takes(w, bits as u32, t))?;
    let p = w.entities.get(&t)?.pos;
    Some(glam::Vec2::new(p.x - w.pos.x, p.z - w.pos.z).length())
}

pub fn spell(w: &World, s: u16) -> Option<Info> {
    let r = w.actions.as_ref()?.spell(s)?.clone();
    let why = magic::magic_check(w, s).map(|id| line(w, id)).or_else(|| effect_reason(w, true).map(String::from));
    // The menu's mask: Pianissimo / Entrust widen a self-only spell to the party (FUN_10102430).
    let targets = crate::dat::actions::spell_targets(r.targets, r.kind, r.flags, &w.buffs);
    Some(Info {
        targets: target_names(targets),
        range: Some(crate::dat::actions::RANGES[(r.range & 0xf) as usize] as f32).filter(|&r| r < 255.0),
        distance: distance(w, targets),
        out: red(w, &super::target::spell_reaches(w, s)),
        why,
    })
}

pub fn ability(w: &World, i: u16) -> Option<Info> {
    let a = w.actions.as_ref()?.abilities.get(i as usize)?.clone();
    if a.kind == 4 || a.sub != 0 {
        // Traits are always on; a sub-list row only opens its list.
        return Some(Info { targets: Vec::new(), ..Default::default() });
    }
    let why = magic::ability_check(w, i).map(|id| line(w, id)).or_else(|| effect_reason(w, false).map(String::from));
    Some(Info {
        targets: target_names(a.targets),
        range: Some(crate::dat::actions::RANGES[(a.range & 0xf) as usize] as f32).filter(|&r| r < 255.0),
        distance: distance(w, a.targets),
        out: red(w, &super::target::ability_reaches(w, i)),
        why,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_names_follow_the_prompt_kinds() {
        assert_eq!(target_names(0x20), ["Enemy"]);
        assert_eq!(target_names(1 | 4 | 8), ["Self", "Party", "Alliance"]);
        assert_eq!(target_names(0x40 | 0x80), ["NPC"]);
        assert!(target_names(0).is_empty());
    }

    /// Out of range only with a target the action may take and a red arrow.
    #[test]
    fn out_of_range_needs_a_target() {
        assert!(Info { distance: Some(30.0), out: true, ..Default::default() }.out_of_range());
        assert!(!Info { distance: Some(30.0), out: false, ..Default::default() }.out_of_range());
        assert!(!Info { distance: None, out: true, ..Default::default() }.out_of_range());
    }
}
