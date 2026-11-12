-- Roll Helper: a Chaos Roll of 4 is lucky (+25), a Double-Up typed then is held back; at 9 it goes through; a
-- bust ends the roll.

local H, F, said = {}, nil, {}
settings = { get = function() return true end }
events = { on = function(n, f) H[n] = f end }
filters = { typed = function(f) F = f end }
chat = { add = function(t) said[#said + 1] = t end }
game = { player = function() return { id = 5 } end }
res = { ability = function(id) return ({ [0x200 + 98] = { name = "Chaos Roll" }, [0x200 + 123] = { name = "Double-Up" } })[id] end,
  status = function() return nil end }
load(TEST.source("init.lua"), "@init.lua")()
local function act(param, msg, value)
  H.action({ actor = 5, category = 6, param = param, targets = { { id = 5, results = { { message = msg, value = value } } } } })
end
act(98, 420, 4)
assert(said[#said] == "Chaos Roll 4 (Lucky!): Attack % +25  (lucky 4, unlucky 8)", said[#said])
assert(F('/ja "Double-Up" <me>') == false)
act(123, 424, 9)
assert(said[#said]:find("^Chaos Roll 9: Attack %% %+17"), said[#said])
assert(F('/ja "Double-Up" <me>') == nil)
act(123, 426, 0)
assert(said[#said] == "Chaos Roll: Bust!")
assert(F('/ja "Double-Up" <me>') == nil)
