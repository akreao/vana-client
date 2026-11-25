-- XIFO HC Icons: a shield left of each hardcore member's party row, none for the others, gone when they leave.

local ev, every, images, cmds, calls = {}, nil, {}, {}, {}
storage = { character = {}, save = function() end }
mods = { call = function(id, name, a) calls[#calls + 1] = name .. ":" .. tostring(a) end }
events = { on = function(n, f) ev[n] = f end }
timer = { after = function(_, f) f() end, every = function(_, f) every = f end }
chat = { add = function() end }
commands = { register = function(n, f) cmds[n] = f end }
local rows = {}
ui = { native = { party_rows = function() return rows end,
  rect = function(name) return name == "party" and { x = 880, y = 560, w = 380, h = 240 } or nil end } }
hud = { image = function(id, spec) images[id] = spec end, remove = function(id) images[id] = nil end }
load(TEST.source("init.lua"), "@init.lua")()

assert(calls[1] == "listen:[HC-ICONS]" and calls[2] == "send:!hcicons")
ev.xifo_link_line("[HC-ICONS]", " !Nymh=1,Fayd=4")
rows = { { panel = "party", slot = 1, name = "Nymh", x = 900, y = 600, w = 340, h = 60 },
  { panel = "party", slot = 2, name = "Rose", x = 900, y = 664, w = 340, h = 60 },
  { panel = "party", slot = 3, name = "Fayd", x = 900, y = 728, w = 340, h = 60 } }
every()
assert(images.hcparty1.path == "img/iron.png" and images.hcparty1.x == 880 - 22 - 2 and images.hcparty1.y == 619)
assert(images.hcparty2 == nil and images.hcparty3.path == "img/random.png")
rows = { rows[1] }
every()
assert(images.hcparty3 == nil, "a member who left loses the shield")
cmds.hcicons()
every()
assert(next(images) == nil, "turned off")
