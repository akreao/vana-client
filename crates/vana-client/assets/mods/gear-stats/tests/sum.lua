-- Gear Stats: descriptions and augments summed; conditional lines and DMG / Delay left out.

local C, W = {}, nil
settings = { get = function() return true end }
commands = { register = function(n, f) C[n] = f end }
timer = { every = function() end }
local open = false
ui = { window = function(_, spec) W = spec end, open = function() open = true end, close = function() end,
  is_open = function() return open end }
local desc = {
  [1] = "DMG:20 Delay:240\nSTR+2 Accuracy+5",
  [2] = "DEF:30 HP+20 MP-10\n\"Store TP\"+3 Haste+3%\nPet: Attack+10",
  [3] = "DEF:5 Accuracy+2\nSet: Increases Haste",
}
res = {
  item = function(id) return { description = desc[id] } end,
  augments = function(id) return { lines = id == 3 and { "STR+1", "Magic Atk. Bonus+4" } or {} } end,
}
game = {
  equipment = function() return { [0] = { id = 1, bag = 0, slot = 1 }, [5] = { id = 2, bag = 0, slot = 2 },
    [15] = { id = 3, bag = 0, slot = 3 } } end,
  items = function() return { { slot = 3, extdata = "x" } } end,
}
load(TEST.source("init.lua"), "@init.lua")()
C.gearstats()
local got = {}
for _, r in ipairs(W.elements[1].rows) do got[#got + 1] = r[1] .. "=" .. r[2] end
local s = table.concat(got, " ")
assert(s == '"Store TP"=+3 Accuracy=+7 DEF=35 HP=+20 Haste=+3% MP=-10 Magic Atk. Bonus=+4 STR=+3', s)
