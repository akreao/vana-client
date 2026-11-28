-- Hotbars: the manager's tabs are Library and Help (keys are assigned on the bars); Help explains the mode in use.

game = { player = function() return { main_job = 1, sub_job = 0 } end,
  spells = function() return {} end, abilities = function() return {} end, find_items = function() return {} end }
res = { spell = function() end, ability = function() end, job = function(id) return ({ [1] = "WAR" })[id] end }
chat = { add = function() end }
local data = require("src.data")
data.init({ read = function() return nil end, write = function() end, decode = json.decode, encode = json.encode })
local mode = require("src.mode")
local manager = require("src.manager")
manager.hooks = { changed = function() end, rebind = function() end }

mode.set("hotbars")
manager.open()
local tabs = ui.get("manager", "tabs")
assert(tabs.items == nil or (#tabs.items == 2 and tabs.items[2] == "Help"))
local assign = false
for i = 1, 14 do
  if ui.get("manager", "help" .. i).text:find("Assign Keys", 1, true) then assign = true end
end
assert(assign, "the hotbars' help tells how to assign keys")
assert(ui.get("manager", "help1").text == "Keyboard hotbars", ui.get("manager", "help1").text)

mode.set("gamepad")
manager.refresh()
assert(ui.get("manager", "help1").text == "Gamepad crossbars", ui.get("manager", "help1").text)
local found = false
for i = 1, 14 do
  if ui.get("manager", "help" .. i).text:find("Trigger + Back", 1, true) then found = true end
end
assert(found, "the gamepad's help tells how to assign")
mode.set("hotbars")
manager.refresh()
assert(ui.get("manager", "help13").text == "", "the hotbars' help has fewer lines: the rest are blank")
