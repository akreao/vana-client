-- Hotbars on a client without the dropdown element (1.0.10): the manager opens with choices in their place.

local abilities = { [547] = { name = "Provoke", kind = 1, targets = 0x20, recast = 30 } }
res = { spell = function() end, ability = function(id) return abilities[id] end,
  job = function(id) return ({ [1] = "WAR" })[id] end }
game = { player = function() return { main_job = 1, sub_job = 0 } end,
  spells = function() return {} end, abilities = function() return { 547 } end,
  find_items = function() return {} end }
chat = { add = function() end }
local real = ui.window
ui.window = function(id, s)
  for _, e in ipairs(s.elements or {}) do
    if e.type == "dropdown" then error('unknown element type "dropdown"') end
  end
  return real(id, s)
end
local on_window = ui.on
ui.on = function(id, f) if id == "manager" then H = f end; on_window(id, f) end
local data = require("src.data")
data.init({ read = function() return nil end, write = function() end, decode = json.decode,
  encode = json.encode })
local manager = require("src.manager")
manager.hooks = { changed = function() end, rebind = function() end }
manager.open()
H("manager", { kind = "press", id = "lbtn", index = 1 })
H("manager", { kind = "set", id = "btype", index = 2 })
H("manager", { kind = "select", id = "found", index = 1 })
assert(ui.get("manager", "line").text == '/ja "Provoke" <t>', ui.get("manager", "line").text)
