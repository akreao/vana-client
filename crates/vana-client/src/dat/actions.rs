//! Spell and ability records: file 81, chunks 0x49 ("mgc_", 100-byte SpellInfo records, DAT_10667a74) and 0x53
//! ("comm", 0x30-byte AbilityInfo records, DAT_1066714c), each record masked as FUN_1021e870 undoes it.

use super::DatIndex;
use anyhow::{Context, Result};

const FILE: u32 = 81;
const SPELL_CHUNK: u8 = 0x49;
const ABILITY_CHUNK: u8 = 0x53;
const SPELL_SIZE: usize = 100;
const ABILITY_SIZE: usize = 0x30;

/// FUN_1021e870 (POLUtils DecodeDataBlockMask): every byte but 2, 11 and 12 rotated by a count from their bits.
pub(crate) fn unmask(r: &mut [u8]) {
    let pc = |b: u8| b.count_ones() as i32;
    let s = (pc(r[2]) + pc(r[12]) - pc(r[11])).abs() % 5;
    for (i, b) in r.iter_mut().enumerate() {
        if matches!(i, 2 | 11 | 12) {
            continue;
        }
        *b = match s {
            0 => b.rotate_left(1),
            1 => b.rotate_right(1),
            2 => b.rotate_left(2),
            3 => b.rotate_right(2),
            _ => b.rotate_left(3),
        };
    }
}

fn records(index: &DatIndex, kind: u8, size: usize) -> Result<Vec<Vec<u8>>> {
    let file = index.read(FILE)?;
    let c = super::chunks(&file)?
        .into_iter()
        .find(|c| c.kind == kind)
        .with_context(|| format!("file {FILE}: no chunk {kind:#x}"))?;
    Ok(c.data
        .chunks_exact(size)
        .map(|r| {
            let mut r = r.to_vec();
            unmask(&mut r);
            r
        })
        .collect())
}

fn u16_at(r: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([r[o], r[o + 1]])
}

#[derive(Clone, Debug, Default)]
pub struct Spell {
    /// +0: the spell id (0 = unused record); names are by record index, which equals it.
    pub id: u16,
    /// +2: 1 white, 2 black, 3 summoning, 4 ninjutsu, 5 song, 6 blue, 7 geomancy, 8 trust.
    pub kind: u16,
    pub element: u16,
    /// +6: valid target bits (1 self, 2 player, 4 party, 8 alliance, 0x10 PC, 0x20 enemy ...).
    pub targets: u16,
    pub skill: u16,
    pub mp: u16,
    /// +0xc / +0xd: cast time and recast in quarter seconds.
    pub cast: u8,
    pub recast: u8,
    /// +0xe: level per job 0..23, -1 when the job can't learn it.
    pub levels: [i16; 24],
    /// +0x3e: the auto-sort position; +0x40 / +0x42: list icons (normal, HD from 1000).
    pub sort: u16,
    pub icon: u16,
    pub icon_hd: u16,
    /// +0x44: 8 SCH main only, 0x20 GEO main only (bit 5 also the free 3D cursor); +0x45: range index.
    pub flags: u8,
    pub range: u8,
    /// +0x47: nonzero for a spell that always hits an area (+0x46 its radius) *(inferred: matches LSB's AOE)*.
    pub area: u8,
    /// +0x5c: per main job, the level is in job points (needs level 99).
    pub jp_mask: u32,
}

/// Table 0x103644a8: spell range by record +0x45.
pub const RANGES: [u16; 16] = [0, 1, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 20, 25, 30, 255];

#[derive(Clone, Debug, Default)]
pub struct Ability {
    /// +0: id in the name table (weapon skills 0..0x1ff, job abilities 0x200.., pet commands 0x400..).
    pub id: u16,
    /// +2: 1 job ability, 2 pet command, 3 weapon skill, 4 trait, 0x14 Monstrosity; 6..0x17 sub-lists.
    pub kind: u8,
    /// +3 / +4: the list icons, standard and HD (FUN_10224a80 -> FUN_10220d40).
    pub icon: u8,
    pub icon_hd: u16,
    /// +6: MP cost, or Ready charges.
    pub cost: u16,
    /// +8: the 0x119 timer id (900 none).
    pub timer: u16,
    /// +0xa: valid targets (spell bits).
    pub targets: u16,
    /// +0xc: TP cost (255 none); the Monstrosity list reads it as a u16 (FUN_1014feb0 kind 9).
    pub tp: u8,
    pub tp16: u16,
    /// +0xe: a row opening the sub-list of this type (skipped by /ja); +0xf level (Monstrosity).
    pub sub: u8,
    pub level: u8,
}

pub struct Actions {
    pub spells: Vec<Spell>,
    pub abilities: Vec<Ability>,
}

impl Actions {
    pub fn load(index: &DatIndex) -> Result<Actions> {
        let spells = records(index, SPELL_CHUNK, SPELL_SIZE)?
            .iter()
            .map(|r| Spell {
                id: u16_at(r, 0),
                kind: u16_at(r, 2),
                element: u16_at(r, 4),
                targets: u16_at(r, 6),
                skill: u16_at(r, 8),
                mp: u16_at(r, 0xa),
                cast: r[0xc],
                recast: r[0xd],
                levels: std::array::from_fn(|j| u16_at(r, 0xe + 2 * j) as i16),
                sort: u16_at(r, 0x3e),
                icon: u16_at(r, 0x40),
                icon_hd: u16_at(r, 0x42),
                flags: r[0x44],
                range: r[0x45],
                area: r[0x47],
                jp_mask: u32::from_le_bytes(r[0x5c..0x60].try_into().unwrap()),
            })
            .collect();
        let abilities = records(index, ABILITY_CHUNK, ABILITY_SIZE)?
            .iter()
            .map(|r| Ability {
                id: u16_at(r, 0),
                kind: r[2],
                icon: r[3],
                icon_hd: u16_at(r, 4),
                cost: u16_at(r, 6),
                timer: u16_at(r, 8),
                targets: u16_at(r, 0xa),
                tp: r[0xc],
                tp16: u16_at(r, 0xc),
                sub: r[0xe],
                level: r[0xf],
            })
            .collect();
        Ok(Actions { spells, abilities })
    }

    pub fn spell(&self, id: u16) -> Option<&Spell> {
        self.spells.get(id as usize).filter(|s| s.id != 0)
    }

    /// FUN_1022b7d0: the main or sub job reaches the spell's level; a job-point spell needs main 99 and the points.
    pub fn can_cast(&self, id: u16, main: (u8, u8), sub: (u8, u8), job_points: u16) -> bool {
        let Some(s) = self.spell(id) else { return false };
        let lv = |job: u8| s.levels.get(job as usize).copied().unwrap_or(-1);
        let reach = |(job, level): (u8, u8)| lv(job) >= 0 && level as i16 >= lv(job);
        if s.jp_mask & (1 << main.0) != 0 {
            main.1 >= 99 && lv(main.0) >= 0 && job_points as i16 >= lv(main.0)
        } else {
            reach(main) || reach(sub)
        }
    }

    /// FUN_101021a0, the magic check's job rule: `can_cast`, and by the spell's +0x44 and our effects: bit 5 GEO main
    /// only; bit 3 on SCH needs Tabula Rasa (377); bit 4 needs Elemental Seal (79) unless the main job is 23; bit 2 on
    /// SCH needs the addendum of the spell's school (white 401, black 402) or Enlightenment (416). A job that fails
    /// its rule leaves the spell to the other job, then to the job points.
    pub fn can_cast_now(&self, id: u16, main: (u8, u8), sub: (u8, u8), job_points: u16, effects: &[u16]) -> bool {
        let Some(s) = self.spell(id) else { return false };
        let has = |e: u16| effects.contains(&e);
        let reach = |(job, level): (u8, u8)| s.levels.get(job as usize).is_some_and(|&l| l >= 0 && l <= level as i16);
        let addendum = (s.kind == 1 && (has(401) || has(416))) || (s.kind == 2 && (has(402) || has(416)));
        if reach(main) {
            if s.flags & 0x20 != 0 {
                return main.0 == 21;
            }
            if main.0 == 20 && s.flags & 8 != 0 {
                return has(377);
            }
            if s.flags & 0x10 != 0 {
                return main.0 == 23 || has(79);
            }
            if main.0 != 20 || s.flags & 4 == 0 || addendum {
                return true;
            }
        }
        if reach(sub) {
            if s.flags & 0x20 != 0 {
                return false;
            }
            if sub.0 != 20 {
                return true;
            }
            if s.flags & 8 != 0 {
                return has(377);
            }
            if s.flags & 4 == 0 || addendum {
                return true;
            }
        }
        let lv = s.levels.get(main.0 as usize).copied().unwrap_or(-1);
        s.jp_mask & (1 << (main.0 & 0x1f)) != 0 && main.1 >= 99 && lv >= 0 && lv as u16 <= job_points
    }
}

/// FUN_10101e50 / FUN_10102050: an MP cost under our effects, by the spell's type (+2; 99 for an ability's cost) and
/// its +0x44 bits 0 / 1. Manafont (47) or Manawell (229): none. White under Light Arts (358) or Addendum: White (401):
/// x2 with Accession (366) on a bit 0 spell, then half with Penury (360), else 90 %; without the arts 90 % with Tabula
/// Rasa (377), else 120 % under Dark Arts (359) or Addendum: Black (402). Black the same the other way round
/// (Manifestation 367 on bit 1, Parsimony 361). An ability's cost x1.5 with Apogee (583). Each step rounds up
/// (FUN_10102180).
pub fn mp_cost(cost: u16, kind: u16, flags: u8, effects: &[u16]) -> u16 {
    let has = |e: u16| effects.contains(&e);
    let pct = |c: u32, p: u32| (c * p).div_ceil(100);
    if has(47) || has(229) {
        return 0;
    }
    let c = cost as u32;
    let (light, dark) = (has(358) || has(401), has(359) || has(402));
    let school = |own: bool, other: bool, wide: bool, half: bool| {
        if !own {
            return if has(377) {
                pct(c, 90)
            } else if other {
                pct(c, 120)
            } else {
                c
            };
        }
        let c2 = if wide { pct(c, 200) } else { c };
        if half {
            pct(c2, 50)
        } else if wide {
            c2
        } else {
            pct(c2, 90)
        }
    };
    (match kind {
        1 => school(light, dark, has(366) && flags & 1 != 0, has(360)),
        2 => school(dark, light, has(367) && flags & 2 != 0, has(361)),
        99 if has(583) => pct(c, 150),
        _ => c,
    }) as u16
}

/// FUN_10102430: a spell's valid targets under our effects: a song (type 5) on ourselves alone takes the party too
/// with Pianissimo (409), a geomancy spell (type 7) without +0x44 bit 5 with Entrust (584).
pub fn spell_targets(targets: u16, kind: u16, flags: u8, effects: &[u16]) -> u16 {
    let wider = match kind {
        5 => effects.contains(&409),
        7 => flags & 0x20 == 0 && effects.contains(&584),
        _ => false,
    };
    if wider && targets == 1 { 5 } else { targets }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FUN_10102050's branches on a 100 MP spell, and the round up.
    #[test]
    fn mp_cost_by_effects() {
        assert_eq!(mp_cost(100, 1, 0, &[]), 100);
        assert_eq!(mp_cost(100, 1, 0, &[47]), 0);
        assert_eq!(mp_cost(100, 6, 0, &[229]), 0);
        assert_eq!(mp_cost(100, 1, 0, &[358]), 90);
        assert_eq!(mp_cost(100, 1, 0, &[359]), 120);
        assert_eq!(mp_cost(100, 2, 0, &[359]), 90);
        assert_eq!(mp_cost(100, 2, 0, &[401]), 120);
        assert_eq!(mp_cost(100, 1, 0, &[377]), 90);
        assert_eq!(mp_cost(100, 1, 0, &[358, 360]), 50);
        assert_eq!(mp_cost(100, 1, 1, &[358, 366]), 200);
        assert_eq!(mp_cost(100, 1, 0, &[358, 366]), 90);
        assert_eq!(mp_cost(100, 1, 1, &[358, 366, 360]), 100);
        assert_eq!(mp_cost(100, 2, 2, &[402, 367]), 200);
        assert_eq!(mp_cost(100, 2, 0, &[359, 361]), 50);
        assert_eq!(mp_cost(100, 99, 0, &[583]), 150);
        assert_eq!(mp_cost(100, 5, 0, &[358, 583]), 100);
        assert_eq!(mp_cost(7, 1, 0, &[358]), 7);
        assert_eq!(mp_cost(8, 1, 0, &[359]), 10);
    }

    #[test]
    fn targets_by_effects() {
        assert_eq!(spell_targets(1, 5, 0, &[409]), 5);
        assert_eq!(spell_targets(1, 5, 0, &[]), 1);
        assert_eq!(spell_targets(0x20, 5, 0, &[409]), 0x20);
        assert_eq!(spell_targets(1, 7, 0, &[584]), 5);
        assert_eq!(spell_targets(1, 7, 0x20, &[584]), 1);
        assert_eq!(spell_targets(1, 1, 0, &[409, 584]), 1);
    }

    fn real() -> Option<Actions> {
        let index = DatIndex::open(&crate::config::Config::load().ffxi_dir()).ok()?;
        Actions::load(&index).ok()
    }

    /// Cure is white magic for WHM 1; Shantotto (896) a trust any job can call at 1; ids match the record index.
    #[test]
    fn spells_from_file_81() {
        let Some(a) = real() else { return };
        assert_eq!(a.spells.len(), 1024);
        let cure = a.spell(1).unwrap();
        assert_eq!((cure.kind, cure.mp, cure.levels[3], cure.levels[1]), (1, 8, 1, -1));
        let trust = a.spell(896).unwrap();
        assert_eq!((trust.kind, trust.targets, trust.levels[1]), (8, 1, 1));
        assert!(a.spells.iter().enumerate().all(|(i, s)| s.id == 0 || s.id as usize == i));
        assert!(
            a.can_cast(1, (3, 1), (0, 0), 0)
                && !a.can_cast(1, (1, 75), (0, 0), 0)
                && a.can_cast(1, (1, 75), (3, 37), 0)
        );
    }

    /// The schools the battle menu's suggestions keep apart: Dia and Protect white, Bio, Water and Poison black.
    #[test]
    fn white_and_black_kinds() {
        let Some(a) = real() else { return };
        for (id, kind) in [(23, 1), (43, 1), (230, 2), (169, 2), (220, 2)] {
            assert_eq!(a.spell(id).map(|s| s.kind), Some(kind), "spell {id}");
        }
    }
}
