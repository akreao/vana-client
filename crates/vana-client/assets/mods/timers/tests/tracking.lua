-- The Timers mod's tracking: LSB's durations (the examples the client's engine was tested with), the charge
-- rule, the luopan, a Slow landing on a monster, and our effects shortest first.

local D = require("durations")
local function row(kind, id) return D.rows(kind, id)[1] end
local function rdm99() return D.Caster.new({ main = { 5, 99 } }) end
-- The tables load.
assert(D.rule(2).overwrite == "higher" and row("spell", 57).formula == "ENH" and row("spell", 57).effect == 33)
-- Slow: RDM 99, 5/5 merits, 20 job points, ENF_MAG_DURATION 20, Saboteur on the first target: 492.
local c = rdm99()
c.merits[2316], c.jps[167], c.mods[1151], c.statuses[454] = 5, 20, 20, true
assert(D.predict(c, row("spell", 56), { first = true, has = {} }, false) == 492)
assert(D.predict(c, row("spell", 56), { has = {} }, false) == 276)
-- Haste: 180; Composure x3 on ourselves only; Perpetuance x2; a level-48 member gets 48/96 of Haste II.
c = rdm99()
local me, other = { me = true, has = {} }, { has = {} }
assert(D.predict(c, row("spell", 57), other, true) == 180)
c.statuses[419] = true
assert(D.predict(c, row("spell", 57), me, true) == 540 and D.predict(c, row("spell", 57), other, true) == 180)
c.statuses[469] = true
assert(D.predict(c, row("spell", 57), other, true) == 360)
assert(D.predict(c, row("spell", 511), { level = 48, has = {} }, true) == 180)
-- Songs: song+ 3: 156; Troubadour 312.
c = D.Caster.new({ main = { 10, 75 }, mods = { [434] = 2, [452] = 1 } })
assert(D.predict(c, row("spell", 398), other, false) == 156)
c.statuses[348] = true
assert(D.predict(c, row("spell", 398), other, false) == 312)
-- Steps: 60 new, then the time left + 30 up to 120.
c = D.Caster.new({ main = { 19, 75 } })
assert(D.predict(c, row("ability", 201), other, false) == 60)
assert(D.predict(c, row("ability", 201), other, false, 45.5) == 75)
assert(D.predict(c, row("ability", 201), other, false, 100) == 120)
-- Rolls: 300 + 20 a Winning Streak merit.
c = D.Caster.new({ main = { 17, 75 }, merits = { [3076] = 2 } })
assert(D.predict(c, row("ability", 105), other, false) == 340)
assert(D.lands(row("ability", 105), 421, 11) and not D.lands(row("ability", 105), 421, 0))
-- Landing, icons and replacement.
assert(D.lands(row("spell", 23), 2, 37) and D.lands(row("spell", 23), 85, 0))
assert(D.lands(row("spell", 57), 230, 33) and not D.lands(row("spell", 57), 230, 0))
assert(not D.lands(row("spell", 57), 75, 0))
assert(D.icon_matches(66, 445) and D.icon_matches(19, 2) and not D.icon_matches(33, 2))
local function has(list, x)
  for _, v in ipairs(list) do if v == x then return true end end
  return false
end
assert(has(D.ends(128), 129) and has(D.ends(134), 135))
-- Weapon skills by TP.
c = D.Caster.new({ main = { 1, 75 }, tp = 2000 })
assert(D.predict(c, D.rows("ws", 83)[1], other, false) == 240)
c.tp = 3000
assert(D.predict(c, D.rows("ws", 87)[1], other, false) == 150)
c.tp = 1500
assert(D.predict(c, D.rows("ws", 129)[1], other, false) == 45)
-- Wards: the Summoning skill over 300 (at most 200); Perfect Defense 30 + skill / 20.
c = D.Caster.new({ main = { 15, 75 }, skills = { [38] = 420 } })
assert(D.predict(c, D.rows("pet", 595)[1], other, false) == 300)
c.skills[38] = 600
assert(D.predict(c, D.rows("pet", 595)[1], other, false) == 380)
assert(D.predict(c, D.rows("pet", 671)[1], other, false) == 60)
-- Dancer: Chocobo Jig II, a samba.
c = D.Caster.new({ main = { 19, 99 }, jps = { [613] = 10 }, mods = { [492] = 60 } })
assert(D.predict(c, row("ability", 381), other, false) == 195)
c.mods[490], c.mods[498] = 10, 10
assert(D.predict(c, row("ability", 189), me, false) == 143)
-- Helix; Indi on an Entrust target.
c = D.Caster.new({ main = { 20, 75 }, mods = { [477] = 72 } })
assert(D.predict(c, row("spell", 278), other, false) == 162)
c.mods[960] = 21
assert(D.predict(c, row("spell", 768), other, false) == 201)
-- Merits count only for the main job at 75.
c = D.Caster.new({ main = { 1, 75 }, merits = { [3076] = 5 } })
assert(c:merit(3076) == 0)
c.main = { 17, 74 }
assert(c:merit(3076) == 0)
c.main = { 17, 75 }
assert(c:merit(3076) == 100)

-- The game as the engine and rows see it: RDM 99 with four buffs, a Valkurm Emperor in front.
local buffs = { { id = 41, remaining = 600 }, { id = 40, remaining = 300 }, { id = 33, remaining = 120 },
  { id = 42, remaining = 200 } }
local names = { [33] = "Haste", [40] = "Protect", [41] = "Shell", [42] = "Regen", [13] = "Slow" }
local p = { id = 1, name = "Lnxcsfix", main_job = 5, main_level = 99, sub_job = 13, sub_level = 49, zone = 103,
  tp = 0 }
game = {
  player = function() return p end, buffs = function() return buffs end, party = function() return {} end,
  entity = function(id) if id == 0x1000 then return { id = id, name = "Valkurm Emperor", kind = "mob", status = 1 } end end,
  merits = function() return {} end, job_points = function() return {} end,
  skills = function() return { combat = {}, magic = {} } end, equipment = function() return {} end,
  items = function() return {} end, recasts = function() return { spells = {}, abilities = {}, calc = {} } end,
  abilities = function() return {} end,
}
res = { status = function(id) return names[id] end, status_info = function() return { untimed = false } end,
  spell = function(id) return id == 56 and { name = "Slow", kind = 1 } or nil end, ability = function() end,
  item = function() end }
mods = { list = function() return {} end }
local E = require("engine")
local R = require("rows")
-- The luopan at 99: 207 s; with Blaze of Glory half.
c = D.Caster.new({ main = { 21, 99 } })
assert(E.luopan_life(c) == 207)
c.statuses[569] = true
assert(E.luopan_life(c) == 105)
-- Charges: Stratagems at SCH 99 with 100 s left: 4 s to the next, 2 ready; Ready with calc2 -15: 25-s shares.
local n, ready, per = R.charges(100, 5, 240, 0, 0)
assert(n == 4 and ready == 2 and per == 48, n .. " " .. ready .. " " .. per)
n, ready, per = R.charges(96, 5, 240, 0, 0)
assert(n == 48 and ready == 3)
n, ready, per = R.charges(60, 3, 90, 0, -15)
assert(n == 10 and ready == 0 and per == 25)
assert(R.charges(10, 0, 90, 0, 0) == nil)
-- Our Slow lands on the Emperor: its row under its name, 180 s (no merits, no Saboteur).
E.on_action({ actor = 1, category = 4, param = 56, targets = { { id = 0x1000, results = { { message = 236, value = 13, info = 0 } } } } })
local secs = R.sections(true)
local mob = secs[3][1]
assert(mob.name == "Valkurm Emperor" and mob.rows[1].name == "Slow", tostring(mob and mob.name))
assert(math.abs(mob.rows[1].left - 180) < 1 and mob.rows[1].total == 180)
-- Our effects, shortest first.
local own = secs[2][1]
local order = {}
for _, r in ipairs(own.rows) do order[#order + 1] = r.name end
assert(own.name == "Lnxcsfix" and table.concat(order, ",") == "Haste,Regen,Protect,Shell", table.concat(order, ","))
-- It wears off: the server's message ends it.
E.on_message(0x1000, 13, 204)
assert(#R.sections(true)[3] == 0)
-- Another mod's row (timers.add through the service), in the buffs section under "Timers".
mods.list = function() return { { id = "countdowns" } } end
R.add("countdowns", "Pop", 300, { group = "Spawns" })
secs = R.sections(false)
assert(#secs[1] == 0 and secs[2][1].name == "Spawns" and secs[2][1].rows[1].name == "Pop")
local ok, err = pcall(R.add, "countdowns", "Bad", 10, { icon = { kind = "status" } })
assert(not ok and tostring(err):find("numeric id"), tostring(err))
-- A number name is the same row on add and remove.
R.clear("countdowns")
R.add("countdowns", 5, 30)
R.add("countdowns", 5, 30)
assert(#R.sections(false)[2][1].rows == 1)
R.remove("countdowns", 5)
assert(#R.sections(false)[2] == 0)
-- Copies of one status (maneuvers) each keep their gauge.
buffs = { { id = 300, remaining = 20 }, { id = 300, remaining = 60 } }
R.sections(true)
local man = R.sections(true)[2][1].rows
assert(man[1].total == 20 and man[2].total == 60, man[1].total .. " " .. man[2].total)
