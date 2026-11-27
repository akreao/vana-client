-- The icon picker: every spell, ability, weapon skill, pet command and status icon, and the items held, as tiles
-- (drawn as the Library's) in pages; a click picks one for a macro. Opened over the Builder.

local M = {}

local WIN = "hbicons"
local CW, T, TG = 480, 52, 6
local COLS = math.floor((CW + TG) / (T + TG))
local ROWS = 5
local PER = COLS * ROWS
local TILE = { 10, 17, 24, 255 }
local TILE_HOVER = { 20, 33, 44, 255 }
local TILE_EDGE = { 184, 139, 73, 205 }
local TILE_SEL = { 255, 214, 120, 255 }
local TILE_TEXT = { 236, 230, 214, 255 }

local CATS = { "Spells", "Job Abilities", "Weapon Skills", "Pet Commands", "Items", "Status" }

-- The state: category, search, page, the tiles found, the hovered tile, the icon chosen now, the handler.
local P = { cat = 1, q = "", page = 1, found = {}, hover = nil, current = nil, on_pick = nil, defined = false }

local cache = {}

-- A category's icons: { {kind, id, name} } by name.
local function all_of(cat)
  if cat ~= 5 and cache[cat] then return cache[cat] end
  local out, seen = {}, {}
  local function add(kind, id, name)
    if name and name ~= "" and not seen[kind .. ":" .. name] then
      seen[kind .. ":" .. name] = true
      out[#out + 1] = { kind = kind, id = id, name = name }
    end
  end
  if cat == 1 then
    for id = 1, 1023 do
      local s = res.spell(id)
      add("spell", id, s and s.name)
    end
  elseif cat >= 2 and cat <= 4 then
    local from, to = ({ 0x200, 0, 0x400 })[cat - 1], ({ 0x3ff, 0x1ff, 0x7ff })[cat - 1]
    for id = from, to do
      local a = res.ability(id)
      add("ability", id, a and a.name)
    end
  elseif cat == 5 then
    -- What is held: the inventory, the temporary items and the wardrobes.
    for _, bag in ipairs({ 0, 3, 8, 10, 11, 12, 13, 14, 15, 16 }) do
      for _, it in ipairs(game.items(bag)) do
        local r = it.id and res.item(it.id)
        add("item", it.id, r and r.name)
      end
    end
  else
    for id = 0, 1023 do
      add("status", id, res.status(id))
    end
  end
  table.sort(out, function(a, b) return a.name < b.name end)
  if cat ~= 5 then cache[cat] = out end
  return out
end

local function short(text)
  local n = math.max(1, math.floor((T - 6) / (6.5 * 0.72)))
  if utf8.len(text) and utf8.len(text) > n then
    return text:sub(1, utf8.offset(text, n) - 1) .. "..."
  end
  return text
end

local function pages() return math.max(1, math.ceil(#P.found / PER)) end

local function tile_xy(k)
  return ((k - 1) % COLS) * (T + TG), ((k - 1) // COLS) * (T + TG)
end

-- The tile under (x, y) on this page: its place in P.found.
local function tile_at(x, y)
  local first = (P.page - 1) * PER
  for k = 1, math.min(PER, #P.found - first) do
    local tx, ty = tile_xy(k)
    if x >= tx and x < tx + T and y >= ty and y < ty + T then return first + k end
  end
end

local function draw()
  local d = {}
  local first = (P.page - 1) * PER
  for k = 1, math.min(PER, #P.found - first) do
    local f = P.found[first + k]
    local x, y = tile_xy(k)
    local sel = P.current and P.current.kind == f.kind and P.current.id == f.id
    d[#d + 1] = { type = "rect", x = x, y = y, w = T, h = T, round = 7, colour = P.hover == first + k and TILE_HOVER or TILE }
    d[#d + 1] = { type = "rect", x = x, y = y, w = T, h = T, round = 7, line = sel and 2.5 or 1.5,
      colour = sel and TILE_SEL or TILE_EDGE }
    local isz = math.floor(T * 0.58)
    d[#d + 1] = { type = "icon", x = x + (T - isz) / 2, y = y + 4, size = isz, kind = f.kind, id = f.id }
    d[#d + 1] = { type = "text", x = x + T / 2, y = y + T - 14, text = short(f.name), size = 0.72, align = "center",
      colour = TILE_TEXT }
    d[#d + 1] = { type = "tip", x = x, y = y, w = T, h = T, title = f.name, text = "Click to use this icon." }
  end
  local rows = math.max(1, math.ceil(math.min(PER, #P.found - first) / COLS))
  ui.update(WIN, "grid", { h = rows * (T + TG) - TG, draw = d })
  ui.update(WIN, "count", { text = #P.found == 0 and "Nothing matches."
    or ("%d icons; page %d of %d."):format(#P.found, P.page, pages()) })
end

local function refresh()
  local q = P.q:lower():gsub("^%s+", ""):gsub("%s+$", "")
  P.found = {}
  for _, f in ipairs(all_of(P.cat)) do
    if q == "" or f.name:lower():find(q, 1, true) then P.found[#P.found + 1] = f end
  end
  P.page = math.min(P.page, pages())
  P.hover = nil
  draw()
end

local function close()
  ui.close(WIN)
  P.on_pick = nil
end

local function choose(icon)
  local on_pick = P.on_pick
  close()
  if on_pick then on_pick(icon) end
end

local function define()
  ui.window(WIN, {
    title = "Choose an Icon",
    width = CW + 64,
    modal = true,
    help = "Click an icon for the macro. No Icon: the one of the action it does.",
    elements = {
      { type = "choice", id = "cat", label = "Icons", items = CATS, at = P.cat },
      { type = "input", id = "q", label = "Search", text = "", max = 32, tip = "Part of a name, e.g. cure." },
      { type = "canvas", id = "grid", w = CW, h = T },
      { type = "text", id = "count", text = "", dim = true },
      { type = "buttons", id = "pbtn", items = { "Previous", "Next", "No Icon", "Close" } },
    },
  })
  ui.on(WIN, function(_, e)
    if e.id == "grid" then
      if e.kind == "hover" or e.kind == "leave" then
        local k = e.kind == "hover" and tile_at(e.x, e.y) or nil
        if k ~= P.hover then
          P.hover = k
          draw()
        end
      elseif e.kind == "click" then
        local k = tile_at(e.x, e.y)
        if k then choose({ kind = P.found[k].kind, id = P.found[k].id }) end
      end
    elseif e.kind == "set" and e.id == "cat" then
      P.cat, P.page = e.index, 1
      refresh()
    elseif e.kind == "edit" and e.id == "q" then
      P.q, P.page = e.text or "", 1
      refresh()
    elseif e.kind == "press" and e.id == "pbtn" then
      if e.index == 1 then
        P.page = P.page > 1 and P.page - 1 or pages()
        draw()
      elseif e.index == 2 then
        P.page = P.page < pages() and P.page + 1 or 1
        draw()
      elseif e.index == 3 then
        choose(false)
      else
        close()
      end
    elseif e.kind == "close" then
      P.on_pick = nil
    end
  end)
  P.defined = true
end

-- Opens the picker; `current` the icon chosen now ({kind, id} or nil), `on_pick(icon)` gets the one clicked, or
-- false for No Icon.
function M.open(current, on_pick)
  if not P.defined then define() end
  P.current, P.on_pick = current, on_pick
  -- Opened on the current icon's category.
  if current then
    local by_kind = { spell = 1, item = 5, status = 6 }
    P.cat = by_kind[current.kind] or (current.kind == "ability" and (current.id < 0x200 and 3 or current.id >= 0x400 and 4 or 2))
      or P.cat
    ui.update(WIN, "cat", { at = P.cat })
  end
  refresh()
  for k, f in ipairs(P.found) do
    if current and f.kind == current.kind and f.id == current.id then
      P.page = math.ceil(k / PER)
      draw()
      break
    end
  end
  ui.open(WIN)
end

function M.is_open() return P.defined and ui.is_open(WIN) end

return M
