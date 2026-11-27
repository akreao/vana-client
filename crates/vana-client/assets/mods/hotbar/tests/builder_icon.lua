-- Hotbars' Builder: the icon tile beside the name is blank, then the pick's until one is picked for the macro (the
-- icon picker: a click on a tile); that one stays through other picks and is saved; No Icon gives back the pick's.
-- A Macro's lines come from its text area, one a line.

local spells = {
  [1] = { name = "Cure", kind = 1, mp = 8, targets = 0x1f, levels = { [3] = 1 } },
  [2] = { name = "Cure II", kind = 1, mp = 24, targets = 0x1f, levels = { [3] = 11 } },
}
res = setmetatable({ spell = function(id) return spells[id] end, job = function(id) return ({ [3] = "WHM" })[id] end },
  { __index = function() return function() end end })
game = setmetatable({ player = function() return { main_job = 3, main_level = 99, sub_job = 0 } end,
  spells = function() return { 1, 2 } end }, { __index = function() return function() return {} end end })
chat = { add = function() end }
local HANDLERS = {}
local on_window = ui.on
ui.on = function(id, f) HANDLERS[id] = f; on_window(id, f) end
local DRAWN, SPECS = {}, {}
local update = ui.update
ui.update = function(w, id, t)
  if t.draw then DRAWN[w .. ":" .. id] = t.draw end
  return update(w, id, t)
end
local window = ui.window
ui.window = function(id, s)
  SPECS[id] = s
  return window(id, s)
end
local data = require("src.data")
data.init({ read = function() return nil end, write = function() end, decode = json.decode, encode = json.encode })
local manager = require("src.manager")
manager.hooks = { changed = function() end, rebind = function() end }
manager.open()
local H = function(e) HANDLERS.manager("manager", e) end
local function bicon()
  for _, c in ipairs(DRAWN["manager:bicon"] or {}) do
    if c.type == "icon" then return c.kind .. ":" .. c.id end
  end
  return "blank"
end

H({ kind = "press", id = "lbtn", index = 1 })
-- The tile floats beside the name.
local float
for _, e in ipairs(SPECS.manager.elements) do
  if e.id == "bicon" then float = e.float end
end
assert(float == "right", "the icon tile does not float beside the name")
assert(bicon() == "blank", "a new macro's icon is not blank: " .. bicon())
H({ kind = "select", id = "found", index = 1 })
assert(bicon() == "spell:1", "the icon does not follow the pick: " .. bicon())
H({ kind = "select", id = "found", index = 2 })
assert(bicon() == "spell:2", bicon())
-- The picker: a click on the tile opens it on Spells; a click on Cure's tile picks it.
H({ kind = "click", id = "bicon", x = 5, y = 5 })
assert(ui.is_open("hbicons"), "the icon picker did not open")
HANDLERS.hbicons("hbicons", { kind = "click", id = "grid", x = 5, y = 5 })
assert(not ui.is_open("hbicons") and bicon() == "spell:1", "the picked icon: " .. bicon())
-- Picked: another pick leaves it.
H({ kind = "select", id = "found", index = 2 })
assert(bicon() == "spell:1", "a pick replaced the picked icon")
H({ kind = "press", id = "bbar", index = 2 })
local e = data.library("WHM")[1].entry
assert(e.name == "Cure II" and e.icon and e.icon.kind == "spell" and e.icon.id == 1, "the icon was not saved")
-- No Icon: the pick's again, nothing saved.
manager.edit_entry(data.library("WHM")[1].id)
H({ kind = "click", id = "bicon", x = 5, y = 5 })
HANDLERS.hbicons("hbicons", { kind = "press", id = "pbtn", index = 3 })
assert(bicon() == "spell:2", "No Icon: " .. bicon())
H({ kind = "press", id = "bbar", index = 2 })
assert(not data.library("WHM")[1].entry.icon, "No Icon kept an icon")

-- A Macro: its lines from the text area, one a line.
H({ kind = "press", id = "lbtn", index = 1 })
H({ kind = "set", id = "btype", index = require("src.builder").type_index(nil) })
local area
for _, x in ipairs(SPECS.manager.elements) do
  if x.id == "extra" then area = x.type end
end
assert(area == "textarea", "the Macro's lines are not a text area: " .. tostring(area))
ui.update("manager", "name", { text = "Talk" })
ui.update("manager", "extra", { text = "/p one | still one\n/wait 1\n/p two" })
H({ kind = "press", id = "bbar", index = 2 })
local talk
for _, it in ipairs(data.library("WHM")) do
  if it.entry.name == "Talk" then talk = it.entry end
end
assert(talk and table.concat(talk.lines, ";") == "/p one | still one;/wait 1;/p two", talk and table.concat(talk.lines, ";"))
