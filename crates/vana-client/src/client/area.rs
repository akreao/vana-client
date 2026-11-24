//! The area display (config 0xc0, Misc. 3): while a spell, ability, weapon skill or item waits for its target, a
//! ring of the area it would hit is drawn around the pick and the actors inside it flash.
//!
//! The prompt opener registers a callback (FUN_10158e30: target object +0xd0, its argument +0x78): magic FUN_1022ce60
//! -> FUN_1022cef0, abilities FUN_102249f0 -> FUN_10224ac0, /weaponskill FUN_10224d70 -> FUN_10224df0, items
//! FUN_101f4b40 -> FUN_101f4b70. Each frame of the field (FUN_10158f90) FUN_101598c0 runs it into target object
//! +0x75..+0x83 and hands the result to our entity (FUN_1009ade0: +0x134 bits 20..24, +0x138 bits 0..21, +0x2a6);
//! FUN_101599c0 does the same for the Geo- spells' ground cursor (FUN_1009af60: +0x138 bits 22..30). Our actor's
//! update (FUN_1008f740) then makes or drops two actors of file 52743 (kinds 0x52 / 0x53, FUN_100d2f60 0xce07), whose
//! `con0` (FUN_100d0fc0) loops the ring, places them each frame (FUN_100ca310 at the pick, FUN_100ca730 at the ground
//! cursor) and flashes the actors inside (FUN_1008f360 -> FUN_1008f500).

use super::{Source, World};
use glam::Vec3;

/// The ring actors' ids (kind 0x52 at the pick, 0x53 at the ground cursor): no entity, target or plate.
pub const RING: u32 = 0x0880_0000;
pub const GEO: u32 = 0x0880_0001;
/// FUN_100d2f60 for kinds 0x52 / 0x53.
const FILE: u32 = 0xce07;

/// Table 0x10365488 (and its copy 0x103654a8): a radius by index.
const RADII: [u16; 16] = [0, 1, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 20, 25, 30, 255];

/// The registered callback: what the open prompt shows the area of.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Of {
    Spell(u16),
    Ability(u16),
    WeaponSkill(u16),
    Item(u16),
}

/// The callbacks' output (target object +0x75..+0x83, FUN_1022e8a0).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Out {
    /// +0x75: 1 a pick without an area, 2 an area around the pick (3: `con1`, never made).
    pub shape: u8,
    /// +0x76: which actors flash (FUN_1008f500's kind in bits 0..3), with the callback's group in bits 4..7.
    pub kind: u8,
    /// +0x77: the radius index (table RADII).
    pub radius: u8,
    /// +0x80: the radius scale, -0x100..0x100 (0 = the table's radius; TP, a harp).
    pub frac: i16,
    /// +0x7c: the pick's server id, 0 none.
    pub target: u32,
    /// +0x82 / +0x83: a Geo- spell's ground cursor ring and its radius index.
    pub geo: bool,
    pub geo_radius: u8,
}

/// Our entity's fields (+0x134 bits 20..24, +0x138, +0x2a6) and the two ring actors (+0x134 bits 18 / 19 with
/// +0xcc / +0xd0, and whether `con0` ran).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Area {
    pub on: bool,
    pub shape: u8,
    pub kind: u8,
    pub radius: u8,
    pub frac: i16,
    pub target: Option<u32>,
    pub geo_on: bool,
    pub geo: bool,
    pub geo_radius: u8,
    ring: Option<bool>,
    geo_ring: Option<(bool, f32)>,
    /// Where each ring was put last (FUN_100ca310 / FUN_100ca730 leave it when they cannot place it).
    ring_at: Option<(Vec3, Vec3)>,
    geo_at: Option<Vec3>,
    /// Target object +0x75..+0x83: what the callback left, read again while none is registered.
    last: Out,
}

/// The callbacks' work record (FUN_1022cef0's locals at -0x28, read by FUN_1022e980 / FUN_1022eb20 / FUN_1022e8a0).
#[derive(Clone, Copy, Debug, Default)]
struct Calc {
    /// +0: the radius index; +4 the area type; +8 the kind (FUN_1022eb20); +0xc the group (0x40 magic, 0x10
    /// abilities, 0x20 /weaponskill, 0x80 items); +0x10 the record's kind; +0x14 the pick; +0x18 the geo radius.
    radius: i32,
    area: u32,
    kind: u32,
    group: u8,
    sel: u32,
    target: u32,
    geo_radius: u32,
    /// +0x1c: the area is the pet's (pet commands, sub-lists 6 / 10 / 0x12); +0x1d..+0x23 FUN_1022e980's picks.
    pet: bool,
    us: bool,
    party: bool,
    pet_target: bool,
    only_pet: bool,
    sub12: bool,
    single: bool,
    current: bool,
    /// +0x27: a Geo- spell's ground cursor ring.
    geo: bool,
}

impl Calc {
    fn new(radius: u8, area: u8, sel: u8, group: u8) -> Self {
        Calc { radius: radius as i32, area: area as u32, kind: 1, group, sel: sel as u32, ..Default::default() }
    }
}

/// FUN_1022e980's table 0x1033d5e4: the pick to take by (+0x22 != 1) * 2 + (+0x1e != 1) * 4 + (+0x1c != 1).
const PICKS: [u8; 8] = [0, 0, 2, 1, 0, 0, 0, 0];
/// FUN_1022eb20's table 0x1033d5ec by (+0x1c == 1) + kind * 2; past it lie function pointers *(as decompiled: the
/// records' kinds are 0..0x11, census test)*.
const KINDS: [u8; 0x24] =
    [4, 4, 1, 2, 1, 2, 1, 2, 3, 3, 3, 3, 1, 2, 1, 2, 1, 2, 1, 7, 5, 5, 1, 2, 0, 0, 3, 3, 3, 3, 6, 6, 0, 0, 0, 0];

/// FUN_101cdae0: our pet's actor (our +0x1fa), by its server id.
fn our_pet(w: &World) -> Option<u32> {
    (w.my_pet != 0).then(|| w.entity_at(w.my_pet).map(|(id, _)| id)).flatten()
}

/// FUN_100875d0: 0x068's pet is the pet we have (+0x1fa == DAT_104840e0).
fn pet_sync(w: &World) -> Option<&super::net::PetSync> {
    w.pet_sync.as_ref().filter(|p| w.my_pet != 0 && p.index == w.my_pet)
}

/// FUN_1022e980: whose place the area is at (+0x14) by the record's kind, the area type and the callback's picks;
/// returns the pick taken (0 the prompt's pick, 1 us, 2 our pet, 3 our pet's target).
fn pick(w: &World, c: &mut Calc) -> u8 {
    match c.sel {
        2..=5 | 0xf => c.party = true,
        7 | 8 => {
            c.party = true;
            c.only_pet = true;
        }
        9 => {
            c.only_pet = true;
            c.us = true;
        }
        1 => c.us = true,
        0xe => c.pet_target = true,
        _ => {}
    }
    if c.area == 2 {
        c.single = true;
    }
    let mut p = PICKS[(!c.single as usize) * 2 + (!c.party as usize) * 4 + (!c.pet as usize)];
    c.target = 0;
    if !c.pet && c.only_pet {
        p = 0;
    }
    if c.sub12 && c.pet_target {
        p = 3;
    }
    if c.current {
        p = 2;
    }
    if c.us {
        p = c.pet as u8 + 1;
    }
    let pet = our_pet(w);
    match p {
        // FUN_10159280: the prompt's pick (its party cursor's member, else the actor it points at).
        0 if !c.pet || pet.is_some() => {
            c.target = if w.party_cursor.is_some() { super::target::cursor_pick(w) } else { w.target }.unwrap_or(0)
        }
        1 if !c.pet || pet.is_some() => c.target = w.my_id,
        2 => c.target = pet.unwrap_or(0),
        // FUN_10087630: 0x068's target, unless that is the pet itself.
        3 if let Some(pet) = pet => {
            let t = pet_sync(w).map_or(0, |s| s.target);
            if pet != t {
                c.target = t;
            }
        }
        _ => {}
    }
    p
}

/// FUN_101cd7f0: an actor the area treats as an enemy (0x401 / 1). It is FUN_101cc220 (target::hostile) but for us:
/// the teams match ourselves, and only our charmed flag (+0x12c bit 13) is read.
fn enemy(w: &World, id: u32) -> bool {
    if id == w.my_id {
        return w.my_plate.charmed;
    }
    w.entities.get(&id).is_some_and(|e| super::target::hostile(w, id, e))
}

/// FUN_1022eb20: the flashed actors' kind (+8) from the record's kind and the pick.
fn kind_of(w: &World, c: &mut Calc) {
    c.kind = match KINDS.get((c.pet as usize + c.sel as usize * 2) & 0xff).copied().unwrap_or(8) {
        0 => 1,
        1 => 2,
        2 => 3,
        3 => 7,
        // The pick's actor: an enemy 7, else 2 / 3 by +0x1c; none (no pick, no actor) 0.
        4 if c.target != 0 && (c.target == w.my_id || w.entities.contains_key(&c.target)) => {
            if enemy(w, c.target) {
                7
            } else {
                2 + c.pet as u32
            }
        }
        5 => 5 + c.pet as u32,
        6 => 8,
        7 => 9,
        _ => 0,
    };
}

/// FUN_1022e8a0: the record into the outputs; `frac` is +0x80 as the callback left it.
fn pack(c: &Calc, frac: i16) -> Out {
    let mut area = c.area;
    if frac != 0 && area == 0 {
        area = 1;
    }
    if area == 2 {
        area = 0;
    }
    let shape = match area {
        0 => 1,
        1 | 3 => 2,
        _ => 0,
    };
    let kind = if c.kind as u8 != 0 { c.group & 0xf0 | c.kind as u8 } else { 0 };
    Out {
        shape,
        kind,
        radius: if c.radius > 0xf { 0 } else { c.radius as u8 },
        frac,
        target: c.target,
        geo: c.geo,
        geo_radius: if c.geo { c.geo_radius as u8 } else { 0 },
    }
}

/// FUN_101318a0 over the 16 equipment slots (DAT_10579364, refreshed from s2c 0x050 / 0x04F by FUN_101317e0 and
/// FUN_101318d0): each worn item's area kind (Item::area_kind), in slot order.
fn gear(w: &World) -> Vec<u8> {
    let Some(cache) = w.item_info.as_ref() else { return Vec::new() };
    w.inv
        .equip
        .iter()
        .flatten()
        .filter_map(|&(c, i)| w.inv.item(c, i))
        .filter_map(|id| cache.peek(id))
        .map(|it| it.area_kind())
        .collect()
}

fn has(w: &World, effect: u16) -> bool {
    w.buffs.contains(&effect)
}

/// FUN_1022cef0: a spell's area. Effects widen it: Pianissimo, Convergence, Divine Seal (with the Divine Veil trait),
/// Diffusion, Accession, Manifestation, Theurgic Focus, Elemental Seal (Meteor), Majesty; gear: Yagrush (the -na
/// family), a harp (songs by String Instrument skill against the song's level row of levc), Yagyu Darkblade
/// (Utsusemi); a Geo- spell takes its radius from +0x53 (Widened Compass x3) and Entrust's party kind.
pub fn spell(w: &World, id: u16) -> Out {
    let none = Out::default();
    if super::monstrosity::active(w) {
        return none;
    }
    let Some(acts) = w.actions.as_ref() else { return none };
    let Some(r) = acts.spells.get(id as usize) else { return none };
    let mut c = Calc::new(r.aoe_radius, r.area, r.aoe_kind, 0x40);
    let mut frac = 0i16;
    let geomancy = r.kind == 7;
    let free = r.flags & 0x20 != 0;
    let wide = |c: &mut Calc| (c.area, c.radius) = (1, 8);
    let family = r.aoe_bits & 0x7f_ff80 == 0x200;
    if has(w, 409) && r.kind == 5 && matches!(r.aoe_kind, 2 | 3) {
        c.area = 0;
    }
    if has(w, 355) && r.kind == 6 && r.aoe_bits & 7 == 0 && (r.aoe_bits as u8 & 0x78) < 0x39 {
        c.area = 0;
    }
    // FUN_10224970(0x445): ability-table bit 0x645 (0x0AC's trait 0x45, Divine Veil).
    let veil = super::monstrosity::ability_bits(w).is_some_and(|b| b.get(0x645 / 8).is_some_and(|x| x & 1 << 5 != 0));
    if has(w, 78) && veil && r.kind == 1 && (r.id == 0x8f || family) {
        wide(&mut c);
    }
    if has(w, 356) && r.area == 0 && r.kind == 6 && r.id != 0x225 && r.id != 0x2c7 && r.aoe_kind == 1 {
        wide(&mut c);
    }
    if has(w, 366) && r.aoe_bits & 0x80_0000 != 0 {
        wide(&mut c);
    }
    if has(w, 367) && r.aoe_bits & 0x100_0000 != 0 {
        wide(&mut c);
    }
    if has(w, 519) && r.aoe_bits & 0x2000_0000 != 0 {
        c.radius -= 4;
        if c.radius < 1 {
            c.radius = 1;
        }
    }
    if has(w, 79) && r.id == 0xda {
        wide(&mut c);
    }
    if has(w, 621) && (1..7).contains(&r.id) | (0x2b..0x30).contains(&r.id) {
        wide(&mut c);
    }
    let compass = has(w, 508);
    for k in gear(w) {
        match k {
            1 if r.kind == 1 && (r.id == 0x8f || family) => {
                wide(&mut c);
                break;
            }
            // The song's level row (+0x22, the bard's level) of levc / 10 against our String Instrument skill.
            4 if r.kind == 5
                && let Some(&lv) = acts.levc.get(r.levels[10] as u16 as usize)
                && lv / 10 != 0 =>
            {
                let skill = w.skill_base.map_or(0, |s| s[41]) as u32 & 0x7fff;
                let f = (((skill << 8) / (lv / 10) as i16 as u32) as i16 as i32 - 0x100) as i16;
                frac = f.clamp(0, 0x100);
                break;
            }
            5 if r.kind == 4 && matches!(r.id, 0x152..=0x154) => {
                wide(&mut c);
                break;
            }
            _ => {}
        }
    }
    if geomancy {
        if !free {
            if has(w, 584) {
                c.sel = 6;
            }
        } else {
            c.geo = true;
        }
        let v = r.geo_radius as u16 * if compass { 3 } else { 1 };
        let mut k = 0;
        while k < 16 && v * 3 + 3 >= RADII[k] {
            k += 1;
        }
        let k = k.saturating_sub(1);
        c.radius = k as i32;
        c.area = 1;
        c.geo_radius = k as u32;
    }
    pick(w, &mut c);
    kind_of(w, &mut c);
    pack(&c, frac)
}

/// FUN_10224cf0: the scale at `tp` between the record's values at 1000, 2000 and 3000.
fn tp_scale(tp: i32, s: [i16; 3]) -> i16 {
    let s = s.map(|v| v as i32);
    let tp = tp.clamp(1000, 3000);
    (if tp < 2000 { (s[1] - s[0]) * (tp - 1000) / 1000 + s[0] } else { (s[2] - s[1]) * (tp - 2000) / 1000 + s[1] })
        as i16
}

/// FUN_10224ac0: an ability's area. Contradance on Curing Waltz (0x2c2); gear: Epeolatry (0x375), Luzaf's Ring (the
/// Phantom Rolls, +0x1c 0x25: radius 16); pet commands and sub-lists 6 / 10 / 0x12 are the pet's; unless +0x15 the
/// radius scales with the TP (the pet's when the area is at the pet or its target).
pub fn ability(w: &World, id: u16) -> Out {
    if super::monstrosity::active(w) {
        return Out::default();
    }
    let Some(a) = w.actions.as_ref().and_then(|x| x.abilities.get(id as usize)) else { return Out::default() };
    let mut c = Calc::new(a.aoe_radius, a.area, a.aoe_kind, 0x10);
    if has(w, 582) && a.id == 0x2c2 {
        (c.area, c.radius) = (1, 8);
    }
    for k in gear(w) {
        match k {
            2 if a.id == 0x375 => {
                (c.area, c.radius) = (1, 8);
                break;
            }
            3 if a.w1c == 0x25 => {
                c.radius = 0xb;
                break;
            }
            _ => {}
        }
    }
    match a.kind {
        2 | 6 | 10 => c.pet = true,
        0x12 => (c.pet, c.sub12) = (true, true),
        _ => {}
    }
    if a.w1c == 0x2b && a.w1e == 6 && matches!(a.w22, 0xbe5 | 0xbeb | 0xbec) {
        c.current = true;
    }
    let p = pick(w, &mut c);
    let mut frac = 0;
    if a.tp_fixed == 0 {
        // FUN_10087610 (the pet's 0x068 TP) or FUN_100e8cc0(0) +0x28 (our party record's TP).
        let tp = if matches!(p, 2 | 3) { pet_sync(w).map_or(0, |s| s.tp as i32) } else { w.party.own().tp as i32 };
        frac = tp_scale(tp, a.tp_scale);
    }
    kind_of(w, &mut c);
    pack(&c, frac)
}

/// FUN_10224df0 (/weaponskill's prompt): the record's area alone.
pub fn weapon_skill(w: &World, id: u16) -> Out {
    if super::monstrosity::active(w) {
        return Out::default();
    }
    let Some(a) = w.actions.as_ref().and_then(|x| x.abilities.get(id as usize)) else { return Out::default() };
    let mut c = Calc::new(a.aoe_radius, a.area, a.aoe_kind, 0x20);
    pick(w, &mut c);
    kind_of(w, &mut c);
    pack(&c, 0)
}

/// FUN_101f4b70: an item's area (its record's +0x1b0..+0x1b2).
pub fn item(w: &World, id: u16) -> Out {
    if super::monstrosity::active(w) {
        return Out::default();
    }
    let Some(it) = w.item_info.as_ref().and_then(|c| c.peek(id)) else { return Out::default() };
    let [radius, area, sel] = it.area();
    let mut c = Calc::new(radius, area, sel, 0x80);
    pick(w, &mut c);
    kind_of(w, &mut c);
    pack(&c, 0)
}

/// The prompt opener's FUN_10158e30: the open 3D prompt shows `of`'s area. The target object has one slot (+0xd0):
/// the last registration wins, so the party cursor's goes.
pub fn register(w: &mut World, of: Of) {
    if let Some(s) = w.prompt.as_mut() {
        s.area = Some(of);
        if let Some(pc) = w.party_cursor.as_mut() {
            pc.area = None;
        }
    }
}

/// FUN_10158e30 from the party cursor's dry run (0x10076ef0 / 0x10076f43): the cursor shows `of`'s area, and the
/// 3D prompt's registration goes.
pub fn register_cursor(w: &mut World, of: Of) {
    if let Some(pc) = w.party_cursor.as_mut() {
        pc.area = Some(of);
        if let Some(s) = w.prompt.as_mut() {
            s.area = None;
        }
    }
}

/// Target object +0xd0: the callback the 3D prompt or the party cursor registered (at most one holds one).
fn registered(w: &World) -> Option<Of> {
    w.prompt.as_ref().and_then(|s| s.area).or_else(|| w.party_cursor.as_ref().and_then(|pc| pc.area))
}

/// FUN_1009aec0: the radius in yalms, (scale + 0x100) x RADII[index] / 0x100, 0..40.
pub fn radius(a: &Area) -> f32 {
    let r = ((a.frac as i32 + 0x100) * RADII[a.radius as usize & 0xf] as i32) >> 8;
    if a.frac < -0x100 { 0.0 } else { r.clamp(0, 40) as f32 }
}

/// FUN_1009af20: the ground cursor ring's radius, RADII[index] up to 40.
fn geo_radius(a: &Area) -> f32 {
    RADII[a.geo_radius as usize & 0xf].min(40) as f32
}

/// FUN_101598c0 / FUN_101599c0 then FUN_1009ade0 / FUN_1009af60: the callback's output onto our fields. A new shape
/// and kind at once while on drop the ring (FUN_10092710), the next frame makes it again; so does a new geo flag.
fn apply(w: &mut World, o: Out) {
    let callback = registered(w).is_some();
    let cfg = w.game.get(0xc0) == 1;
    let ground = w.ground_cursor.is_some();
    let on = callback && !ground && cfg && o.target != 0 && o.shape != 0 && o.kind != 0;
    let a = &mut w.aoe;
    if on && a.on && o.shape != a.shape && o.kind != a.kind {
        drop_ring(w, RING);
    }
    let a = &mut w.aoe;
    (a.on, a.shape, a.kind, a.radius, a.frac) = (on, o.shape, o.kind, o.radius & 0xf, o.frac);
    // +0x2a6: the pick's actor; FUN_100983a0 wants it modelled (+0x120 bit 9, a skeleton actor), else off.
    a.target = (o.target != 0).then_some(o.target);
    let modelled = a.target.is_some_and(|t| w.actors.get(&t).is_some_and(|x| x.character.is_some()));
    if !modelled {
        w.aoe.on = false;
    }
    let geo_on = callback && ground && o.geo && cfg;
    let a = &mut w.aoe;
    if geo_on && a.geo_on && o.geo != a.geo {
        drop_ring(w, GEO);
    }
    let a = &mut w.aoe;
    (a.geo_on, a.geo, a.geo_radius) = (geo_on, o.geo, o.geo_radius & 0xf);
}

/// FUN_10092710 / FUN_100926c0: the ring actor goes with its effects.
fn drop_ring(w: &mut World, id: u32) {
    if id == RING {
        w.aoe.ring = None;
    } else {
        w.aoe.geo_ring = None;
    }
    w.actors.remove(&id);
    w.effects.forget_actor(id);
    w.event_looks.retain(|(i, _)| *i != id);
}

/// FUN_1008e1e0: the actor whose fellow (+0x2a0) `id` is, when `id` is owned by a player (+0x1d0 bit 8).
fn fellow_master(w: &World, id: u32) -> Option<u32> {
    let e = w.entities.get(&id)?;
    if !e.marks.charm {
        return None;
    }
    if w.my_fellow != 0 && w.my_fellow == e.index {
        return Some(w.my_id);
    }
    w.entities.iter().find(|(o, x)| x.fellow == e.index && w.actors.contains_key(o)).map(|(o, _)| *o)
}

/// FUN_1008e180: the actor whose pet (+0x1fa) or fellow `id` is, not a trust (+0x12c bit 22), when `id` is owned.
fn owner(w: &World, id: u32) -> Option<u32> {
    let e = w.entities.get(&id)?;
    if !e.marks.charm {
        return None;
    }
    if (w.my_pet != 0 && w.my_pet == e.index) || (w.my_fellow != 0 && w.my_fellow == e.index) {
        return Some(w.my_id);
    }
    w.entities
        .iter()
        .find(|(o, x)| (x.pet == e.index || x.fellow == e.index) && !x.marks.trust && w.actors.contains_key(o))
        .map(|(o, _)| *o)
}

/// FUN_10204870: `a` is in `b`'s party of the window's three (both listed); FUN_10204800(id, 0, 1): listed at all.
fn same_party(slots: &super::party::Slots, a: u32, b: u32) -> bool {
    let Some(k) = slots.slots.iter().position(|s| s.as_ref().is_some_and(|m| b != 0 && m.id == b)) else {
        return false;
    };
    let p = k / 6;
    slots.slots[p * 6..p * 6 + 6].iter().flatten().any(|m| m.id == a)
}

fn listed(slots: &super::party::Slots, id: u32) -> bool {
    slots.slots.iter().flatten().any(|m| m.id == id)
}

/// FUN_10097a60: K.O. (+0x170 2 / 3).
fn down(w: &World, id: u32) -> bool {
    let s = if id == w.my_id { w.my_motion.status } else { w.entities.get(&id).map_or(0, |e| e.motion.status) };
    matches!(s, 2 | 3)
}

/// FUN_1008f500: whether actor `cand` flashes for the area at `tgt` (with `pa`: the pick when it is someone's fellow).
fn marked(w: &World, slots: &super::party::Slots, tgt: Option<u32>, cand: u32, kind: u8, pa: Option<u32>) -> bool {
    let no_pets = kind >> 7 & 1 != 0;
    let k = kind & 0xf;
    let master = fellow_master(w, cand);
    let fellow = match pa {
        Some(pa) if pa == cand => true,
        Some(pa) => {
            if fellow_master(w, pa) != Some(cand) {
                return false;
            }
            true
        }
        None => match master {
            Some(m) => {
                if Some(m) != tgt {
                    return false;
                }
                true
            }
            None => false,
        },
    };
    let (mut ours, mut pet_owner, mut c) = (false, false, cand);
    if fellow {
        if let Some(m) = master {
            c = m;
        }
    } else if let Some(o) = owner(w, cand) {
        if no_pets {
            return false;
        }
        if o == w.my_id {
            c = o;
            ours = true;
        }
    } else if let Some(p) = our_pet(w)
        && owner(w, p) == Some(cand)
    {
        pet_owner = true;
    }
    let t = tgt.unwrap_or(0);
    let (foe, ko) = (enemy(w, c), down(w, c));
    match k {
        2 => !ko && !foe && !ours && same_party(slots, c, t),
        3 => !ko && !foe && same_party(slots, c, t),
        5 => !ko && !foe && !ours && listed(slots, c),
        6 => !ko && !foe && listed(slots, c),
        7 => !ko && foe,
        8 => !foe && same_party(slots, c, t),
        9 => !ko && !foe && (ours || pet_owner),
        _ => false,
    }
}

/// An actor's drawn place for the area (vt+0x1bc of its model), and its facing.
fn place(w: &World, id: u32) -> Option<(Vec3, Vec3)> {
    let my_dir = w.net.me.lock().map(|m| m.dir).unwrap_or(0);
    super::base_place(w, id, my_dir).map(|p| (p.pos, p.forward))
}

/// FUN_1008f360 at a ring's place: with config 0x99 bit 7 clear and the area around the pick, every modelled actor
/// within the radius (and 8 up or down) that FUN_1008f500 takes flashes (model +0x88 |= 0x600).
fn flash(w: &mut World, at: Vec3) {
    if w.game.get(0x99) & 0x80 != 0 || w.aoe.shape != 2 {
        return;
    }
    let r = radius(&w.aoe);
    let mut tgt = w.aoe.target.filter(|t| *t == w.my_id || w.entities.contains_key(t));
    let mut pa = None;
    if let Some(t) = tgt {
        if fellow_master(w, t).is_some() {
            pa = Some(t);
        }
        if let Some(o) = owner(w, t) {
            tgt = Some(o);
        }
    }
    let slots = w.party.slots();
    let kind = w.aoe.kind;
    let mut ids: Vec<u32> = w.entities.keys().copied().collect();
    ids.push(w.my_id);
    let hits: Vec<u32> = ids
        .into_iter()
        .filter(|id| w.actors.get(id).is_some_and(|a| a.character.is_some()))
        .filter(|&id| marked(w, &slots, tgt, id, kind, pa))
        .filter(|&id| {
            place(w, id).is_some_and(|(p, _)| {
                let d = p - at;
                d.y * d.y <= 64.0 && d.x * d.x + d.z * d.z <= r * r
            })
        })
        .collect();
    for id in hits {
        super::target::force_flash(w, id);
    }
}

/// Per frame in the field (FUN_10158f90 -> FUN_101598c0 / FUN_101599c0, then our actor's FUN_1008f740 block): the
/// callback, the rings made, `con0` once each is in, their places and the flash.
pub fn update(w: &mut World) {
    let o = match registered(w) {
        Some(Of::Spell(id)) => spell(w, id),
        Some(Of::Ability(id)) => ability(w, id),
        Some(Of::WeaponSkill(id)) => weapon_skill(w, id),
        Some(Of::Item(id)) => item(w, id),
        None => w.aoe.last,
    };
    w.aoe.last = o;
    apply(w, o);
    // A ring the actor table lost (a zone change, a model reload) is made again.
    let pending = |w: &World, id: u32| w.actors.contains_key(&id) || w.event_looks.iter().any(|(i, _)| *i == id);
    if w.aoe.ring.is_some() && !pending(w, RING) {
        w.aoe.ring = None;
    }
    if w.aoe.geo_ring.is_some() && !pending(w, GEO) {
        w.aoe.geo_ring = None;
    }
    // Kind 0x52: made while on; `con0` for an area around the pick, `con1` for shape 3 (FUN_100d0fc0).
    if !w.aoe.on {
        if w.aoe.ring.is_some() {
            drop_ring(w, RING);
        }
    } else {
        if w.aoe.ring.is_none() {
            log::debug!("area: ring at {:?}, shape {} radius {}", w.aoe.target, w.aoe.shape, radius(&w.aoe));
            w.actors.remove(&RING);
            w.effects.forget_actor(RING);
            w.event_looks.push((RING, Source::File(FILE)));
            w.aoe.ring = Some(false);
            w.aoe.ring_at = None;
        }
        if w.aoe.ring == Some(false) && w.actors.get(&RING).is_some_and(|a| a.character.is_some()) {
            match w.aoe.shape {
                2 => w.effects.on_scheduler_quiet(RING, u32::from_le_bytes(*b"con0")),
                3 => w.effects.on_scheduler_quiet(RING, u32::from_le_bytes(*b"con1")),
                _ => {}
            }
            w.aoe.ring = Some(true);
        }
        // FUN_100ca310: at the pick's model, its facing; x / z the radius, y 0.15 of it + 1.
        if let Some(p) = w.aoe.target.and_then(|t| place(w, t)) {
            w.aoe.ring_at = Some(p);
        }
        let r = radius(&w.aoe);
        if let Some(a) = w.actors.get_mut(&RING) {
            a.axes = Some(Vec3::new(r, r * 0.15 + 1.0, r));
        }
        if let Some((at, _)) = w.aoe.ring_at {
            flash(w, at);
        }
    }
    // Kind 0x53: at the ground cursor, `con0`; x / z the geo radius, y 1.
    if !w.aoe.geo_on {
        if w.aoe.geo_ring.is_some() {
            drop_ring(w, GEO);
        }
        return;
    }
    if w.aoe.geo_ring.is_none() {
        // Made at our facing; nothing turns it after (FUN_100ca730 sets the place only).
        let yaw = place(w, w.my_id).map_or(0.0, |(_, f)| (-f.z).atan2(f.x));
        w.actors.remove(&GEO);
        w.effects.forget_actor(GEO);
        w.event_looks.push((GEO, Source::File(FILE)));
        w.aoe.geo_ring = Some((false, yaw));
        w.aoe.geo_at = None;
    }
    if let Some((false, yaw)) = w.aoe.geo_ring
        && w.actors.get(&GEO).is_some_and(|a| a.character.is_some())
    {
        w.effects.on_scheduler_quiet(GEO, u32::from_le_bytes(*b"con0"));
        w.aoe.geo_ring = Some((true, yaw));
    }
    // FUN_1015b250: the cursor's place (+0x10..+0x18).
    if let Some((_, at)) = w.ground_cursor {
        w.aoe.geo_at = Some(at);
    }
    let r = geo_radius(&w.aoe);
    if let Some(a) = w.actors.get_mut(&GEO) {
        a.axes = Some(Vec3::new(r, 1.0, r));
    }
    if let Some(at) = w.aoe.geo_at {
        flash(w, at);
    }
}

/// The rings' places for the draw (FUN_100ca310 / FUN_100ca730).
pub fn place_of(w: &World, id: u32) -> Option<(Vec3, Vec3)> {
    match id {
        RING => w.aoe.ring_at,
        GEO => {
            let yaw = w.aoe.geo_ring?.1;
            w.aoe.geo_at.map(|p| (p, Vec3::new(yaw.cos(), 0.0, -yaw.sin())))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radii() {
        let mut a = Area { radius: 7, ..Default::default() };
        assert_eq!(radius(&a), 8.0);
        a.frac = 0x100;
        assert_eq!(radius(&a), 16.0);
        a.frac = -0x80;
        assert_eq!(radius(&a), 4.0);
        // Capped at 40; index 15 is 255.
        (a.radius, a.frac) = (15, 0);
        assert_eq!(radius(&a), 40.0);
        let g = Area { geo_radius: 11, ..Default::default() };
        assert_eq!(geo_radius(&g), 16.0);
    }

    #[test]
    fn tp_scales() {
        let s = [-0x40, 0, 0x80];
        assert_eq!(tp_scale(0, s), -0x40);
        assert_eq!(tp_scale(1500, s), -0x20);
        assert_eq!(tp_scale(2000, s), 0);
        assert_eq!(tp_scale(2500, s), 0x40);
        assert_eq!(tp_scale(3000, s), 0x80);
        assert_eq!(tp_scale(3500, s), 0x80);
    }

    #[test]
    fn packing() {
        // Area type 2 is a single target: no area; a nonzero scale on type 0 makes it an area.
        let c = Calc { area: 2, kind: 7, group: 0x40, radius: 8, ..Default::default() };
        assert_eq!(pack(&c, 0), Out { shape: 1, kind: 0x47, radius: 8, ..Default::default() });
        let c = Calc { area: 0, kind: 3, group: 0x10, radius: 20, ..Default::default() };
        assert_eq!(pack(&c, 5), Out { shape: 2, kind: 0x13, radius: 0, frac: 5, ..Default::default() });
        let c = Calc { area: 1, kind: 0, group: 0x80, geo: true, geo_radius: 6, ..Default::default() };
        assert_eq!(pack(&c, 0), Out { shape: 2, kind: 0, geo: true, geo_radius: 6, ..Default::default() });
    }

    /// File 52743: `con0` loops the ring (`clp0`'s three generators, attached to the actor), and the file loads as the
    /// actor's model.
    #[test]
    fn ring_file() {
        let Ok(index) = crate::dat::DatIndex::open(&crate::config::Config::load().ffxi_dir()) else { return };
        let Ok(bytes) = index.read(FILE) else { return };
        let lib = crate::fx::lib::Library::load(FILE, bytes.clone()).unwrap();
        let ops = |name: &[u8; 4]| {
            let n = lib.find_global(crate::dat::generator::kind::SCHEDULER, u32::from_le_bytes(*name)).unwrap();
            crate::dat::sched::ops(&bytes[lib.tree.nodes[n].data.clone()])
                .iter()
                .map(|o| (o.code, o.name().unwrap_or("").to_string()))
                .collect::<Vec<_>>()
        };
        assert!(ops(b"con0").contains(&(0x73, "clp0".to_string())));
        assert!(ops(b"clp0").contains(&(0x02, "cc00".to_string())));
        assert!(lib.gens.iter().all(|g| g.attach_mode() == 1));
        // The ring is made through Source::File: it needs its own skeleton, else `con0` never starts.
        crate::dat::npc::load_file_model(&index, FILE).unwrap();
    }

    /// FUN_1022e980's and FUN_1022eb20's tables (0x1033d5e4, 0x1033d5ec, back to back) and the radius table
    /// (0x10365488) as the DLL holds them.
    #[test]
    fn dll_tables() {
        let dir = crate::config::Config::load().ffxi_dir();
        let Ok(dll) = std::fs::read(crate::dat::ffximain(&dir)) else { return };
        let picks: Vec<u8> = PICKS.iter().chain(KINDS.iter()).copied().collect();
        assert!(dll.windows(picks.len()).any(|x| x == picks.as_slice()));
        let radii: Vec<u8> = RADII.iter().flat_map(|r| r.to_le_bytes()).collect();
        assert!(dll.windows(radii.len()).any(|x| x == radii.as_slice()));
    }

    /// The records' area kinds stay inside FUN_1022eb20's table (0..0x11) and the radii inside RADII.
    #[test]
    fn record_census() {
        let Ok(index) = crate::dat::DatIndex::open(&crate::config::Config::load().ffxi_dir()) else { return };
        let Ok(acts) = crate::dat::actions::Actions::load(&index) else { return };
        assert!(acts.spells.iter().all(|s| s.aoe_kind < 0x12 && s.aoe_radius < 16));
        assert!(acts.abilities.iter().all(|a| a.aoe_kind < 0x12 && a.aoe_radius < 16));
        assert!(acts.levc.len() > 99);
    }
}
