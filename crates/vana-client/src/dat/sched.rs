//! Type-0x07 schedulers (FUN_10057000) as far as combat motions need them. A scheduler is a header
//! (op stream at data + u32 @0x14 - 0x10, length in ticks at u32 @0x1c) and ops {u8 op, u8 length in u32s (low 5
//! bits), u16, u16 wait @4 (ticks before the next op runs), ...}; op 0 ends it.

/// One op: code, wait before the next op, and its bytes.
#[derive(Clone, Debug)]
pub struct Op<'a> {
    pub code: u8,
    pub wait: u16,
    pub raw: &'a [u8],
}

impl Op<'_> {
    /// The 4cc at +8 (motion / scheduler / sound name).
    pub fn name(&self) -> Option<&str> {
        let b = self.raw.get(8..12)?;
        std::str::from_utf8(b).ok().map(|s| s.trim_end_matches('\0'))
    }
}

pub fn ops(data: &[u8]) -> Vec<Op<'_>> {
    let mut out = Vec::new();
    // FUN_10055aa0: the offsets count from the chunk start (16 bytes before the data).
    let Some(mut at) =
        data.get(0x14..0x18).map(|b| (u32::from_le_bytes(b.try_into().unwrap()) as usize).saturating_sub(0x10))
    else {
        return out;
    };
    while at + 8 <= data.len() {
        let (code, len) = (data[at], (data[at + 1] & 0x1f) as usize * 4);
        if code == 0 || len == 0 || at + len > data.len() {
            break;
        }
        out.push(Op { code, wait: u16::from_le_bytes([data[at + 4], data[at + 5]]), raw: &data[at..at + len] });
        at += len;
    }
    out
}

/// FUN_10056e10: the end stream (header u32 @0x18, chunk-relative) until op 0; an op 0x01 there means the task starts
/// over at its end (FUN_10057c20).
pub fn loops(data: &[u8]) -> bool {
    // An offset inside the chunk header (below 0x10) points at no stream: no loop.
    let Some(mut at) =
        data.get(0x18..0x1c).and_then(|b| (u32::from_le_bytes(b.try_into().unwrap()) as usize).checked_sub(0x10))
    else {
        return false;
    };
    while at + 4 <= data.len() {
        let (code, len) = (data[at], (data[at + 1] & 0x1f) as usize * 4);
        if code == 1 {
            return true;
        }
        if code == 0 || len == 0 {
            return false;
        }
        at += len;
    }
    false
}

/// FUN_10056e80: started by itself when its file is parsed (FUN_10056a50) or its weather folder is set
/// (FUN_1018a7a0): an end loop or an op 0x52 in the main stream, and no op 0xa2.
pub fn auto_start(data: &[u8]) -> bool {
    let main = ops(data);
    (loops(data) || main.iter().any(|o| o.code == 0x52)) && !main.iter().any(|o| o.code == 0xa2)
}

/// FUN_100c46b0: started by a furniture actor on itself when its file comes in: marked as self-starting (FUN_10056e50:
/// an end loop or an op 0x52) and holding the op 0xa2 that keeps the parse from starting it.
pub fn furniture_start(data: &[u8]) -> bool {
    let main = ops(data);
    (loops(data) || main.iter().any(|o| o.code == 0x52)) && main.iter().any(|o| o.code == 0xa2)
}

/// FUN_10056ae0: max(start + dur) over the main stream, op 0 included (it overwrites the header's length).
#[cfg(test)]
pub fn span(data: &[u8]) -> i32 {
    let Some(mut at) =
        data.get(0x14..0x18).map(|b| (u32::from_le_bytes(b.try_into().unwrap()) as usize).saturating_sub(0x10))
    else {
        return 0;
    };
    let (mut t, mut best) = (0i32, 0i32);
    while at + 8 <= data.len() {
        let (code, len) = (data[at], (data[at + 1] & 0x1f) as usize * 4);
        let wait = i16::from_le_bytes([data[at + 4], data[at + 5]]) as i32;
        let dur = i16::from_le_bytes([data[at + 6], data[at + 7]]) as i32;
        best = best.max(t + dur);
        if code == 0 || len == 0 {
            break;
        }
        t += wait;
        at += len;
    }
    best
}

/// Header u32 @0x1c: the scheduler's length in ticks.
pub fn length(data: &[u8]) -> u32 {
    data.get(0x1c..0x20).map_or(0, |b| u32::from_le_bytes(b.try_into().unwrap()))
}

/// A motion a scheduler plays (op 0x05 -> FUN_100cb8e0): name with the region '?' stripped ("at0?" ->
/// "at0"), the tick it starts, f32 speed @0x10, the play length u16 @6, the play count byte @0x1e & 0x7f
/// (0 = loop until replaced; motion player +0x2c) and the blend-in ticks u16 @0x18.
#[derive(Clone, Debug, PartialEq)]
pub struct Motion {
    pub name: String,
    pub at: u32,
    pub speed: f32,
    pub ticks: u16,
    pub count: u8,
    pub blend: u16,
}

/// What a scheduler does that combat shows: its motions and when sub-scheduler `hit` (op 0x03) runs.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Plan {
    pub motions: Vec<Motion>,
    pub hit_at: Option<u32>,
    pub length: u32,
    /// Op 0x28 (FUN_100c6b70 + FUN_100cb8e0): from here the status motion is chosen again, a 10-tick blend
    /// *(inf: back to the stance)*.
    pub stance_at: Option<u32>,
    /// When the holds end, from the start: op 0x59 on the attacker (FUN_10062650, about 30 ticks past the hit: no
    /// reactions, server status changes wait) and op 0x20 on the target (FUN_100601f0, hit + 4); u16 @6 each.
    pub attacker_hold: Option<u32>,
    pub target_hold: Option<u32>,
}

pub fn plan(data: &[u8], hit: &[&str]) -> Plan {
    let mut p = Plan { length: length(data), ..Default::default() };
    let mut t = 0u32;
    for op in ops(data) {
        match op.code {
            0x05 => {
                if let Some(n) = op.name() {
                    let speed = op.raw.get(16..20).map_or(1.0, |b| f32::from_le_bytes(b.try_into().unwrap()));
                    let u16_at = |o: usize| op.raw.get(o..o + 2).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]));
                    let count = op.raw.get(0x1e).map_or(1, |b| b & 0x7f);
                    p.motions.push(Motion {
                        name: n.trim_end_matches('?').to_string(),
                        at: t,
                        speed,
                        ticks: u16_at(6),
                        count,
                        blend: u16_at(0x18),
                    });
                }
            }
            0x03 if op.name().is_some_and(|n| hit.contains(&n)) && p.hit_at.is_none() => p.hit_at = Some(t),
            0x28 => p.stance_at = Some(t),
            0x59 | 0x20 => {
                let end = t + op.raw.get(6..8).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]) as u32);
                let slot = if op.code == 0x59 { &mut p.attacker_hold } else { &mut p.target_hold };
                *slot = Some(slot.map_or(end, |e| e.max(end)));
            }
            _ => {}
        }
        t += op.wait as u32;
    }
    p
}

/// Op 0x84's calls (dual wield draw / sheathe): the tick, sheathe (u16 @8), the sub hand (u16 @10).
pub fn hand_calls(data: &[u8]) -> Vec<(u32, bool, bool)> {
    let mut t = 0u32;
    let mut out = Vec::new();
    for op in ops(data) {
        if op.code == 0x84 {
            let u16_at = |o: usize| op.raw.get(o..o + 2).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]));
            out.push((t, u16_at(8) != 0, u16_at(10) != 0));
        }
        t += op.wait as u32;
    }
    out
}

/// Scheduler names matching a 4cc pattern ('?' = any byte), as FUN_100cc740 counts them.
pub fn matching<'a>(names: impl Iterator<Item = &'a str>, pattern: &str) -> Vec<&'a str> {
    let p = pattern.as_bytes();
    names.filter(|n| n.len() == 4 && n.bytes().zip(p).all(|(a, &b)| b == b'?' || a == b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Model 0 (file 1300) "ati0": motion at0? for 80 ticks; "dada" (the hit) runs after 24 + 12 ticks.
    #[test]
    fn mob_attack_plan() {
        let cfg = crate::config::Config::load();
        let Ok(index) = crate::dat::DatIndex::open(&cfg.ffxi_dir()) else { return };
        let file = index.read(1300).unwrap();
        let chunks = crate::dat::chunks(&file).unwrap();
        let get = |n: &str| chunks.iter().find(|c| c.kind == 7 && c.name() == n).unwrap().data;
        let p = plan(get("ati0"), &["dada"]);
        assert_eq!(p.motions, vec![Motion { name: "at0".into(), at: 0, speed: 1.0, ticks: 80, count: 1, blend: 10 }]);
        assert_eq!((p.hit_at, p.length), (Some(36), 80));
        assert_eq!(p.target_hold, Some(40));
        let d = plan(get("dead"), &[]);
        assert_eq!(d.motions.iter().map(|m| (m.name.as_str(), m.at)).collect::<Vec<_>>(), [("ded", 0), ("cor0", 111)]);
        let names: Vec<String> = chunks.iter().filter(|c| c.kind == 7).map(|c| c.name()).collect();
        let mut m = matching(names.iter().map(|s| s.as_str()), "ati?");
        m.sort();
        assert_eq!(m, ["ati0", "ati1", "ati2"]);
    }
}

#[cfg(test)]
mod census {
    use super::*;
    use crate::dat::generator::{Tree, kind};

    /// Every self-starting scheduler's first step (the ops FUN_10057e10 runs when FUN_10056b80 steps the new task
    /// inside the load: each op up to and including the first with a wait, or an op 0x52), split by whether it sits
    /// under a `weat` folder (by hand: `cargo test --release auto_first_step_census -- --ignored --nocapture`).
    #[test]
    #[ignore]
    fn auto_first_step_census() {
        let Ok(index) = crate::dat::DatIndex::open(&crate::config::Config::load().ffxi_dir()) else { return };
        let trim = |n: [u8; 4]| String::from_utf8_lossy(&n).trim_end_matches(['\0', ' ']).to_string();
        let mut seqs: std::collections::BTreeMap<(bool, String), (usize, Vec<String>)> = Default::default();
        let mut tags: std::collections::BTreeMap<String, usize> = Default::default();
        for id in 0..index.len() as u32 {
            let Ok(file) = index.read(id) else { continue };
            let Ok(tree) = Tree::build(&file) else { continue };
            for n in tree.of_kind(kind::SCHEDULER).collect::<Vec<_>>() {
                let data = &file[tree.nodes[n].data.clone()];
                if !auto_start(data) {
                    continue;
                }
                let path: Vec<String> = tree.path(n).into_iter().map(trim).collect();
                let weat = path.iter().position(|p| p == "weat");
                if let Some(w) = weat {
                    *tags.entry(path.get(w + 1).cloned().unwrap_or_default()).or_default() += 1;
                }
                let mut seq = Vec::new();
                for o in ops(data) {
                    let name = o.name().unwrap_or("").to_string();
                    seq.push(match o.code {
                        0x01 | 0x52 => format!("{:02x}", o.code),
                        _ => format!("{:02x}:{}", o.code, name.len().min(4).min(usize::from(name.is_ascii()) * 4)),
                    });
                    if o.code == 0x52 || o.wait != 0 {
                        break;
                    }
                }
                let e = seqs.entry((weat.is_some(), seq.join(" "))).or_default();
                e.0 += 1;
                if e.1.len() < 6 {
                    e.1.push(format!("{id} {}/{}", path.join("/"), trim(tree.nodes[n].name)));
                }
            }
        }
        println!("weat tags: {tags:?}");
        for ((weat, seq), (count, eg)) in &seqs {
            println!("{} [{seq}] x{count}: {}", if *weat { "weat" } else { "zone" }, eg.join(", "));
        }
    }
    /// The sub-area files the zones' RID records name ('m' / 'M', FUN_101779b0) and what sits under their `weat`
    /// folders: tag folders, looping (0x1000) generators, self-starting schedulers, sounds (by hand: `cargo test
    /// --release sub_area_weat_census -- --ignored --nocapture`).
    #[test]
    #[ignore]
    fn sub_area_weat_census() {
        use crate::dat::generator::Generator;
        let Ok(index) = crate::dat::DatIndex::open(&crate::config::Config::load().ffxi_dir()) else { return };
        let trim = |n: [u8; 4]| String::from_utf8_lossy(&n).trim_end_matches(['\0', ' ']).to_string();
        let mut subs: std::collections::BTreeMap<u32, Vec<u32>> = Default::default();
        for zone in (0..300).chain(1000..1300) {
            let Ok(file) = index.read(crate::dat::zone::file_id(zone)) else { continue };
            let Ok(list) = crate::dat::chunks(&file) else { continue };
            for c in list.iter().filter(|c| c.kind == 0x36) {
                for r in crate::dat::zone::parse_rid(c.data).iter().filter(|r| r.is_sub_area()) {
                    let e = subs.entry(r.param).or_default();
                    if !e.contains(&zone) {
                        e.push(zone);
                    }
                }
            }
        }
        println!("{} sub-area ids named", subs.len());
        let (mut with, mut without, mut missing) = (0, 0, 0);
        for (id, zones) in &subs {
            let Ok(file) = index.read(crate::dat::zone::file_id(*id)) else {
                missing += 1;
                continue;
            };
            let Ok(tree) = Tree::build(&file) else { continue };
            let weat: Vec<usize> = tree.of_kind(kind::FOLDER).filter(|&n| tree.nodes[n].name == *b"weat").collect();
            if weat.is_empty() {
                without += 1;
                continue;
            }
            with += 1;
            let under = |n: usize| tree.path(n).iter().any(|p| p == b"weat");
            let mut tags: Vec<String> = tree
                .of_kind(kind::FOLDER)
                .filter(|&n| tree.nodes[n].parent.is_some_and(|p| weat.contains(&p)))
                .map(|n| trim(tree.nodes[n].name))
                .collect();
            tags.sort();
            let gens: Vec<Generator> = tree
                .of_kind(kind::GENERATOR)
                .filter(|&n| under(n))
                .filter_map(|n| Generator::parse(&tree, n, &file).ok())
                .collect();
            let looping = gens.iter().filter(|g| g.flags & 0x1000 != 0).count();
            let scheds: Vec<usize> = tree.of_kind(kind::SCHEDULER).filter(|&n| under(n)).collect();
            let auto = scheds.iter().filter(|&&n| auto_start(&file[tree.nodes[n].data.clone()])).count();
            let sounds = tree.of_kind(0x3d).filter(|&n| under(n)).count();
            let top: Vec<String> = tree
                .of_kind(kind::FOLDER)
                .filter(|&n| tree.nodes[n].parent.is_some_and(|p| tree.nodes[p].parent.is_none()))
                .map(|n| trim(tree.nodes[n].name))
                .collect();
            println!(
                "sub {id} (zones {zones:?}) top {top:?} weat x{}: tags {tags:?}; {} generators ({looping} looping), {} schedulers ({auto} auto), {sounds} sounds",
                weat.len(),
                gens.len(),
                scheds.len()
            );
        }
        println!("{with} with a weat folder, {without} without, {missing} files missing");
    }
    /// The schedulers the furniture actor starts on itself (FUN_100c46b0: an end loop or an op 0x52, and an op 0xa2):
    /// file, path, ops with their waits and names (by hand: `cargo test --release furniture_auto_census -- --ignored
    /// --nocapture`).
    #[test]
    #[ignore]
    fn furniture_auto_census() {
        let Ok(index) = crate::dat::DatIndex::open(&crate::config::Config::load().ffxi_dir()) else { return };
        let trim = |n: [u8; 4]| String::from_utf8_lossy(&n).trim_end_matches(['\0', ' ']).to_string();
        let (mut n, mut files) = (0, std::collections::BTreeSet::new());
        for id in 0..index.len() as u32 {
            let Ok(file) = index.read(id) else { continue };
            let Ok(tree) = Tree::build(&file) else { continue };
            for node in tree.of_kind(kind::SCHEDULER).collect::<Vec<_>>() {
                let data = &file[tree.nodes[node].data.clone()];
                if !furniture_start(data) {
                    continue;
                }
                n += 1;
                files.insert(id);
                let path: Vec<String> = tree.path(node).into_iter().map(trim).collect();
                let seq: Vec<String> = ops(data)
                    .iter()
                    .map(|o| match o.code {
                        0x52 => {
                            let u = |k: usize| u32::from_le_bytes(o.raw[k..k + 4].try_into().unwrap());
                            let hm = |v: u32| format!("{:02}:{:02}", v / 216000, v % 216000 / 3600);
                            format!(
                                "52[{}-{} P{}m tol {}m w{}]",
                                hm(u(8)),
                                hm(u(0xc)),
                                u(0x10) / 3600,
                                u(0x14) / 3600,
                                o.wait
                            )
                        }
                        0x01 | 0xa2 => format!("{:02x}", o.code),
                        _ => format!("{:02x}:{} w{}", o.code, o.name().unwrap_or("?"), o.wait),
                    })
                    .collect();
                println!(
                    "{id} {}/{} loop {}: {}",
                    path.join("/"),
                    trim(tree.nodes[node].name),
                    loops(data),
                    seq.join(" ")
                );
            }
        }
        println!("{n} schedulers in {} files: {files:?}", files.len());
    }
    /// The zone-lighting scheduler ops across every DAT: 0x82 / 0x83 (the lighting override task, FUN_10057fb0
    /// 0x10059273 / 0x10059181), 0xc0 (fixed sky time, 0x10058079) and 0xc3 (sun / moon tilt, 0x1005809b), with the
    /// weather ops 0x7b..0x7d for scale, and a histogram of every op code (by hand: `cargo test --release
    /// light_op_census -- --ignored --nocapture`).
    #[test]
    #[ignore]
    fn light_op_census() {
        let Ok(index) = crate::dat::DatIndex::open(&crate::config::Config::load().ffxi_dir()) else { return };
        let trim = |n: [u8; 4]| String::from_utf8_lossy(&n).trim_end_matches(['\0', ' ']).to_string();
        let mut hist: std::collections::BTreeMap<u8, (usize, std::collections::BTreeSet<u32>)> = Default::default();
        for id in 0..index.len() as u32 {
            let Ok(file) = index.read(id) else { continue };
            let Ok(tree) = Tree::build(&file) else { continue };
            for node in tree.of_kind(kind::SCHEDULER).collect::<Vec<_>>() {
                let data = &file[tree.nodes[node].data.clone()];
                if &tree.nodes[node].name == b"@ini" {
                    let path: Vec<String> = tree.path(node).into_iter().map(trim).collect();
                    let seq: Vec<String> = ops(data)
                        .iter()
                        .map(|o| {
                            let hex: Vec<String> =
                                o.raw[8.min(o.raw.len())..].iter().map(|b| format!("{b:02x}")).collect();
                            format!("{:02x} w{} {}", o.code, o.wait, hex.join(""))
                        })
                        .collect();
                    println!("@ini {id} {} loop {}: {}", path.join("/"), loops(data), seq.join(" | "));
                }
                for o in ops(data) {
                    let e = hist.entry(o.code).or_default();
                    e.0 += 1;
                    e.1.insert(id);
                    if matches!(o.code, 0x82 | 0x83 | 0xc0 | 0xc3 | 0x7c | 0x7d) {
                        let path: Vec<String> = tree.path(node).into_iter().map(trim).collect();
                        let hex: Vec<String> = o.raw.iter().map(|b| format!("{b:02x}")).collect();
                        println!(
                            "{id} {}/{} op {:02x}: {}",
                            path.join("/"),
                            trim(tree.nodes[node].name),
                            o.code,
                            hex.join("")
                        );
                    }
                }
            }
        }
        for (code, (n, files)) in &hist {
            let eg: Vec<u32> = files.iter().take(8).copied().collect();
            println!("op {code:02x}: {n} in {} files, e.g. {eg:?}", files.len());
        }
    }
}
