-- GS1 share codes: a code made by XI: Forever Offline's own Gambits add-on (fixed below) reads here, ours comes out
-- byte for byte the same, and cut, changed or foreign codes are refused with a reason.

local S = require("src.share")

-- made by their add-on (share_code.lua, format 1) for this set
local THEIRS = "GS13DdoESngoKjQb6avx1sHnA4cq8BqdehKpLPGRAVCZVJgoLuZ5pKXUBPABmeUEuo74D575rnpebNweeoiZb1G5aaqsoe5"
local set = { job = "BRD", name = "Songs & Co.", rules = {
  { on = true, target = "MELEE", include = { "RANGED" }, exclude = { "TANK" }, cond = "NOT_STATUS", arg = "MARCH.2",
    kind = "s", name = "VICTORY_MARCH" },
  { on = false, target = "PARTY_LOWEST_HPP", cond = "HPP_LT", arg = "50", cond2 = "MPP_GTE", arg2 = "0", kind = "f",
    name = "CURE", retry = "5", level = "30", id = "w7" },
  { on = true, target = "TARGET", cond = "ALWAYS", arg = "", kind = "n" },
} }
assert(S.encode(set) == THEIRS, S.encode(set))

local d = assert(S.decode("  `" .. THEIRS:sub(1, 40) .. "\n" .. THEIRS:sub(41) .. "`  "))
assert(d.job == "BRD" and d.name == "Songs & Co." and #d.rules == 3)
local a, b, c = d.rules[1], d.rules[2], d.rules[3]
assert(a.on and a.target == "MELEE" and a.include[1] == "RANGED" and a.exclude[1] == "TANK" and a.arg == "MARCH.2")
assert(a.kind == "s" and a.name == "VICTORY_MARCH" and a.cond2 == nil)
assert(not b.on and b.arg2 == "0" and b.retry == "5" and b.level == "30" and b.id == "w7" and #b.include == 0)
assert(c.kind == "n" and c.name == nil and c.arg == "")
assert(S.encode(d) == THEIRS, "read and written again: the same code")

-- refused
local function refused(code, what)
  local x, why = S.decode(code)
  assert(x == nil and why:find(what, 1, true), tostring(why))
end
refused(THEIRS:sub(1, -2), "check number")
refused(THEIRS:sub(1, 20) .. "2" .. THEIRS:sub(22), "check number")
refused("GS1abc0def", "never has")
refused("GS2" .. THEIRS:sub(4), "newer format")
refused("hello", "not a Gambits share code")
assert(S.encode({ job = "WHM", name = "x", rules = { { raw = "?" } } }) == nil)
assert(S.encode({ job = "XYZ", name = "x", rules = set.rules }) == nil)
