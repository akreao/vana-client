-- Hotbars' Builder lists the spells the magic list offers: learned and castable on our jobs at their levels
-- (RDM 54 / NIN 27: no Dia III, no Utsusemi: Ni), blue magic only while set.

local spells = {
  [4] = { name = "Cure IV", mp = 88, kind = 1, levels = { [3] = 41, [5] = 48 } },
  [24] = { name = "Dia II", mp = 30, kind = 1, targets = 0x20, levels = { [3] = 36, [5] = 31 },
    description = "Deals light damage over time and lowers defense." },
  [25] = { name = "Dia III", mp = 45, kind = 1, targets = 0x20, levels = { [5] = 75 } },
  [338] = { name = "Utsusemi: Ichi", mp = 0, kind = 4, levels = { [13] = 12 } },
  [339] = { name = "Utsusemi: Ni", mp = 0, kind = 4, levels = { [13] = 37 } },
  [513] = { name = "Venom Shell", mp = 86, kind = 6, levels = { [16] = 42 } },
  [519] = { name = "Screwdriver", mp = 21, kind = 6, levels = { [16] = 26 } } }
res = { spell = function(id) return spells[id] end, ability = function() end,
  job = function(id) return ({ [5] = "RDM", [13] = "NIN", [16] = "BLU" })[id] end }
local me = { main_job = 5, main_level = 54, sub_job = 13, sub_level = 27, mp = 40 }
game = { player = function() return me end, spells = function() return { 4, 24, 25, 338, 339 } end,
  abilities = function() return {} end, job_data = function() return {} end,
  find_items = function() return {} end }
chat = { add = function() end }
local builder = require("src.builder")
local function names()
  local out = {}
  for _, e in ipairs(builder.find("ma", "")) do out[#out + 1] = e.name end
  return table.concat(out, ", ")
end
local got = names()
assert(got == "Cure IV, Dia II, Utsusemi: Ichi", got)
-- A spell's default target: the party picker for one that takes no other player (Refresh 0x05), the
-- alliance's for 0x0d, any player for Protect's 0x1d, the current target for one that hits enemies.
assert(builder.default_target(0x05) == "stpt" and builder.default_target(0x0d) == "stal")
assert(builder.default_target(0x1d) == "stpc" and builder.default_target(0x3f) == "t")
assert(builder.default_target(0x01) == "me")
-- New: the Builder lists them, each row with its description.
local shown, update = nil, ui.update
ui.update = function(w, id, t)
  if id == "found" and t.rows then shown = t.rows end
  return update(w, id, t)
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
local count = ui.get("manager", "fcount").text
assert(count == "3 found.", count)
assert(shown[2][1] == "Dia II" and shown[2].tip == spells[24].description and shown[1].tip == nil)
H("manager", { kind = "select", id = "found", index = 2 })
local line = ui.get("manager", "line").text
assert(line == '/ma "Dia II" <t>', line)
-- BLU 50 / NIN 25: Venom Shell set, Screwdriver learned but not set.
me = { main_job = 16, main_level = 50, sub_job = 13, sub_level = 25 }
game.spells = function() return { 338, 513, 519 } end
game.job_data = function() return { main = { blue_spells = { 513 } } } end
got = names()
assert(got == "Utsusemi: Ichi, Venom Shell", got)
-- The bar's shading uses the same rule, and the MP.
local actions = require("src.actions")
me = { main_job = 5, main_level = 54, sub_job = 13, sub_level = 27, mp = 40 }
assert(not actions.usable({ kind = "spell", id = 25 }, me))
assert(actions.usable({ kind = "spell", id = 24 }, me))
assert(not actions.usable({ kind = "spell", id = 4 }, me))
