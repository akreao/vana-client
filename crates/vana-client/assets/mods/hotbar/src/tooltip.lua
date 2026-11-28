-- One tooltip for every place a macro or an action shows (the Builder's results, the Library's tiles, the bars):
-- the name beside its icon; a spell's MP, cast and recast, an ability's TP, cost and recast; then its description.
-- A macro of typed lines shows its lines, a group its members.

local data = require("src.data")
local actions = require("src.actions")

local M = {}

-- Seconds as a recast reads: "45s", "5:00".
local function clock_of(secs)
  secs = math.floor((secs or 0) + 0.5)
  if secs < 60 then return secs .. "s" end
  return ("%d:%02d"):format(secs // 60, secs % 60)
end

-- The line of numbers and the description of the spell or ability `act` ({kind, id}), or nil, nil.
local function facts(act)
  local info, desc = {}, nil
  if act and act.kind == "spell" then
    local s = res.spell(act.id)
    if s then
      if (s.mp or 0) > 0 then info[#info + 1] = ("MP %d"):format(s.mp) end
      if (s.cast_time or 0) > 0 then info[#info + 1] = ("Cast %ss"):format((tostring(s.cast_time):gsub("%.0$", ""))) end
      if (s.recast or 0) > 0 then info[#info + 1] = "Recast " .. clock_of(s.recast) end
      desc = s.description
    end
  elseif act and act.kind == "ability" then
    local a = res.ability(act.id)
    if a then
      if a.tp and a.tp ~= 255 and a.tp > 0 then info[#info + 1] = ("TP %d"):format(a.tp) end
      if (a.cost or 0) > 0 then info[#info + 1] = ("Cost %d"):format(a.cost) end
      if (a.recast or 0) > 0 then info[#info + 1] = "Recast " .. clock_of(a.recast) end
      desc = a.description
    end
  end
  return #info > 0 and table.concat(info, "    ") or nil, desc
end

local function join(info, text)
  if info and text and text ~= "" then return info .. "\n\n" .. text end
  return info or text or ""
end

-- An action's tooltip: `icon` its spell / ability / item icon, `title` its name, `tip` the description to fall back on.
function M.action(icon, title, tip)
  local info, desc = facts(icon)
  return { title = title or "", text = join(info, desc or tip), icon = icon }
end

-- A macro's tooltip (a slot's, a tile's): its name beside the icon it shows; one action's numbers and description,
-- else its lines (a group: its members).
function M.macro(macro, fallback)
  local name = data.trim(macro.name)
  local title = name ~= "" and name or (fallback or "")
  local look = actions.look(macro) or nil
  local one = not macro.group and type(macro.action) == "table" and data.trim(macro.action.line) == ""
    and #(macro.lines or {}) <= 1
  if one then
    local act = actions.of(macro)
    local info, desc = facts(act or nil)
    if info or desc then return { title = title, text = join(info, desc), icon = look } end
  end
  return { title = title, text = data.tip_text(macro), icon = look }
end

return M
