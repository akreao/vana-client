-- (Gearshift's XML front ends and regex: ashitacast eval.)

local said, ran, sent = {}, {}, {}
local BUFFS = { { id = 5 }, { id = 33 } }
local PLAYER = { name = "Tester", id = 100, index = 1, main_job = 1, main_level = 75, sub_job = 13, sub_level = 37,
  hp = 300, hp_max = 1000, mp = 50, mp_max = 100, tp = 1000, hpp = 75, mpp = 50, status = 1, zone = 230, pos = { x = 0, y = 0, z = 0 } }
game = {
  player = function() return PLAYER end,
  buffs = function() return BUFFS end,
  party = function() return {} end,
  target = function() return nil end,
  entity = function() return nil end,
  entities = function() return {} end,
  entity_by_tag = function() return nil end,
  stats = function() return { attack = 300, defense = 250, resist = { 1, 2, 3, 4, 5, 6, 7, 8 } } end,
  time = function() return { earth = 0, weekday = 0, hour = 7, minute = 5, moon = 50, moon_phase = 0 } end,
  weather = function() return 4 end,
  equipment = function() return {} end,
  items = function() return {} end,
}
res = {
  job = function(id) return ({ [0] = nil, "WAR", "MNK", "WHM", [13] = "NIN" })[id] end,
  status = function(id) return ({ [5] = "Blindness", [33] = "Haste", [580] = "Haste" })[id] end,
  spell = function(id) if id == 1 then return { name = "Cure", mp = 8, cast_time = 2, element = 6, skill = 33, kind = 1 } end end,
  ability = function() return nil end,
  item = function() return nil end,
  zone = function() return "Somewhere" end,
  mob_skill = function() return nil end,
}
chat = { add = function(t) said[#said + 1] = t end }
clock = function() return 1000 end
timer = { after = function(_, f) f() end }
commands = { run = function(l) ran[#ran + 1] = l end }
packets = { inject_out = function(d) sent[#sent + 1] = d end, build = function(id, b) return string.pack("<I2", id) .. b end }
fs = { list = function() return {} end, read = function() return nil end }

local ac = require("src.acxml")
local R, B, A, S = ac._resolve, ac._basic, ac._advanced, ac._setvar

-- Substitution: longest names, text after a name kept, unknown names lose their $, right to left.
assert(R("$p_hpp") == "75" and R("$p_hp") == "300" and R("$p_hpmax") == "1000", R("$p_hpp"))
assert(R("$p_hppx") == "75x", R("$p_hppx"))
assert(R("$U_x") == "U_x" and R("$nope") == "nope")
ac._uvars({ x = "p" })
assert(R("$p_hp$u_x") == "75", R("$p_hp$u_x"))
assert(R("$p_job") == "WAR/NIN" and R("$e_time") == "07.05" and R("$e_weather") == "fire")
-- Buffs by id and by name (the longest name; several ids under one name summed).
assert(R("$b_33") == "1" and R("$b_haste") == "1" and R("$b_5 ") == "1 ")
-- The single-digit quirk: $b_5=1 reads "5=" as the id.
assert(R("$b_5=1") == "11")
assert(B("b_5", "1") == false)
-- Arguments only inside a command match.
assert(R("$arg1") == "arg1")
ac._args({ [0] = "/acc", "x", "y", text = "x y" })
assert(R("$args|$arg0|$arg1|$arg2|$arg9") == "x y|/acc|x|y|arg9", R("$args|$arg0|$arg1|$arg2|$arg9"))
assert(R("$arg1st") == "x")
ac._args(nil)

-- Basic rules: | before &, operators anywhere, wildcards, the stray character of two-character operators.
assert(B("p_hpp", "<50") == false and B("p_hpp", ">=75") == true and B("p_hpp", "75") == true)
assert(B("p_mainjob", "WAR|NIN") == true and B("p_mainjob", "NIN|THF") == false)
assert(B("p_mainjob", "W*") == true and B("p_mainjob", "w*R") == true and B("p_mainjob", "WA**") == true and B("p_mainjob", "WAR**") == false)
assert(B("p_mainjob", "!WAR") == false and B("p_mainjob", "!=WAR") == true)
assert(B("p_mainjob", "THF|") == true, "an empty piece compares as numbers")
assert(B("p_hpp", ">50&<80") == true and B("p_hpp", ">50&<70") == false)

-- Advanced rules: precedence, left association, signed values (no hang), no crash on division by zero.
assert(A("$p_hpp > 50 & $p_mp >= 10") == true)
assert(A("($p_hp - 500) > 0") == false and A("$p_hp-500<0") == true)
assert(A("2^3^2=64.000000") == true and A("8-2-1=5") == true and A("7/2=3") == true and A("-7/2=-3") == true)
assert(A("5/0=0") == true and A("$p_job=WAR/NIN") == true)
-- "*" multiplies here: WAR=W* is WAR=0, true for any name (as in the original).
assert(A("$p_mainjob=war") == true and A("$p_mainjob=W*") == true and A("$p_mainjob=NIN") == false)
assert(A("($p_mainjob=WAR)|($p_mainjob=NIN)") == true and A("!true") == false)

-- setvar arithmetic: * / + - only, names with - become numbers.
ac._uvars({ n = "3" })
assert(S("$u_n + 1") == "4" and S("$u_n - 5") == "-2" and S("Mid-Acc") == "0" and S("Plain") == "Plain")

-- Augment path / rank / trial from extdata.
local ext = string.char(2, 0x20, 0x07, 0) .. string.rep("\0", 20)
local p, r = ac._aug_meta(ext)
assert(p == 4 and r == 1, p .. " " .. r)
