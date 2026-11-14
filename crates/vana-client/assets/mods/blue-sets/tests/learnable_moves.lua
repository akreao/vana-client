-- Blue Magic Sets: a monster readying a move that teaches a blue spell we lack gives a line and a call (only as
-- a Blue Mage, never for one we know); the codex lists what is missing and what teaches it.

local S = { on = true, alert = true, alert_sound = true }
local H, C, said, calls, wins = {}, {}, {}, {}, {}
local job, learned = 16, { [517] = true }
settings = { get = function(k) return S[k] end }
events = { on = function(n, f) H[n] = f end }
commands = { register = function(n, f) C[n] = f end }
timer = { every = function() end }
chat = { add = function(t) said[#said + 1] = t end }
audio = { play_ui = function(id) calls[#calls + 1] = id end }
ui = { window = function(id, w) wins[id] = w end, open = function() end, close = function() end,
  is_open = function() return false end, on = function() end, update = function() end }
game = {
  player = function() return { name = "Tester", main_job = job, main_level = 75 } end,
  entity = function(id) return { id = id, name = "Bat", kind = id == 9 and "player" or "mob" } end,
  spells = function() local t = {} for id in pairs(learned) do t[#t + 1] = id end return t end,
}
res = { spell = function(id) return ({ [515] = { name = "Maelstrom", levels = { [16] = 61 } },
    [517] = { name = "Metallic Body", levels = { [16] = 8 } } })[id] or { name = "Spell " .. id, levels = {} } end,
  mob_skill = function(id) return ({ [462] = "Maelstrom", [448] = "Metallic Body" })[id] end }
load(TEST.source("init.lua"), "@init.lua")()
local function readies(actor, move, cat)
  H.action({ actor = actor, category = cat or 7, param = 0, targets = { { id = 1, results = { { value = move, message = 43 } } } } })
end
readies(5, 462)
assert(said[#said] == "Blue Magic: Bat readies Maelstrom, which teaches Maelstrom.", said[#said])
assert(#calls == 1 and calls[1] == 18)
readies(5, 448)
readies(9, 462)
readies(5, 462, 8)
assert(#said == 1, "a learned spell, a player, a spell cast: none")
job = 1
readies(5, 462)
assert(#said == 1, "not a Blue Mage")
C.blu("codex")
local w = wins.blucodex
assert(w and w.elements[1].text:match("^1 of %d+ learned$"), w and w.elements[1].text)
local rows = w.elements[3].rows
local found
for _, r in ipairs(rows) do
  if r[1] == "Maelstrom" then found = r end
  assert(r[1] ~= "Metallic Body", "the Missing tab leaves learned spells out")
end
assert(found and found[2] == "61" and found[4] == "Maelstrom")
