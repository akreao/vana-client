//! `vana-client skillchain-gen`: the Skillchain mod's data (assets/mods/skillchain/data/skillchains.txt) from
//! LandSandBoat's SQL: weapon skill, mob skill (trusts, pets) and blue magic properties and the trusts' skill lists.
//! The mod reads it; the client keeps only this generator.

use anyhow::{Context, Result};
use std::path::Path;

/// Lower case, letters and digits only: "TRUST_Ayame_UC" and "Ayame (UC)" both give "ayameuc".
fn fold(name: &str) -> String {
    name.trim_start_matches("TRUST_")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// The values of each `INSERT INTO <table> VALUES (...)` line (quotes stripped, `@VARS` kept as text).
fn rows(sql: &str, table: &str) -> Vec<Vec<String>> {
    let head = format!("INSERT INTO `{table}` VALUES (");
    sql.lines()
        .filter_map(|l| l.trim_start().strip_prefix(&head))
        .map(|l| {
            let mut out = Vec::new();
            let (mut cur, mut quoted, mut prev) = (String::new(), false, ' ');
            for c in l.chars() {
                match c {
                    '\'' if prev != '\\' => quoted = !quoted,
                    ',' if !quoted => out.push(std::mem::take(&mut cur).trim().to_string()),
                    ')' if !quoted => {
                        out.push(std::mem::take(&mut cur).trim().to_string());
                        break;
                    }
                    _ => cur.push(c),
                }
                prev = c;
            }
            out
        })
        .collect()
}

/// `skillchain-gen`: the mod's data/skillchains.txt from LSB's sql/ tables.
pub fn generate(lsb: &Path, out: &Path) -> Result<()> {
    let read =
        |name: &str| std::fs::read_to_string(lsb.join("sql").join(name)).with_context(|| format!("reading {name}"));
    let n = |v: &str| v.parse::<i64>().unwrap_or(0).max(0);
    let mut text = String::from(
        "# vana-client skillchain-gen from LandSandBoat sql/: weapon_skills, mob_skills, pet_skills, blue_spell_list, mob_skill_lists\n",
    );
    text += "# w id p1 p2 p3 type skilllevel name / m id p1 p2 p3 name / b spell p1 p2 p3 / t trust skill...\n";
    for r in rows(&read("weapon_skills.sql")?, "weapon_skills") {
        // weaponskillid, name, jobs, type, skilllevel, element, animation, animationTime, range, aoe, radius, primary_sc..
        if r.len() >= 14 {
            text += &format!(
                "w {} {} {} {} {} {} {}\n",
                n(&r[0]),
                n(&r[11]),
                n(&r[12]),
                n(&r[13]),
                n(&r[3]),
                n(&r[4]),
                r[1]
            );
        }
    }
    let mut mob = std::collections::BTreeMap::new();
    for r in rows(&read("mob_skills.sql")?, "mob_skills") {
        if r.len() >= 15 && (n(&r[12]) | n(&r[13]) | n(&r[14])) != 0 {
            mob.insert(n(&r[0]), format!("{} {} {} {}", n(&r[12]), n(&r[13]), n(&r[14]), r[2]));
        }
    }
    // pet_skill_id, mob_skill_id, anim, name, aoe, radius, distance, anim time, prepare, targets, message, flag, param,
    // finish category, knockback, primary_sc..: keyed by the pet skill id the action carries.
    for r in rows(&read("pet_skills.sql")?, "pet_skills") {
        if r.len() >= 18 && (n(&r[15]) | n(&r[16]) | n(&r[17])) != 0 {
            mob.entry(n(&r[0])).or_insert_with(|| format!("{} {} {} {}", n(&r[15]), n(&r[16]), n(&r[17]), r[3]));
        }
    }
    for (id, rest) in mob {
        text += &format!("m {id} {rest}\n");
    }
    for r in rows(&read("blue_spell_list.sql")?, "blue_spell_list") {
        if r.len() >= 8 && (n(&r[5]) | n(&r[6]) | n(&r[7])) != 0 {
            text += &format!("b {} {} {} {}\n", n(&r[0]), n(&r[5]), n(&r[6]), n(&r[7]));
        }
    }
    let mut trusts: std::collections::BTreeMap<String, Vec<i64>> = Default::default();
    for r in rows(&read("mob_skill_lists.sql")?, "mob_skill_lists") {
        if r.len() >= 3 && r[0].starts_with("TRUST_") && n(&r[2]) != 0 {
            trusts.entry(fold(&r[0])).or_default().push(n(&r[2]));
        }
    }
    for (name, ids) in trusts {
        text += &format!("t {name} {}\n", ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(" "));
    }
    std::fs::write(out, text).with_context(|| format!("writing {}", out.display()))?;
    println!("wrote {}", out.display());
    Ok(())
}
