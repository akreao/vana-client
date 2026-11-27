-- The bars' right-click menu: the slot's macro (when it was opened on one), then the actions.

local data = require("src.data")
local actions = require("src.actions")

local M = {}

local WIN = "slotmenu"
local GOLD = { 232, 210, 156 }
local DIM = { 160, 162, 172 }

-- items: { { label, fn }, ... }; `where` names the slot ("Bar 1, Slot 2", "LT A"), nil off the slots (no slot line).
function M.open(title, macro, where, items)
  local act = macro and actions.look(macro)
  local rows = {}
  for n, it in ipairs(items) do rows[n] = { it[1] } end
  local elements = {}
  if where then
    elements[1] = { type = "list", id = "head", height = 1, rows = { {
      macro and data.name_of(macro) or "Empty slot", where, icon = act and { kind = act.kind, id = act.id } or nil,
      colour = macro and GOLD or DIM } }, columns = { {}, { width = 70, align = "right" } } }
    elements[2] = { type = "separator" }
  end
  elements[#elements + 1] = { type = "list", id = "acts", rows = rows, height = #rows }
  ui.window(WIN, { title = title, width = 270, popup = true, elements = elements })
  ui.on(WIN, function(_, e)
    if e.kind ~= "select" or e.id ~= "acts" then return end
    ui.close(WIN)
    items[e.index][2]()
  end)
  ui.open(WIN)
end

return M
