-- Pending Actions: a melee round or another spell's finish while a spell waits is not its answer, nor a message
-- that is not one of the spell's refusals (akreao/vana-client#243); the spell's start, finish and refusals are, and
-- each kind matches its ids. The game scripts' checks ("You cannot learn...") and the target check's refusals refuse
-- too.

local H, OUT, chip = {}, {}, nil
local T = 100
os.clock = function() return T end
settings = { get = function() return true end }
events = { on = function(n, f) H[n] = f end }
local IN = {}
packets = { on_out = function(id, f) OUT[id] = f end, on_in = function(id, f) IN[id] = f end }
game = { player = function() return { id = 5 } end, entity = function() return nil end,
  items = function() return { { slot = 3, id = 4112 } } end }
res = { spell = function() return { name = "Cure" } end, ability = function() return { name = "Provoke" } end,
  item = function() return { name = "Potion" } end,
  message = function() return { text = "No." } end }
text = { strip = function(s) return s end }
ui = { window = function(_, spec) chip = spec.elements[1] end, update = function(_, _, v) chip = v end,
  open = function() end, close = function() chip = nil end }
load(TEST.source("init.lua"), "@init.lua")()
local function ask(cat, param) OUT[0x01A]({ data = string.pack("<I4I4I2I2I4", 0, 0, 0, cat, param) }) end
local function item() OUT[0x037]({ data = string.pack("<I4I4I4BBI4I4", 0, 0, 0, 0, 0, 0, 0)
  :sub(1, 0x0e) .. string.pack("<BBI4I4", 3, 0, 0, 0) }) end
local function state() T = T + 0.2 H.frame() return chip and chip.tone end
local function act(cat, param, value)
  local targets = value and { { id = 9, results = { { value = value } } } } or {}
  H.action({ actor = 5, category = cat, param = param, targets = targets })
end
-- A spell (Cure, 1): a melee round, another spell's finish, an interrupt and another spell's start leave
-- it waiting.
ask(3, 1)
act(1, 0x746b6174) act(4, 2) act(8, 0x6f737073, 1) act(8, 0x6f736163, 2)
assert(state() == "waiting")
-- Melee-side chatter of ours (71 with no spell id, experience) is no refusal; its start answers.
H.battle_message(5, 9, 71, 0, 0) H.battle_message(5, 5, 8, 300, 0)
assert(state() == "waiting")
act(8, 0x6f736163, 1)
assert(state() == nil)
-- Its finish answers too; its refusal (47 with its id) refuses.
ask(3, 1) act(4, 1) assert(state() == nil)
ask(3, 1) H.battle_message(5, 9, 47, 2, 0) assert(state() == "waiting")
H.battle_message(5, 9, 47, 1, 0) assert(state() == "refused")
T = T + 10 state()
-- A job ability (Provoke, 0x01A param 0x23): its finish's param, not another's.
ask(9, 0x23) act(6, 0x24) assert(state() == "waiting")
act(6, 0x23) assert(state() == nil)
ask(9, 0x23) H.battle_message(5, 9, 88, 0x24, 0) assert(state() == "waiting")
H.battle_message(5, 9, 88, 0, 0) assert(state() == "refused")
T = T + 10 state()
-- A weapon skill: its start's record value; a monster skill finish answers whatever its id.
ask(7, 32) act(7, 0x65746163, 33) assert(state() == "waiting")
act(7, 0x65746163, 32) assert(state() == nil)
ask(7, 32) act(11, 999) assert(state() == nil)
ask(7, 32) H.battle_message(5, 9, 192, 0, 0) assert(state() == "refused")
T = T + 10 state()
-- An item: its category alone; 62 only with no id or its id.
item() act(1, 0) assert(state() == "waiting")
act(9, 0) assert(state() == nil)
item() H.battle_message(5, 9, 62, 4113, 0) assert(state() == "waiting")
H.battle_message(5, 9, 62, 4112, 0) assert(state() == "refused")
T = T + 10 state()
-- The scripts' checks: a scroll the player cannot learn yet (95, naming the spell it teaches), a
-- medicated player's potion (111 with the item id, not another's), the 0 a stun sends with the item id
-- (not a bare 0).
item() H.battle_message(5, 5, 95, 1, 0) assert(state() == "refused")
T = T + 10 state()
item() H.battle_message(5, 5, 111, 4113, 0) H.battle_message(5, 5, 0, 0, 0) assert(state() == "waiting")
H.battle_message(5, 5, 111, 4112, 0) assert(state() == "refused")
T = T + 10 state()
item() H.battle_message(5, 5, 0, 4112, 0) assert(state() == "refused")
T = T + 10 state()
-- A spell's script check names it (316 not here); the target check names none (12 claimed).
ask(3, 1) H.battle_message(5, 9, 316, 2, 0) H.battle_message(5, 9, 316, 0, 0) assert(state() == "waiting")
H.battle_message(5, 9, 316, 1, 0) assert(state() == "refused")
T = T + 10 state()
ask(3, 1) H.battle_message(5, 9, 12, 0, 0) assert(state() == "refused")
T = T + 10 state()
-- An ability's script check names it as both values (523 already active); a weapon skill's target check.
ask(9, 0x23) H.battle_message(5, 5, 523, 0x24, 0x24) assert(state() == "waiting")
H.battle_message(5, 5, 523, 0x23, 0x23) assert(state() == "refused")
T = T + 10 state()
ask(7, 32) H.battle_message(5, 9, 446, 0, 0) assert(state() == "refused")
T = T + 10 state()
-- A player blocking outside help answers with system message 225 (0x053); another system message does
-- not, nor 225 for a weapon skill, and none after the refusal window.
local function sys(n) IN[0x053]({ data = string.pack("<I4I4I4I2I2", 0, 0, 0, n, 0) }) end
ask(3, 1) sys(224) assert(state() == "waiting")
sys(225) assert(state() == "refused")
T = T + 10 state()
ask(7, 32) sys(225) assert(state() == "waiting")
T = T + 10 state()
ask(3, 1) T = T + 3 sys(225) assert(state() == "waiting")
