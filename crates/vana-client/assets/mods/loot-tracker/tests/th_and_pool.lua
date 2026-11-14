-- Loot Tracker: a TH rise (proc 603) shows on the target; a new pool item counts as its dropper's drop, with the
-- pool's time left.

local H, T, shown = {}, nil, {}
settings = { get = function() return true end }
events = { on = function(n, f) H[n] = f end }
timer = { every = function(_, f) T = f end }
commands = { register = function() end }
local mob = { id = 77, name = "Forest Hare", kind = "mob" }
game = { target = function() return mob end, entity = function(id) return id == 77 and mob or nil end,
  treasure = function() return { { slot = 0, item = 4358, count = 1, dropper = 77 } } end }
res = { item = function() return { name = "Hare Meat" } end }
ui = { window = function(_, s) shown = s.elements end, open = function() end, close = function() end }
os.clock = function() return 100 end
load(TEST.source("init.lua"), "@init.lua")()
H.action({ targets = { { id = 77, results = { { proc = { message = 603, value = 3 } } } } } })
H.battle_message(5, 77, 6, 0, 0)
T()
assert(shown[1].label == "Treasure Hunter: Forest Hare" and shown[1].text == "3", shown[1].text)
assert(shown[2].label == "Hare Meat" and shown[2].text == "5:00", shown[2].text)
assert(shown[3].text == "Forest Hare: 1 killed \u{2014} Hare Meat x1", shown[3].text)
