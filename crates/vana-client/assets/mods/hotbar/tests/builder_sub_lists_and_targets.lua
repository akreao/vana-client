-- Hotbars' Builder: Rune Enchantment opens its runes, as the game's menu does (a search finds them too); the name
-- follows the pick until typed over; an action is saved as one (no lines), its target from the Target dropdown; an
-- older entry with a line of its own opens as a Macro of all its lines.

local abilities = { [547] = { name = "Provoke", kind = 1, targets = 0x20, recast = 30 },
  [869] = { name = "Rune Enchantment", kind = 1, sub = 21, targets = 1 },
  [870] = { name = "Ignis", kind = 21, targets = 1, recast = 5 },
  [877] = { name = "Tenebrae", kind = 21, targets = 1, recast = 5 } }
res = { spell = function() end, ability = function(id) return abilities[id] end,
  job = function(id) return ({ [22] = "RUN" })[id] end }
game = { player = function() return { main_job = 22, sub_job = 0 } end,
  spells = function() return {} end, abilities = function() return { 547, 869, 870, 877 } end,
  find_items = function() return {} end }
chat = { add = function() end }
local on_window = ui.on
ui.on = function(id, f) if id == "manager" then H = f end; on_window(id, f) end
local data = require("src.data")
data.init({ read = function() return nil end, write = function() end, decode = json.decode,
  encode = json.encode })
local manager = require("src.manager")
manager.hooks = { changed = function() end, rebind = function() end }
manager.open()
local builder = require("src.builder")
local JA, MACRO = builder.type_index("ja"), builder.type_index(nil)
local function name() return ui.get("manager", "name").text end
-- New, Job ability: Provoke and the opener; the runes only inside it.
H("manager", { kind = "press", id = "lbtn", index = 1 })
H("manager", { kind = "set", id = "btype", index = JA })
H("manager", { kind = "select", id = "found", index = 2 })
-- Back, Ignis, Tenebrae.
H("manager", { kind = "select", id = "found", index = 2 })
assert(name() == "Ignis", name())
-- A search finds a rune from the top list; the name follows the pick.
H("manager", { kind = "select", id = "found", index = 1 })
H("manager", { kind = "edit", id = "q", text = "tene" })
H("manager", { kind = "select", id = "found", index = 1 })
assert(name() == "Tenebrae", name())
H("manager", { kind = "press", id = "bbar", index = 2 })
local e = data.library("RUN")[1].entry
assert(e.action.kind == "ja" and e.action.name == "Tenebrae" and e.action.target == "me" and not e.action.line)
assert(#(e.lines or {}) == 0 and data.entry_lines(e)[1] == '/ja "Tenebrae" <me>', data.entry_lines(e)[1])
-- Provoke starts on the target, the Target dropdown sets another; a name typed over stays.
H("manager", { kind = "press", id = "lbtn", index = 1 })
H("manager", { kind = "set", id = "btype", index = JA })
ui.update("manager", "name", { text = "Pull" })
H("manager", { kind = "select", id = "found", index = 1 })
assert(name() == "Pull", "a typed name was replaced by the pick's")
H("manager", { kind = "set", id = "target", index = 3 })
H("manager", { kind = "press", id = "bbar", index = 2 })
local pull
for _, it in ipairs(data.library("RUN")) do
  if it.entry.name == "Pull" then pull = it.entry end
end
assert(pull and pull.action.name == "Provoke" and pull.action.target == "bt", "Provoke's target")
-- An entry saved before with a line of its own and a line after it opens as a Macro of all its lines.
local old = data.save_entry("RUN", nil, { name = "Old", action = { kind = "ja", name = "Ignis", target = "me",
  line = '/ja "Ignis" <stpc>' }, lines = { "/p rune!" } })
manager.edit_entry(old)
assert(ui.get("manager", "btype").at == MACRO, "an entry with lines did not open as a Macro")
assert(ui.get("manager", "extra").text == '/ja "Ignis" <stpc>\n/p rune!', ui.get("manager", "extra").text)
