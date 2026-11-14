-- Suggestions' rules replay every recorded case (tests/cases: the game's facts and the rows the rules made of
-- them), with the chain rules as the Skillchain mod provides them; each rule has a case; held rows regroup.

local RULES = { "raise", "shadows", "ailment", "curaga", "cure", "waltz", "burst", "close", "provoke", "job_ability",
  "ninja_stance", "samba", "rune_fencer", "geomancer", "debuff", "open_chain", "strongest_ws", "renew", "between" }

-- Skillchain's `rules` service (LSB's battleutils): map {closer's property, resonance, chain}, names, elements.
local CHAIN = {
  map = {
    { 13, 13, 15 }, { 14, 14, 16 }, { 10, 9, 14 }, { 12, 9, 12 }, { 9, 10, 14 }, { 11, 10, 11 }, { 9, 11, 9 },
    { 12, 11, 13 }, { 10, 12, 10 }, { 11, 12, 13 }, { 4, 1, 10 }, { 8, 3, 11 }, { 2, 6, 9 }, { 5, 7, 12 },
    { 2, 1, 2 }, { 5, 1, 5 }, { 1, 2, 1 }, { 6, 2, 6 }, { 4, 3, 4 }, { 3, 4, 3 }, { 5, 4, 5 }, { 6, 4, 6 },
    { 7, 5, 7 }, { 8, 5, 8 }, { 4, 6, 4 }, { 2, 7, 2 }, { 8, 7, 8 }, { 3, 8, 3 }, { 6, 8, 6 },
  },
  names = { "Transfixion", "Compression", "Liquefaction", "Scission", "Reverberation", "Detonation", "Induration",
    "Impaction", "Gravitation", "Distortion", "Fusion", "Fragmentation", "Light", "Darkness", "Light", "Darkness" },
  elements = { { 6 }, { 7 }, { 0 }, { 3 }, { 5 }, { 2 }, { 1 }, { 4 }, { 7, 3 }, { 5, 1 }, { 6, 0 }, { 2, 4 },
    { 6, 0, 2, 4 }, { 7, 3, 5, 1 }, { 6, 0, 2, 4 }, { 7, 3, 5, 1 } },
}

local function case(name) return json.decode(TEST.source("tests/cases/" .. name)) end
local files = {}
for _, n in ipairs(TEST.list("tests/cases")) do
  if n:match("%.json$") then files[#files + 1] = n end
end
assert(#files > 0, "no cases")

-- The chain rules through init.lua, from a fake Skillchain mod: one tick of the battle menu sets them.
local R = require("rules")
local F = require("facts")
local tick
settings = { get = function() return true end }
events = { on = function() end, off = function() end }
timer = { every = function(_, f) tick = f end }
ui = { native = { battle_menu = function() return {} end, slot = function() end } }
mods = { emit = function() end,
  call = function(id, name) assert(id == "skillchain" and name == "rules", id .. "." .. name) return CHAIN end }
load(TEST.source("init.lua"), "@init.lua")()
local gather = F.gather
F.gather = function() return case(files[1]).facts end
tick()
F.gather = gather
assert(R.chain, "no chain rules")

local seen, failed = {}, {}
for _, name in ipairs(files) do
  local c = case(name)
  local known = false
  for _, r in ipairs(RULES) do known = known or r == c.rule end
  assert(known, name .. ": rule " .. tostring(c.rule))
  seen[c.rule] = true
  local got = R.rows(R.suggest(c.facts))
  local want = {}
  for _, e in ipairs(c.expect) do
    local kind, id = "spell", e.action.Spell
    if not id then kind, id = "ability", e.action.Ability end
    want[#want + 1] = { kind = kind, id = id, target = e.target, reason = e.reason, label = e.label,
      group = e.group or {} }
  end
  local a, b = json.encode(got), json.encode(want)
  if a ~= b then failed[#failed + 1] = name .. " (" .. tostring(c.note) .. "): expected " .. b .. "\n got " .. a end
end
assert(#failed == 0, table.concat(failed, "\n\n"))
local missing = {}
for _, r in ipairs(RULES) do
  if not seen[r] then missing[#missing + 1] = r end
end
assert(#missing == 0, "rules without a case: " .. table.concat(missing, ", "))

-- Held rows: a tier back from its recast returns to its family, the nuke row back to the list's top; down to one
-- castable spell the row casts it, and the family comes back.
do
  local function named(id, name, mp, area)
    return { id = id, name = name, skill = 36, element = 1, mp = mp, targets = 0x20, area = area }
  end
  local all = { named(150, "Blizzard II", 31, false), named(149, "Blizzard", 8, false),
    named(180, "Blizzaga II", 175, true), named(179, "Blizzaga", 80, true) }
  local target = { name = "Bark Spider", hpp = 90, ranks = { 0, -2, 0, 0, 0, 0, 0, 0 } }
  local f = { party = { { name = "Lenon", me = true, here = true, hp = 1000, hp_max = 1000 } },
    spells = { all[2], all[4] }, target = target }
  local s = R.suggest(f)
  local k
  for i, x in ipairs(s) do if x.label == "Ice" then k = i end end
  assert(k and s[k].id == 149 and table.concat(s[k].group, ",") == "149,179")
  R.regroup({ party = f.party, spells = all, target = target }, s)
  assert(s[k].id == 150 and table.concat(s[k].group, ",") == "150,149,180,179")
  R.regroup({ party = f.party, spells = { all[2] }, target = target }, s)
  assert(s[k].id == 149 and #s[k].group == 0 and s[k].label == nil)
  R.regroup({ party = f.party, spells = all, target = target }, s)
  assert(s[k].id == 150 and s[k].label == "Ice")
end
-- Protect's family takes Protectra.
local p = function(id, name, mp, area) return { id = id, name = name, skill = 34, mp = mp, targets = 1, area = area } end
assert(R.family(p(44, "Protect II", 28, false)) == "Protect" and R.family(p(125, "Protectra", 9, true)) == "Protect")
-- LSB's resistance ranks moved with the mod (data/resistances.txt): a monster's over its species'.
assert(table.concat(F.ranks_of(100, " Wild Rabbit "), ",") == "-2,-1,-2,-1,-3,-3,-1,-3")
assert(F.ranks_of(100, "No Such Monster") == nil)
