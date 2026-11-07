//! Pending actions (not retail; the plan's "state changes are server-confirmed" and its open question "how latency,
//! failed transfers and invalid targets are shown"): a spell, weapon skill, job ability or item we asked the server
//! for (c2s 0x01A / 0x037) is pending until the server answers. An action packet of ours (0x028) of its kind and id
//! confirms it (a melee round or another action's does not); a battle message of ours (0x029) soon after the ask
//! that is one of the request path's refusals for its kind — and names the asked id where the server sends one — is
//! its refusal, kept with the message's text a few seconds; no answer for a while says so. The modern layer shows
//! it (modern/pending.rs chip); retail's lines print as ever.

use super::World;
use crate::net::action::{Action, category as cat};
use std::time::{Duration, Instant};

/// What was asked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ask {
    Spell(u16),
    WeaponSkill(u16),
    /// The ability's name-table id (the 0x01A param + 0x200).
    Ability(u16),
    Item(u16),
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Waiting,
    /// The server's refusal (its line, plain).
    Refused(String),
    /// Nothing came back.
    NoReply,
}

#[derive(Clone, Debug)]
pub struct Pending {
    pub ask: Ask,
    /// On whom (0: none named).
    pub target: u32,
    pub at: Instant,
    pub state: State,
    /// When the state last changed.
    pub since: Instant,
}

/// The chip shows only past this (a quick answer never flashes it).
pub const SHOW_AFTER: Duration = Duration::from_millis(150);
/// No answer within this is NoReply.
const TIMEOUT: Duration = Duration::from_secs(5);
/// A refusal comes at once (the server answers a refused ask as it reads it); a message of ours later is something
/// else (an effect wearing off ...). Inferred, with room for lag.
const REFUSED_WITHIN: Duration = Duration::from_secs(2);
/// A refusal or NoReply stays this long.
const KEEP: Duration = Duration::from_secs(4);

/// An action asked for.
pub fn start(w: &mut World, ask: Ask, target: u32) {
    let now = Instant::now();
    w.asked = Some(Pending { ask, target, at: now, state: State::Waiting, since: now });
}

/// The id the server's packets carry for `ask`: the spell or weapon-skill id as is, the ability as its 0x01A param
/// (the name-table id - 0x200: LSB answers with `ability->getId()`, the client reads a finish's cmd_arg back through
/// timers.rs `Source::Ability` as `n.ability(0x200 + arg)`), the item id.
fn packet_id(ask: Ask) -> u16 {
    match ask {
        Ask::Spell(id) | Ask::WeaponSkill(id) | Ask::Item(id) => id,
        Ask::Ability(id) => id.wrapping_sub(0x200),
    }
}

/// The action id a 0x028 of ours carries, where it names one: a finish's cmd_arg (spell / weapon skill / ability
/// param / item id), or a "ca.." start's first record value ("cate" skill, "caso" song, "cait" item ...; an interrupt's
/// "sp.." names none). Melee and ranged rounds carry none either.
fn action_id(a: &Action) -> Option<u16> {
    match a.category {
        cat::SKILL_FINISH..=cat::ABILITY_FINISH | cat::MOB_SKILL_FINISH | 13..=15 => Some(a.arg as u16),
        cat::SKILL_START..=cat::ABILITY_START | cat::RANGED_START if a.arg & 0xffff == 0x6163 => {
            a.targets.first()?.results.first().map(|r| r.value as u16)
        }
        _ => None,
    }
}

/// Whether a 0x028 of ours answers `ask`: its category is the ask's kind and — where the packet names an action — its
/// id is the asked one (another spell's finish, or a melee round while a spell waits, does not).
fn answers(ask: Ask, a: &Action) -> bool {
    let kind = match ask {
        Ask::Spell(_) => matches!(a.category, cat::MAGIC_FINISH | cat::MAGIC_START),
        // Monster skills (Monstrosity) finish as category 11: a mob skill id, not a weapon skill's, so no id check.
        Ask::WeaponSkill(_) => matches!(a.category, cat::SKILL_FINISH | cat::SKILL_START | cat::MOB_SKILL_FINISH),
        Ask::Ability(_) => matches!(a.category, cat::ABILITY_FINISH | 14 | 15),
        // An item use's 0x028 id space is not established; the category alone confirms.
        Ask::Item(_) => matches!(a.category, cat::ITEM_FINISH | cat::ITEM_START),
    };
    if !kind {
        return false;
    }
    match ask {
        Ask::Item(_) => true,
        Ask::WeaponSkill(_) if a.category == cat::MOB_SKILL_FINISH => true,
        _ => action_id(a).is_some_and(|id| id == packet_id(ask)),
    }
}

/// 0x028 of ours: the action waiting is confirmed by its own start or finish (the cast bar or the effect takes over).
pub fn confirm(w: &mut World, a: &Action) {
    if w.asked.as_ref().is_some_and(|p| p.state == State::Waiting && answers(p.ask, a)) {
        w.asked = None;
    }
}

/// Whether a 0x029 of ours is the refusal of `ask`: its id is one of the messages the server's request path sends for
/// that kind (LSB's controllers and action states), and where the message names an action (its Data) that is the
/// asked id. Messages that only ever answer the ask (a recast's 18, an interrupt's 16) carry no id.
fn refusal(ask: Ask, id: u16, param: i32) -> bool {
    let named = param == packet_id(ask) as i32;
    match ask {
        // magic_state's refusals carry the spell id; the controller's recast check (18) and an interrupt (16) carry 0.
        Ask::Spell(_) => match id {
            16 | 18 => param == 0,
            34 | 35 | 40 | 47 | 49 | 71 | 78 | 155 | 313 => named,
            _ => false,
        },
        // weapon skill: CannotPerformAction 71, UnableToUseWeaponskill 89 / CannotUseWeaponskill 190,
        // CannotUseAnyWeaponskill 191, NotEnoughTP 192, NoRangedWeapon 216, CannotSee 217 — all param 0
        // (player_controller / weaponskill_state).
        Ask::WeaponSkill(_) => param == 0 && matches!(id, 71 | 89 | 190 | 191 | 192 | 216 | 217),
        // ability: out of range 4 / unable to see 5 / too far 78 / cannot use 87 / 88 / recast 94 — param 0, or the
        // ability's param for its lua check's message (player_controller / ability_state).
        Ask::Ability(_) => matches!(id, 4 | 5 | 78 | 87 | 88 | 94) && (param == 0 || named),
        // item: UnableToUseItem 56 param 0; CannotUseItems 445 / ItemFailsToActivate 62 carry the item id
        // (item_state's OnItemCheck / TryInterrupt).
        Ask::Item(_) => match id {
            56 => param == 0,
            62 | 445 => param == 0 || named,
            _ => false,
        },
    }
}

/// 0x029 of ours soon after the ask, while waiting: its line is the refusal.
pub fn message(w: &mut World, id: u16, param: i32, line: &str) {
    if let Some(p) = w.asked.as_mut().filter(|p| p.state == State::Waiting && p.at.elapsed() < REFUSED_WITHIN) {
        if refusal(p.ask, id, param) {
            p.state = State::Refused(crate::dat::font::strip_codes(line).trim().to_string());
            p.since = Instant::now();
        }
    }
}

/// Per frame: the wait runs out, an answer shown long enough goes.
pub fn update(w: &mut World) {
    let Some(p) = w.asked.as_mut() else { return };
    match p.state {
        State::Waiting if p.at.elapsed() > TIMEOUT => {
            p.state = State::NoReply;
            p.since = Instant::now();
        }
        State::Refused(_) | State::NoReply if p.since.elapsed() > KEEP => w.asked = None,
        _ => {}
    }
}

/// The action's name.
pub fn name(w: &World, ask: Ask) -> String {
    match ask {
        Ask::Spell(id) => super::magic::spell_name(w, id),
        Ask::WeaponSkill(id) | Ask::Ability(id) => super::magic::ability_name(w, id),
        Ask::Item(id) => w.item_info.as_ref().and_then(|c| c.item(id)).map(|it| it.name.clone()).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Ask, answers, refusal};
    use crate::net::action::{Action, ActionResult, ActionTarget};

    fn action(category: u8, arg: u32, value: Option<u32>) -> Action {
        let targets = value
            .map(|v| vec![ActionTarget { id: 1, results: vec![ActionResult { value: v, ..Default::default() }] }])
            .unwrap_or_default();
        Action { actor: 1, category, arg, targets, ..Default::default() }
    }

    #[test]
    fn a_melee_round_does_not_answer_a_spell() {
        // Issue #243: a category-1 Action of ours while a spell waits is not its answer.
        assert!(!answers(Ask::Spell(10), &action(1, 0x746b_6174, None)));
        assert!(answers(Ask::Spell(10), &action(4, 10, None)));
        assert!(!answers(Ask::Spell(10), &action(4, 11, None)), "another spell's finish");
        // A magic start ("caso", the spell in the record's value) confirms; an interrupt ("spso") does not.
        assert!(answers(Ask::Spell(10), &action(8, 0x6f73_6163, Some(10))));
        assert!(!answers(Ask::Spell(10), &action(8, 0x6f73_7073, Some(10))));
        assert!(!answers(Ask::Spell(10), &action(8, 0x6f73_6163, Some(11))));
    }

    #[test]
    fn each_kind_matches_its_own_ids() {
        assert!(answers(Ask::WeaponSkill(300), &action(7, 0x6574_6163, Some(300))));
        assert!(!answers(Ask::WeaponSkill(300), &action(7, 0x6574_6163, Some(301))));
        assert!(answers(Ask::WeaponSkill(300), &action(3, 300, None)));
        // A monster skill finish (11) names a mob skill, so any answers it.
        assert!(answers(Ask::WeaponSkill(300), &action(11, 999, None)));
        // An ability's finish carries its 0x01A param (name-table id - 0x200).
        assert!(answers(Ask::Ability(0x250), &action(6, 0x50, None)));
        assert!(!answers(Ask::Ability(0x250), &action(6, 0x51, None)));
        assert!(answers(Ask::Item(1856), &action(5, 1856, None)) && answers(Ask::Item(1856), &action(9, 0, None)));
        assert!(!answers(Ask::Item(1856), &action(1, 0, None)));
    }

    #[test]
    fn only_the_asks_own_refusals_answer() {
        // A spell's refusals carry its id; a melee-side "cannot perform action" of ours (param 0) is not one.
        assert!(refusal(Ask::Spell(10), 47, 10) && refusal(Ask::Spell(10), 34, 10));
        assert!(!refusal(Ask::Spell(10), 47, 11));
        assert!(!refusal(Ask::Spell(10), 71, 0));
        assert!(refusal(Ask::Spell(10), 18, 0) && !refusal(Ask::Spell(10), 18, 10));
        // Experience, wear-offs and other chatter are no kind's refusal.
        assert!(!refusal(Ask::Spell(10), 8, 0) && !refusal(Ask::Spell(10), 204, 12));
        // Weapon skills, job abilities and items answer by their own ids.
        assert!(
            refusal(Ask::WeaponSkill(300), 190, 0)
                && refusal(Ask::WeaponSkill(300), 192, 0)
                && refusal(Ask::WeaponSkill(300), 89, 0)
        );
        assert!(!refusal(Ask::WeaponSkill(300), 47, 10));
        assert!(refusal(Ask::Ability(0x250), 88, 0) && !refusal(Ask::Ability(0x250), 88, 10));
        assert!(refusal(Ask::Item(1856), 56, 0) && refusal(Ask::Item(1856), 62, 1856));
        assert!(!refusal(Ask::Item(1856), 62, 1857));
    }
}
