-- XIFO Squad's reading of the server's "!squad panel" and "!squad sheet" lines (as XIFO Link hands them on, the tag
-- cut off); lines it does not know are skipped.

local P = require("src.parse")

local p = P.panel({
  "1|Fayd|WHM|30|BLM|15|healer|balanced|1|15",
  "jl|1|BLM:15,WHM:30",
  "2|Salia|WAR|28|-|0|tank|aggressive|0|5",
  "camp|1|2|1|7",
  "campbots|Aldo,Bea|Bea",
  "engage|1",
  "opt|1|1|1000|-|0|Hexa_Strike,True_Strike",
  "cd|2|42",
  "something|new",
})
assert(#p.mates == 2)
local f, s = p.mates[1], p.mates[2]
assert(f.name == "Fayd" and f.job == "WHM" and f.level == 30 and f.sub == "BLM" and f.sub_level == 15)
assert(f.role == "healer" and f.mode == "balanced" and f.out and f.switches == 15)
assert(f.levels.WHM == 30 and f.levels.BLM == 15)
assert(s.sub == nil and not s.out and s.cooldown == 42 and s.role == "tank")
assert(P.has_switch(s.switches, 1) and not P.has_switch(s.switches, 2) and P.has_switch(s.switches, 3))
assert(not P.has_switch(s.switches, 4))
assert(p.camp.on and p.camp.puller == 2 and p.camp.paused and p.camp.pulls == 7)
assert(#p.bots == 2 and p.bot_puller == "Bea")

local e = P.panel({ "camp|0|0|0|0", "campbots|-|-" })
assert(#e.mates == 0 and not e.camp.on and #e.bots == 0 and e.bot_puller == nil)

local sh = P.sheet({
  "hdr|Fayd|WHM|30|BLM|15|healer|balanced|Tarutaru F|15",
  "job|WHM|30",
  "job|BLM|15",
  "exp|WHM|30|1200|5400",
  "vital|HP|250|310",
  "vital|MP|120|200",
  "stat|STR|20",
  "combat|Attack|88",
  "haste|gear|500",
  "eq|4|head|Silver Hairpin|12345",
  "eq|0|main|Maple Wand|17049",
  "ls|5|body|Robe|12600",
  "note|Not called right now, so there are no live stats.",
  "spell|Cure II|24",
  "ability|Divine Seal|15",
})
assert(sh.name == "Fayd" and sh.race == "Tarutaru F" and sh.sub == "BLM" and sh.switches == 15)
assert(#sh.jobs == 2 and sh.jobs[2].job == "BLM" and sh.exp.need == 5400)
assert(sh.vitals.HP.now == 250 and sh.vitals.MP.max == 200 and sh.vitals.TP == nil)
assert(sh.stats[1].name == "STR" and sh.combat[1].value == 88 and sh.haste[1].value == 500)
assert(sh.gear[1].slot_name == "main" and sh.gear[2].id == 12345, "gear sorted by slot")
assert(#sh.look == 1 and sh.look[1].item == "Robe")
assert(sh.notes[1]:find("Not called", 1, true))
assert(sh.spells[1].name == "Cure II" and sh.spells[1].mp == 24 and sh.abilities[1].level == 15)

-- Servers from v0.3.11: the Rules line per mate, the capability words, the sheet's slot line.
local r = P.panel({
  "1|Fayd|WHM|30|BLM|15|healer|balanced|1|15",
  "rs|1|pending|edited|4|Heals|4|Heals|0|1|WHM",
  "2|Salia|WAR|28|-|0|tank|aggressive|0|5",
  "rs|2|off|-|0|-|0|-|0|0|-",
  "3|Bo|BLM|20|-|0|auto|balanced|1|15",
  "rs|3|fallback|-|0|-|6|?|1|0|BLM",
  "cap|reapply1,later",
})
assert(r.caps.reapply1 and r.caps.later)
local f1 = r.mates[1].rules
assert(f1.state == "pending" and f1.why == "edited" and f1.live == 4 and f1.live_name == "Heals" and f1.dropped == 1)
assert(f1.live_job == "WHM" and r.mates[2].rules.live_job == nil)
assert(P.rules_text(f1) == "Rules: set 4 Heals now, set 4 Heals from its next call (its set was edited; Apply now) (1 rule left out)",
  P.rules_text(f1))
assert(P.rules_text(r.mates[2].rules) == "Rules: its job rules at its next call", P.rules_text(r.mates[2].rules))
assert(P.rules_text(r.mates[3].rules) == "Rules: its job rules (the chosen set is gone)", P.rules_text(r.mates[3].rules))
assert(P.rules_text(nil) == nil and next(p.caps) == nil and p.mates[1].rules == nil)
assert(P.sheet({ "slot|3", "hdr|Bo|BLM|20|-|0|auto|balanced|Taru|15" }).slot == 3)

-- an rs line before its mate's row still counts; a chosen set that can no longer be used is not promised
local o = P.panel({ "rs|1|pending|choice|4|Heals|8|?|0|0|WHM", "1|Fayd|WHM|30|BLM|15|healer|balanced|1|15" })
assert(o.mates[1].rules and P.rules_text(o.mates[1].rules) ==
  "Rules: set 4 Heals now, its job rules from its next call (another set was chosen; Apply now)", P.rules_text(o.mates[1].rules))
