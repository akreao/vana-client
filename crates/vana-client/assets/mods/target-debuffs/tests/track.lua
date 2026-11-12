-- Target Debuffs: an effect landing on the target shows its icon; it goes when it wears off, and all go when the
-- target is defeated.

local H, icons = {}, {}
settings = { get = function() return true end }
events = { on = function(n, f) H[n] = f end }
game = { target = function() return { id = 77 } end }
res = { status = function(id) return "Effect " .. id end }
ui = { native = { rect = function() return nil end } }
hud = { screen = function() return { width = 1920, height = 1080, ui_scale = 1 } end,
  clear = function() icons = {} end, icon = function(k, s) icons[k] = s.id end }
load(TEST.source("init.lua"), "@init.lua")()
local function count() local n = 0 for _ in pairs(icons) do n = n + 1 end return n end
H.action({ actor = 5, category = 4, targets = { { id = 77, results = { { message = 236, value = 4 } } } } })
H.action({ actor = 5, category = 4, targets = { { id = 77, results = { { message = 2, value = 30 } } } } })
H.frame()
assert(icons.d4 == 4 and count() == 1)
H.action({ actor = 5, category = 4, targets = { { id = 77, results = { { message = 237, value = 13 } } } } })
H.frame()
assert(count() == 2)
H.battle_message(5, 77, 206, 4, 0)
H.frame()
assert(icons.d4 == nil and icons.d13 == 13)
H.battle_message(5, 77, 6, 0, 0)
H.frame()
assert(count() == 0)
