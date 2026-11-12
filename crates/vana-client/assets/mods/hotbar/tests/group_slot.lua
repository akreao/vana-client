-- Hotbars' group slots: Rune Enchantment's list made a group in the Builder; its list runs the member picked and
-- starts on it the next time.

local abilities = { [869] = { name = "Rune Enchantment", kind = 1, sub = 21, targets = 1 },
  [870] = { name = "Ignis", kind = 21, targets = 1, recast = 5 },
  [877] = { name = "Tenebrae", kind = 21, targets = 1, recast = 5 } }
res = { spell = function() end, ability = function(id) return abilities[id] end,
  job = function(id) return ({ [22] = "RUN" })[id] end }
game = { player = function() return { main_job = 22, sub_job = 0 } end,
  spells = function() return {} end, abilities = function() return { 869, 870, 877 } end,
  find_items = function() return {} end,
  recasts = function() return { spells = {}, abilities = {}, totals = { spells = {}, abilities = {} } } end }
chat = { add = function() end }
SENT = {}
commands.run = function(line) SENT[#SENT + 1] = line end
local on_window = ui.on
ui.on = function(id, f)
  if id == "manager" then H = f elseif id == "group" then G = f end
  on_window(id, f)
end
local data = require("src.data")
data.init({ read = function() return nil end, write = function() end, decode = json.decode,
  encode = json.encode })
local manager = require("src.manager")
manager.hooks = { changed = function() end, rebind = function() end }
manager.open()
-- New, Job ability, into Rune Enchantment, Make a Group of These, Save.
H("manager", { kind = "press", id = "lbtn", index = 1 })
H("manager", { kind = "set", id = "btype", index = 2 })
H("manager", { kind = "select", id = "found", index = 1 })
H("manager", { kind = "press", id = "grp", index = 1 })
-- Another target for the group leaves the runes, which take only us, on <me>.
H("manager", { kind = "set", id = "target", index = 3 })
H("manager", { kind = "press", id = "bbtn", index = 1 })
local it = data.library("RUN")[1]
assert(it.entry.name == "Rune Enchantment" and #it.entry.group == 2, it.entry.name)
assert(it.entry.group[2].action.name == "Tenebrae" and it.entry.group[2].action.target == "me")
local macro = data.macro_of("RUN", it.id)
assert(macro.group and #macro.lines == 0)
-- Its list: the second runs, and is the one shown and offered first next time.
local chooser = require("src.chooser")
chooser.open(macro, nil, nil, 1)
assert(chooser.is_open() and chooser.bar() == 1)
G("group", { kind = "select", index = 2 })
assert(not chooser.is_open())
assert(SENT[1] == '/ja "Tenebrae" <me>', tostring(SENT[1]))
assert(data.last_member(macro) == 2)
assert(data.current_member(macro).name == "Tenebrae")
