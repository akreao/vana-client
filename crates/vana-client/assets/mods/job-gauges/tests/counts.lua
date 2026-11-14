-- Job Gauges: SCH 99 with 100 s of the stratagem recast left has 2 of 5 charges, the next in 4 s; a NIN sub sees
-- its shadows and tools.

local T, shown = nil, nil
settings = { get = function() return true end }
timer = { every = function(_, f) T = f end }
game = {
  player = function() return { main_job = 20, main_level = 99, sub_job = 13, sub_level = 49 } end,
  recasts = function() return { abilities = { [231] = 100 }, spells = {}, totals = { abilities = {}, spells = {} } } end,
  buffs = function() return { { id = 445 } } end,
  find_items = function(n) return n == "Shihei" and { { count = 30 } } or {} end,
}
ui = { window = function(_, spec) shown = spec.elements end, open = function() end, close = function() end }
load(TEST.source("init.lua"), "@init.lua")()
T()
local by = {}
for _, e in ipairs(shown) do by[e.label] = e.text end
assert(by["Stratagems"] == "2 / 5  +1 in 4s", by["Stratagems"])
assert(by["Shadows"] == "3", by["Shadows"])
assert(by["Shihei"] == "30")
assert(by["Ready"] == nil)
