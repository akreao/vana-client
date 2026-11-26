//! The mission and quest logs' name files (MODLOG "Missions"): d_msg entries {u32 number, name,
//! description}, one file per mission log (15) and per quest area (11), in the file language.

use super::DatIndex;
use super::items::Field;
use super::names::DMsg;
use anyhow::Result;

/// Mission logs 0..14 (DAT_10667a90 + log * 0x94): San d'Oria, Bastok, Windurst, Zilart, CoP, Assault, ToAU, Campaign,
/// WotG, SoA, RoV, TVR, ACP, AMK, ASA. Language slots 0x49..0x4f, 0x50, 0x51, 0x55..0x57, 0x52..0x54; ja / en / de /
/// fr as the slot tables hold them (30261102_0).
pub const MISSION_FILES: [[u32; 4]; 15] = [
    [55595, 55715, 55835, 56255],
    [55596, 55716, 55836, 56256],
    [55597, 55717, 55837, 56257],
    [55598, 55718, 55838, 56258],
    [55599, 55719, 55839, 56259],
    [55600, 55720, 55840, 56260],
    [55601, 55721, 55841, 56261],
    [55604, 55724, 55844, 56264],
    [55603, 55723, 55843, 56263],
    [55618, 55738, 55858, 56278],
    [55621, 55741, 55861, 56281],
    [55622, 55742, 55862, 56282],
    [55615, 55735, 55855, 56275],
    [55616, 55736, 55856, 56276],
    [55617, 55737, 55857, 56277],
];
/// Quest areas 0..10: San d'Oria, Bastok, Windurst, Jeuno, Other Areas, Outlands, Aht Urhgan, Crystal War, Abyssea,
/// Adoulin, Coalition. Language slots 0x40..0x46, 0x47, 0x48, 0x58, 0x59.
pub const QUEST_FILES: [[u32; 4]; 11] = [
    [55586, 55706, 55826, 56246],
    [55587, 55707, 55827, 56247],
    [55588, 55708, 55828, 56248],
    [55589, 55709, 55829, 56249],
    [55590, 55710, 55830, 56250],
    [55591, 55711, 55831, 56251],
    [55592, 55712, 55832, 56252],
    [55602, 55722, 55842, 56262],
    [55593, 55713, 55833, 56253],
    [55619, 55739, 55859, 56279],
    [55620, 55740, 55860, 56280],
];

#[derive(Clone, Debug, Default)]
pub struct Entry {
    pub number: u32,
    pub name: String,
    pub desc: String,
}

/// One log file's entries in file order.
#[derive(Clone, Debug, Default)]
pub struct LogFile {
    pub entries: Vec<Entry>,
}

impl LogFile {
    pub fn load(index: &DatIndex, file: u32) -> Result<LogFile> {
        let m = DMsg::load(index, file)?;
        let entries = (0..m.len())
            .map(|i| {
                let f = m.fields(i).unwrap_or_default();
                let num = f.iter().find_map(|x| if let Field::Num(n) = x { Some(*n) } else { None }).unwrap_or(0);
                let mut texts = f.iter().filter_map(|x| if let Field::Text(s) = x { Some(s.clone()) } else { None });
                Entry { number: num, name: texts.next().unwrap_or_default(), desc: texts.next().unwrap_or_default() }
            })
            .collect();
        Ok(LogFile { entries })
    }
}

/// Every mission log and quest area, loaded together on the first open.
#[derive(Clone, Debug, Default)]
pub struct Logs {
    pub missions: Vec<LogFile>,
    pub quests: Vec<LogFile>,
}

impl Logs {
    pub fn load(index: &DatIndex) -> Logs {
        let load = |files: &[[u32; 4]]| {
            files
                .iter()
                .map(super::lang::file)
                .map(|f| LogFile::load(index, f).map_err(|e| log::warn!("journal file {f}: {e:#}")).unwrap_or_default())
                .collect()
        };
        Logs { missions: load(&MISSION_FILES), quests: load(&QUEST_FILES) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files() {
        let Ok(index) = DatIndex::open(&crate::config::Config::load().ffxi_dir()) else { return };
        let l = Logs::load(&index);
        assert_eq!(l.missions[0].entries.len(), 24);
        assert_eq!(
            (l.missions[0].entries[0].number, l.missions[0].entries[0].name.as_str()),
            (0, "Smash the Orcish Scouts")
        );
        assert_eq!(l.missions[4].entries.len(), 65);
        assert!(l.quests.iter().all(|q| !q.entries.is_empty()));
    }
}
