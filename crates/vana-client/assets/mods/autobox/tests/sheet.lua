-- Autobox's window: //autobox opens the sheet with the main job's preset, rules start switched off, a new rule is
-- saved from the rule window, and the switch turns the engine on, which then casts.

C, E, cast, said, tick = {}, {}, {}, {}, nil
settings = { get = function() return true end }
commands = { register = function(n, f) C[n] = f end, usage = function() return {} end }
events = { on = function(n, f) E[n] = f end }
chat = { add = function(t) said[#said + 1] = t end }
timer = { every = function(_, f) tick = f end }
actions = { cast = function(id, tag) cast[#cast + 1] = id .. tag end, ability = function() end }
local spells = { [43] = { name = "Protect", targets = 1, kind = 1 }, [1] = { name = "Cure", targets = 0x1F, kind = 1 } }
res = {
  job = function(id) return ({ [3] = "WHM" })[id] end,
  spell = function(id) return spells[id] end,
  ability = function() return nil end,
  status = function(id) return ({ [40] = "Protect" })[id] end,
  item = function() return nil end,
}
game = {
  player = function() return { id = 1, main_job = 3, pos = { x = 0, z = 0 }, hpp = 100, mpp = 100, tp = 0, status = 0 } end,
  buffs = function() return {} end,
  spells = function() return { 1, 43 } end,
  abilities = function() return {} end,
  party = function() return {} end,
  info = function() return { event = false } end,
  entity = function() return nil end,
  target = function() return nil end,
  cast = function() return nil end,
  usable = function() return true end,
  equipment = function() return {} end,
}
local on_window, update = ui.on, ui.update
H = {}
ui.on = function(id, f) H[id] = f; on_window(id, f) end
local drawn = {}
ui.update = function(id, el, fields)
  drawn[id .. "." .. el] = drawn[id .. "." .. el] or {}
  for k, v in pairs(fields) do drawn[id .. "." .. el][k] = v end
  return update(id, el, fields)
end
load(TEST.source("init.lua"), "@init.lua")()

C.autobox()
assert(ui.is_open("autobox"), "//autobox opened no window")
local rows = drawn["autobox.rules"].rows
assert(#rows == 8 and rows[1][3] == "Party member" and rows[1][5] == "Best Cure", rows[1][5])
assert(storage.character.rules.WHM, "the preset was saved")
assert(drawn["autobox.switch"].marked == 2, "rules start off")

-- Off: nothing is cast.
tick() tick()
assert(#cast == 0)

-- A new rule: me, always, Cure.
H.autobox("autobox", { kind = "press", id = "ops", index = 1 })
assert(ui.is_open("autobox_rule"))
H.autobox_rule("autobox_rule", { kind = "set", id = "what", index = 1 })
H.autobox_rule("autobox_rule", { kind = "press", id = "done", index = 1 })
assert(not ui.is_open("autobox_rule"), drawn["autobox_rule.problem"] and drawn["autobox_rule.problem"].text)
assert(#storage.character.rules.WHM == 9 and storage.character.rules.WHM[9].action.name == "Cure")

-- An edit saved after its rule moved goes to that rule, wherever it is now.
-- (a click moves the list's cursor, then sends select)
ui.update("autobox", "rules", { at = 2 })
H.autobox("autobox", { kind = "select", id = "rules", index = 2 })
local second = storage.character.rules.WHM[2]
H.autobox("autobox", { kind = "press", id = "ops", index = 5 }) -- Up: now first
H.autobox_rule("autobox_rule", { kind = "set", id = "on", index = 2 })
H.autobox_rule("autobox_rule", { kind = "press", id = "done", index = 1 })
assert(storage.character.rules.WHM[1].on == false and storage.character.rules.WHM[1].when.value == second.when.value,
  "the moved rule was edited")
assert(storage.character.rules.WHM[2].on ~= false, "the rule now second was left alone")
-- An edit of a deleted rule is refused.
ui.update("autobox", "rules", { at = 1 })
H.autobox("autobox", { kind = "select", id = "rules", index = 1 })
H.autobox("autobox", { kind = "press", id = "ops", index = 4 })
H.autobox_rule("autobox_rule", { kind = "press", id = "done", index = 1 })
assert(ui.is_open("autobox_rule") and drawn["autobox_rule.problem"].text:find("deleted"), "deleted rule not saved")
H.autobox_rule("autobox_rule", { kind = "press", id = "done", index = 2 })

-- On: the first rule that holds is Protect (we lack it, not engaged).
H.autobox("autobox", { kind = "set", id = "switch", index = 1 })
assert(storage.character.on == true)
tick() tick()
assert(cast[1] == "43<me>", tostring(cast[1]))
C.autobox("off")
assert(storage.character.on == false and said[#said] == "Autobox: off.", said[#said])
