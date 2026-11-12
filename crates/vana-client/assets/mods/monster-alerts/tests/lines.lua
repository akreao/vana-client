-- Monster Alerts: a monster's readied move and started spell make lines in the box; a watched move calls; a
-- player's action makes none.

local H, texts, calls = {}, {}, 0
local S = { on = true, moves = "Foot Kick", monsters = "" }
settings = { get = function(k) return S[k] end }
events = { on = function(n, f) H[n] = f end }
timer = { every = function() end }
local E = { [9] = { id = 9, name = "Forest Hare", kind = "mob" }, [5] = { id = 5, name = "Lnxcsfix", kind = "player" } }
game = { entity = function(id) return E[id] end, entities = function() return {} end }
res = { mob_skill = function(id) return id == 257 and "Foot Kick" or nil end, ability = function() return nil end,
  spell = function(id) return id == 144 and { name = "Fire" } or nil end }
audio = { play_ui = function() calls = calls + 1 end }
hud = { screen = function() return { width = 1920, height = 1080, ui_scale = 1 } end, clear = function() texts = {} end,
  text = function(k, s) texts[k] = s.text end }
load(TEST.source("init.lua"), "@init.lua")()
local function act(actor, cat, value)
  H.action({ actor = actor, category = cat, param = 24931, targets = { { id = 5, results = { { message = 43, value = value } } } } })
end
act(9, 7, 257)
act(9, 8, 144)
act(5, 7, 257)
H.frame()
assert(texts.l1 == "Forest Hare readies Foot Kick.", texts.l1)
assert(texts.l2 == "Forest Hare starts casting Fire on Lnxcsfix.", texts.l2)
assert(texts.l3 == nil and calls == 1)
