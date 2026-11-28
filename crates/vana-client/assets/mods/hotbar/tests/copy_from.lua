-- Hotbars' Copy From: the other jobs with macros of their own (not this one, not the shared ones) in a menu; the
-- one picked shows its macros as tiles under this job's, those this job has already dimmed; a click copies one,
-- Copy All the rest (the same ones skipped), Close puts them away. There is no Job field.

local abilities = { [547] = { name = "Provoke", kind = 1, targets = 0x20, recast = 30 } }
res = { spell = function() end, ability = function(id) return abilities[id] end,
  job = function(id) return ({ [1] = "WAR", [6] = "THF" })[id] end }
game = { player = function() return { main_job = 6, sub_job = 0 } end,
  spells = function() return {} end, abilities = function() return { 547 } end,
  find_items = function() return {} end }
chat = { add = function() end }
local on_window = ui.on
ui.on = function(id, f) if id == "manager" then H = f end; on_window(id, f) end
local MENU
ui.menu = function(items, h) MENU = { items = items, h = h } end
local DRAWN = {}
local update = ui.update
ui.update = function(w, id, t)
  if w == "manager" and t.draw then DRAWN[id] = t.draw end
  return update(w, id, t)
end
local data = require("src.data")
data.init({ read = function() return nil end, write = function() end, decode = json.decode, encode = json.encode })
local function entry(name, line) return { name = name, lines = { line } } end
data.save_entry("WAR", nil, entry("Shout", "/sh hi"))
data.save_entry("WAR", nil, entry("Yell", "/yell hi"))
data.save_entry("WAR", nil, entry("Same", "/p same"))
data.save_entry("THF", nil, entry("Same", "/p same"))
data.save_entry("THF", nil, entry("Mine", "/p mine"), true)
local manager = require("src.manager")
manager.hooks = { changed = function() end, rebind = function() end }
manager.open()

local ok = pcall(ui.get, "manager", "job")
assert(not ok, "the Job field is still there")
local jobs = data.jobs_with_macros()
assert(#jobs == 2 and jobs[1].job == "THF" and jobs[1].n == 1 and jobs[2].job == "WAR" and jobs[2].n == 3,
  "jobs: the shared macro counted, or one missing")

-- Copy From: the menu leaves THF out.
local function tiles(id)
  local n = 0
  for _, c in ipairs(DRAWN[id] or {}) do
    if c.type == "tip" then n = n + 1 end
  end
  return n
end
H("manager", { kind = "press", id = "lbtn", index = 4 })
assert(MENU and #MENU.items == 1 and MENU.items[1] == "WAR (3 macros)", MENU and MENU.items[1])
MENU.h(1)
assert(tiles("copy") == 3, "WAR's tiles: " .. tiles("copy"))
-- "Same" is THF's already: drawn dimmed (a dark cover over it).
local covers = 0
for _, c in ipairs(DRAWN.copy) do
  if c.type == "rect" and c.colour and c.colour[4] == 120 and not c.line then covers = covers + 1 end
end
assert(covers == 1, "the macro THF has is not dimmed")
-- A click on the first tile (Shout: by id, Shout, Yell, Same) copies it.
H("manager", { kind = "click", id = "copy", x = 5, y = 5 })
local names = {}
for _, it in ipairs(data.own_library("THF")) do names[#names + 1] = it.entry.name end
assert(table.concat(names, ",") == "Same,Shout", table.concat(names, ","))
-- Copy All: Yell; Shout and Same are here already.
H("manager", { kind = "press", id = "cbtn", index = 1 })
assert(#data.own_library("THF") == 3, "Copy All: " .. #data.own_library("THF"))
-- Close puts them away; a drop from them onto THF's tiles copies too (modern UI).
H("manager", { kind = "press", id = "cbtn", index = 2 })
data.save_entry("WAR", nil, entry("Dropped", "/p drop"))
H("manager", { kind = "press", id = "lbtn", index = 4 })
MENU.h(1)
H("manager", { kind = "drag_start", id = "copy", x = 5 + 3 * 58, y = 5, dx = 0, dy = 0 })
H("manager", { kind = "drag_end", id = "copy", x = 5 + 3 * 58, y = -40, dx = 0, dy = 0 })
H("manager", { kind = "drop", id = "tiles", x = 5, y = 5, from = "copy" })
local dropped = false
for _, it in ipairs(data.own_library("THF")) do dropped = dropped or it.entry.name == "Dropped" end
assert(dropped, "a drop from WAR's tiles did not copy the macro")
