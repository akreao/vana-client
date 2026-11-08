-- A slot's right-click menu (a pop-up at the pointer) and the pickers it opens: one of our spells, one of our
-- abilities (job abilities, weapon skills, pet commands) or an existing macro (the game's macro palette or another
-- hotbar slot) put into the slot as a ready macro. The manager's editor stays for writing one by hand.

local data = require("src.data")
local jobs = require("src.jobs")
local keys = require("src.keys")
local actions = require("src.actions")

local M = {}

-- Set by init.lua: edit_slot(bar, slot, from), open_manager(), changed()
M.hooks = {}

local MENU, PICK = "slotmenu", "slotpick"
local GOLD = { 232, 210, 156 }
local DIM = { 160, 162, 172 }

local at = nil        -- {bar, slot, from} the menu and the pickers work on
local all = {}        -- the picker's whole list: {label, info, icon, macro}
local rows = {}       -- what it shows after the filter

-- The set a slot's new macro goes into: the sub job's when the slot shows the sub set's.
local function store(macro)
  local main, sub = jobs.current()
  if not (main and at) then return end
  data.set_macro(main, at.from == "sub" and sub or nil, at.bar, at.slot, macro)
  if M.hooks.changed then M.hooks.changed() end
  chat.add(("Hotbars: %s is on bar %d, slot %d (%s)."):format(macro.name, at.bar, at.slot, keys.hint(at.bar, at.slot)))
end

local function quote(name)
  return name:find(" ") and ('"' .. name .. '"') or name
end

-- <me> for an action that can only aim at ourselves (target bits: 1 = self), else <t>.
local function aim(targets)
  return (tonumber(targets) or 0) == 1 and "<me>" or "<t>"
end

-- ---------------------------------------------------------------------------
-- The pickers
-- ---------------------------------------------------------------------------

local function spells()
  local out = {}
  for _, id in ipairs(game.spells()) do
    local s = res.spell(id)
    if s and s.name and s.name ~= "" then
      local info = s.kind == 8 and "Trust" or (s.mp and s.mp > 0 and ("%d MP"):format(s.mp) or "")
      out[#out + 1] = { label = s.name, info = info, icon = { kind = "spell", id = id },
        macro = { name = s.name, lines = { ("/ma %s %s"):format(quote(s.name), aim(s.targets)) } } }
    end
  end
  return out
end

local function abilities()
  local out = {}
  for _, id in ipairs(game.abilities()) do
    local a = res.ability(id)
    if a and a.name and a.name ~= "" then
      local cmd, info = "/ja", "Ability"
      if id < 0x200 then
        cmd, info = "/ws", "Weapon skill"
      elseif id >= 0x400 then
        cmd, info = "/pet", "Pet command"
      end
      out[#out + 1] = { label = a.name, info = info, icon = { kind = "ability", id = id },
        macro = { name = a.name, lines = { ("%s %s %s"):format(cmd, quote(a.name), aim(a.targets)) } } }
    end
  end
  return out
end

local function lines_of(m)
  local out = {}
  for _, l in ipairs(m.lines or {}) do
    if data.trim(l) ~= "" then out[#out + 1] = l end
  end
  return out
end

local function macros()
  local out = {}
  local p = game.macros()
  if p then
    for _, row in ipairs({ { "ctrl", "Ctrl" }, { "alt", "Alt" } }) do
      for i, m in ipairs(p[row[1]] or {}) do
        local ls = lines_of(m)
        if #ls > 0 then
          local name = data.trim(m.title or "") ~= "" and data.trim(m.title) or ls[1]:sub(1, 24)
          local act = actions.of({ lines = ls })
          out[#out + 1] = { label = name, info = ("%s+%d, %s"):format(row[2], i % 10, p.book_title or "palette"),
            icon = act and { kind = act.kind, id = act.id } or nil, macro = { name = name, lines = ls } }
        end
      end
    end
  end
  local main, sub = jobs.current()
  if main then
    local eff = data.effective(main, sub)
    for b, slots in pairs(eff) do
      for s, cell in pairs(slots) do
        if cell.macro and not (at and b == at.bar and s == at.slot) then
          local m = cell.macro
          local act = actions.of(m)
          out[#out + 1] = { label = data.trim(m.name) ~= "" and m.name or (m.lines[1] or "?"),
            info = ("Bar %d, slot %d"):format(b, s), icon = act and { kind = act.kind, id = act.id } or nil,
            macro = { name = m.name, label = m.label, lines = m.lines } }
        end
      end
    end
  end
  return out
end

local function show(filter)
  local f = data.trim(filter or ""):lower()
  rows = {}
  local list = {}
  for _, e in ipairs(all) do
    if f == "" or e.label:lower():find(f, 1, true) or (e.info or ""):lower():find(f, 1, true) then
      rows[#rows + 1] = e
      list[#list + 1] = { e.label, e.info or "", icon = e.icon }
    end
  end
  ui.update(PICK, "list", { rows = list, at = 1 })
  ui.update(PICK, "count", { text = #rows == 0 and "Nothing matches." or ("%d to choose from"):format(#rows) })
end

local TITLES = { spell = "Cast a Spell", ability = "Use an Ability", macro = "Assign a Macro" }

local function pick(kind)
  all = kind == "spell" and spells() or kind == "ability" and abilities() or macros()
  table.sort(all, function(a, b)
    if kind == "macro" and a.info ~= b.info then return a.info < b.info end
    return a.label < b.label
  end)
  ui.window(PICK, {
    title = TITLES[kind],
    width = 400,
    modal = true,
    help = "Type to filter, then pick one: it goes into the slot as a macro you can still edit in the manager.",
    elements = {
      { type = "text", id = "where", text = ("Bar %d, slot %d (%s)"):format(at.bar, at.slot, keys.hint(at.bar, at.slot)),
        colour = GOLD },
      { type = "input", id = "filter", label = "Filter", text = "", max = 32, tip = "Part of a name to narrow the list." },
      { type = "list", id = "list", rows = {}, height = 12,
        columns = { { title = kind == "macro" and "Macro" or "Name" }, { title = kind == "macro" and "From" or "",
          width = kind == "macro" and 150 or 110, align = "right" } } },
      { type = "text", id = "count", text = "", dim = true },
      { type = "buttons", id = "btn", items = { "Assign", "Cancel" } },
    },
  })
  ui.on(PICK, function(_, e)
    if e.kind == "edit" and e.id == "filter" then
      show(e.text)
    elseif e.kind == "select" and e.id == "list" then
      local r = rows[e.index]
      if r then
        ui.close(PICK)
        store(r.macro)
      end
    elseif e.kind == "press" and e.id == "btn" then
      if e.index == 1 then
        local r = rows[ui.get(PICK, "list").at or 1]
        if r then store(r.macro) end
      end
      ui.close(PICK)
    end
  end)
  show("")
  ui.open(PICK)
end

-- ---------------------------------------------------------------------------
-- The menu
-- ---------------------------------------------------------------------------

local ITEMS = {
  { "Cast a spell...", "spell" },
  { "Use an ability...", "ability" },
  { "Assign a macro...", "macro" },
  { "Edit the macro", "edit" },
  { "Clear the slot", "clear" },
  { "Open the manager", "manager" },
}

function M.open(bar, slot, from)
  at = { bar = bar, slot = slot, from = from }
  local main, sub = jobs.current()
  local macro = main and data.resolve(main, sub, bar, slot)
  local act = macro and actions.of(macro)
  local list = {}
  local icons = { spell = { kind = "spell", id = 1 }, ability = { kind = "ability", id = 0x200 + 35 } }
  for _, it in ipairs(ITEMS) do
    if it[2] ~= "clear" or macro then
      list[#list + 1] = { it[1], icon = icons[it[2]] }
    end
  end
  ui.window(MENU, {
    title = ("Bar %d, Slot %d"):format(bar, slot),
    width = 250,
    popup = true,
    elements = {
      { type = "list", id = "head", height = 1, rows = { {
        macro and (data.trim(macro.name) ~= "" and macro.name or (macro.lines[1] or "")) or "Empty slot",
        keys.hint(bar, slot), icon = act and { kind = act.kind, id = act.id } or nil,
        colour = macro and GOLD or DIM } }, columns = { {}, { width = 60, align = "right" } } },
      { type = "separator" },
      { type = "list", id = "acts", rows = list, height = #list },
    },
  })
  ui.on(MENU, function(_, e)
    if e.kind ~= "select" or e.id ~= "acts" then return end
    local want = list[e.index] and list[e.index][1]
    local what
    for _, it in ipairs(ITEMS) do
      if it[1] == want then what = it[2] end
    end
    ui.close(MENU)
    if what == "spell" or what == "ability" or what == "macro" then
      pick(what)
    elseif what == "edit" then
      M.hooks.edit_slot(bar, slot, from)
    elseif what == "clear" and main then
      data.clear(main, from == "sub" and sub or nil, bar, slot)
      if M.hooks.changed then M.hooks.changed() end
    elseif what == "manager" then
      M.hooks.open_manager()
    end
  end)
  ui.open(MENU)
end

return M
