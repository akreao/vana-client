-- The macro manager: Library (the job's macros and the shared ones as tiles on two pages: new, edit, delete, copy from
-- another job, drag onto slots), Help (the mode in use), and the Builder (New / Edit, in the
-- window's place until Save or Cancel).

local data = require("src.data")
local jobs = require("src.jobs")
local keys = require("src.keys")
local actions = require("src.actions")
local builder = require("src.builder")
local iconpick = require("src.iconpick")
local tooltip = require("src.tooltip")
local tiledrag = require("src.tiledrag")
local ghost = require("src.ghost")
local mode = require("src.mode")
local overlay = require("src.overlay")
local bar = require("src.bar")
local pad = require("src.pad")

local M = {}

-- Set by init.lua: changed(), rebind().
M.hooks = {}

local WIN = "manager"
local TAB_LIB, TAB_HELP = 1, 2

local S = { mj = nil }
local defined = false
local DROP, building = "dropdown", false -- "choice" on clients without dropdowns
-- A macro's lines: a text area, or (clients without one) a line with | between them; SEP splits and joins them.
local AREA, SEP = "textarea", "\n"
local layout, leave_builder

local open_builder
-- Builder state. id: entry edited (nil: new); kind: action kind (nil: typed lines); pick: {name}; line: the
-- player's line over the built one; opened: {list, name} of an opener shown; group: members ({name, action}).
local E = { id = nil, kind = "ma", pick = nil, target = "t", line = nil, results = {}, opened = nil, auto_name = nil,
  group = nil, sticky = false }
local BUILD_W = 576

local function say(msg)
  chat.add("Hotbars: " .. msg)
end

-- ---------------------------------------------------------------------------
-- Library
-- ---------------------------------------------------------------------------

local function does_text(entry)
  if type(entry.group) == "table" then
    local names = {}
    for k, m in ipairs(entry.group) do names[k] = m.name or "?" end
    return ("a group of %d: %s"):format(#names, table.concat(names, ", "))
  end
  local first = data.command(entry.action)
  local n = #(entry.lines or {})
  if first then
    return n > 0 and ("%s  +%d"):format(first, n) or first
  end
  return n == 1 and entry.lines[1] or ("%d lines"):format(n)
end

local CW, T, TG = 480, 52, 6 -- canvas width, tile size, gap
local COLS = math.floor((CW + TG) / (T + TG))
local TILE = { 10, 17, 24, 255 }
local TILE_HOVER = { 20, 33, 44, 255 }
local TILE_EDGE = { 184, 139, 73, 205 }
local TILE_SHARED = { 96, 196, 186, 230 }
local TILE_SEL = { 255, 214, 120, 255 }
local TILE_TEXT = { 236, 230, 214, 255 }
local GHOST_BG = { 20, 24, 38, 160 }
local GHOST_INK = { 255, 255, 255, 170 }

-- The Library's two pages, each a canvas of tiles {id, entry, act, label, tip, shared}: the job's macros ("tiles")
-- and the shared ones ("shared"). L.page: the page shown (1 the job's, 2 shared); L.sel: the chosen id;
-- L.hover: {c, n}; L.drag: {c, n, x, y} (x nil outside its canvas).
local PAGE_JOB, PAGE_SHARED = 1, 2
local CANVAS = { [PAGE_JOB] = "tiles", [PAGE_SHARED] = "shared" }
local lists = { tiles = {}, shared = {}, copy = {} }
local L = { sel = nil, hover = nil, drag = nil, page = PAGE_JOB }
-- Copy From: the job whose macros show under the job's page ("copy" canvas) to copy from, or nil.
-- dragged: the tile (its place in lists.copy) being dragged; over: {x, y} where it is over the job's tiles.
local C = { job = nil, dragged = nil, over = nil }

local function shown() return lists[CANVAS[L.page]] end

local function tile_xy(n)
  return ((n - 1) % COLS) * (T + TG), ((n - 1) // COLS) * (T + TG)
end

local function tile_at(c, x, y)
  for n = 1, #lists[c] do
    local tx, ty = tile_xy(n)
    if x >= tx and x < tx + T and y >= ty and y < ty + T then return n end
  end
end

-- Whether (x, y) is on canvas c's tiles area.
local function on_canvas(c, x, y)
  return x >= 0 and y >= 0 and x < CW and y < math.max(1, math.ceil(#lists[c] / COLS)) * (T + TG)
end

local function short(text)
  local n = math.max(1, math.floor((T - 6) / (6.5 * 0.72)))
  if utf8.len(text) and utf8.len(text) > n then
    return text:sub(1, utf8.offset(text, n) - 1) .. "..."
  end
  return text
end

local function draw_canvas(c)
  local d = {}
  local function add(e) d[#d + 1] = e end
  local tiles = lists[c]
  for n, t in ipairs(tiles) do
    local x, y = tile_xy(n)
    local hover = L.hover and L.hover.c == c and L.hover.n == n
    add({ type = "rect", x = x, y = y, w = T, h = T, round = 7, colour = hover and TILE_HOVER or TILE })
    local sel = c ~= "copy" and L.sel == t.id
    add({ type = "rect", x = x, y = y, w = T, h = T, round = 7, line = sel and 2.5 or 1.5,
      colour = sel and TILE_SEL or t.shared and TILE_SHARED or TILE_EDGE })
    if t.act then
      local isz = math.floor(T * 0.58)
      add({ type = "icon", x = x + (T - isz) / 2, y = y + 4, size = isz, kind = t.act.kind, id = t.act.id })
      add({ type = "text", x = x + T / 2, y = y + T - 14, text = short(t.label:match("^[^\n]*")), size = 0.72,
        align = "center", colour = TILE_TEXT })
    else
      local ls = {}
      for l in t.label:gmatch("[^\n]+") do ls[#ls + 1] = l end
      local top = y + (T - #ls * 12) / 2
      for k, l in ipairs(ls) do
        add({ type = "text", x = x + T / 2, y = top + (k - 1) * 12, text = short(l), size = 0.72, align = "center",
          colour = TILE_TEXT })
      end
    end
    if t.have or L.drag and L.drag.c == c and L.drag.n == n then
      add({ type = "rect", x = x, y = y, w = T, h = T, round = 7, colour = { 0, 0, 0, 120 } })
    end
    -- The same tooltip as the bars' and the Builder's (tooltip.macro).
    add({ type = "tip", x = x, y = y, w = T, h = T, title = t.title or t.entry.name or "", icon = t.act or nil,
      text = t.tip })
  end
  local g = L.drag and L.drag.c == c and L.drag.x and tiles[L.drag.n]
  local gx, gy = g and L.drag.x - T / 2, g and L.drag.y - T / 2
  -- Another job's tile dragged over this job's: its ghost here.
  if c == "tiles" and C.over and C.dragged and lists.copy[C.dragged] and not ghost.top then
    g, gx, gy = lists.copy[C.dragged], C.over.x - T / 2, C.over.y - T / 2
  end
  if g then
    add({ type = "rect", x = gx, y = gy, w = T, h = T, round = 7, colour = GHOST_BG })
    if g.act then
      local isz = math.floor(T * 0.58)
      add({ type = "icon", x = gx + (T - isz) / 2, y = gy + 4, size = isz, kind = g.act.kind, id = g.act.id,
        colour = GHOST_INK })
    else
      add({ type = "text", x = gx + T / 2, y = gy + T / 2 - 6, text = short(g.label:match("^[^\n]*")), size = 0.72,
        align = "center", colour = GHOST_INK })
    end
  end
  local rows = math.max(1, math.ceil(#tiles / COLS))
  ui.update(WIN, c, { h = rows * (T + TG) - TG, draw = d })
end

local function draw_tiles()
  draw_canvas("tiles")
  draw_canvas("shared")
  if C.job then draw_canvas("copy") end
end

-- The Copy From part under the job's page: the other job's macros as tiles (dimmed when this job has the same), or
-- hidden when none is chosen.
local function refresh_copy()
  local from = C.job
  for _, id in ipairs({ "cctx", "copy", "cbtn" }) do ui.update(WIN, id, { hidden = not from }) end
  lists.copy = {}
  if not from or not S.mj then return end
  for _, it in ipairs(data.own_library(from)) do
    local macro = data.macro_of(from, it.id)
    local tt = macro and tooltip.macro(macro)
    lists.copy[#lists.copy + 1] = { id = it.id, entry = it.entry, tip = tt and tt.text or "", title = tt and tt.title or nil,
      act = macro and actions.look(macro), label = data.display_label(it.entry), have = data.has_same(S.mj, it.entry) }
  end
  ui.update(WIN, "cctx", { text = ("Copy from %s: click a macro to copy it to %s's library."):format(from, S.mj) })
  draw_canvas("copy")
end

local function show_selected()
  local t
  for _, x in ipairs(shown()) do
    if x.id == L.sel then t = x end
  end
  local text = t and ('%s: %s; on %d slot%s.'):format(t.entry.name or "?", does_text(t.entry),
    data.uses(S.mj, t.id), data.uses(S.mj, t.id) == 1 and "" or "s") or "Click a macro to choose it."
  ui.update(WIN, "lsel", { text = text })
  ui.update(WIN, "ssel", { text = text })
end

-- The chosen macro on the page shown: kept when it is there, else its first.
local function settle_selection()
  local list = shown()
  for _, t in ipairs(list) do
    if t.id == L.sel then return end
  end
  L.sel = list[1] and list[1].id or nil
end

local function refresh_library(select_id)
  lists.tiles, lists.shared = {}, {}
  if not S.mj then
    ui.update(WIN, "lctx", { text = "Jobs are not loaded yet." })
    draw_tiles()
    return
  end
  ui.update(WIN, "ltabs", { items = { S.mj .. " Macros", "Shared Macros" } })
  for _, it in ipairs(data.library(S.mj)) do
    local macro = data.macro_of(S.mj, it.id)
    local list = it.sticky and lists.shared or lists.tiles
    local tt = macro and tooltip.macro(macro)
    list[#list + 1] = { id = it.id, entry = it.entry, tip = tt and tt.text or "", title = tt and tt.title or nil,
      act = macro and actions.look(macro), label = data.display_label(it.entry), shared = it.sticky }
  end
  -- A macro asked for (just saved or copied) turns to its page.
  if select_id then
    L.sel = tostring(select_id)
    local page = data.is_sticky(L.sel) and PAGE_SHARED or PAGE_JOB
    if page ~= L.page then
      L.page = page
      ui.update(WIN, "ltabs", { at = page })
    end
  end
  settle_selection()
  ui.update(WIN, "lctx", {
    text = #lists.tiles == 0 and ("%s has no macros yet: press New."):format(S.mj)
      or ("%s macros: drag one onto a hotbar or crossbar slot."):format(S.mj),
  })
  ui.update(WIN, "sctx", {
    text = #lists.shared == 0 and "No shared macros yet: press New (shared in every job's library)."
      or "In every job's library, for every character: drag one onto a slot.",
  })
  draw_tiles()
  show_selected()
  refresh_copy()
end

-- Copies `ids` of the Copy From job's macros (all when nil) into this job's library, and says what it did.
local function copy_these(ids)
  local from, here = C.job, S.mj
  if not from or not here then return end
  local copied, skipped, why = data.copy_entries(from, here, ids)
  if why then
    say("the copies could not be saved: " .. why)
  elseif copied == 0 then
    say(("%s already has %s."):format(here, ids and "that macro" or "all of them"))
  else
    say(("copied %d macro%s from %s%s."):format(copied, copied == 1 and "" or "s", from,
      skipped > 0 and ("; %d already here"):format(skipped) or ""))
  end
  refresh_library()
  M.hooks.changed()
end

-- Hover, click to choose, right-click to edit, drag onto slots.
-- Whether a drag's pointer is over a window (the HUD under it hidden): the client says (covered); a client before
-- that, only our tiles and our other windows' canvases.
local function covered(e, inside)
  if e.covered ~= nil then return e.covered end
  return inside or e.window ~= nil
end

local function on_tiles(e)
  local c = e.id
  local tiles = lists[c]
  -- The other job's tiles: a click copies one, and so does a drop on this job's tiles (modern UI); they do not go
  -- onto the bars (those are this job's).
  if c == "copy" then
    if e.kind == "click" then
      local n = tile_at(c, e.x, e.y)
      if n then copy_these({ tiles[n].id }) end
      return
    elseif e.kind == "drag_start" then
      local n = tile_at(c, e.x, e.y)
      if not n then return end
      C.dragged, L.drag = n, { c = c, n = n, x = e.x, y = e.y }
      local sx, sy = overlay.screen_xy(e)
      if sx and sy then ghost.start(sx, sy, math.floor(T * hud.screen().ui_scale), tiles[n].act, tiles[n].label) end
    elseif e.kind == "drag" and L.drag then
      ghost.top = e.covered ~= nil
      L.drag.x, L.drag.y = on_canvas(c, e.x, e.y) and not ghost.top and e.x or nil, e.y
      local sx, sy = overlay.screen_xy(e)
      if sx and ghost.top then ghost.move(sx, sy) elseif sx then ghost.move(-10000, -10000) end
    elseif e.kind == "drag_end" then
      -- A drop on the job's tiles comes after this.
      L.drag, C.over = nil, nil
      ghost.stop()
      draw_canvas("tiles")
    else
      return
    end
    draw_canvas(c)
    return
  end
  if c == "tiles" and (e.kind == "drag_over" or e.kind == "drag_out" or e.kind == "drop") and e.from == "copy" then
    C.over = e.kind == "drag_over" and { x = e.x, y = e.y } or nil
    local t = e.kind == "drop" and C.dragged and lists.copy[C.dragged]
    C.dragged = e.kind == "drop" and nil or C.dragged
    draw_canvas("tiles")
    if t then copy_these({ t.id }) end
    return
  end
  if e.kind == "hover" or e.kind == "leave" then
    local n = e.kind == "hover" and tile_at(c, e.x, e.y) or nil
    local was = L.hover
    if (was and was.c == c and was.n or nil) ~= n then
      L.hover = n and { c = c, n = n } or nil
      draw_canvas(c)
    end
  elseif e.kind == "click" then
    local n = tile_at(c, e.x, e.y)
    if not n then return end
    L.sel = tiles[n].id
    draw_canvas(c)
    show_selected()
    if e.button == "right" then open_builder(tiles[n].id) end
  elseif e.kind == "drag_start" then
    local n = tile_at(c, e.x, e.y)
    if not n then return end
    local t = tiles[n]
    L.drag = { c = c, n = n, x = e.x, y = e.y }
    if S.mj == jobs.current() then
      tiledrag.start(t.id, t.act, t.label)
    else
      tiledrag.last = nil
      say(("these are %s's macros; the bars show %s's."):format(S.mj, jobs.current() or "?"))
    end
    local sx, sy = overlay.screen_xy(e)
    if sx and sy then ghost.start(sx, sy, math.floor(settings.get("size") * hud.screen().ui_scale), t.act, t.label) end
    draw_canvas(c)
  elseif e.kind == "drag" and L.drag then
    -- The HUD ghost: over every window where the client draws it so (`covered` comes with that), else only outside
    -- our windows, which draw their own copy.
    ghost.top = e.covered ~= nil
    tiledrag.top = ghost.top
    local inside = on_canvas(c, e.x, e.y)
    L.drag.x, L.drag.y = inside and not ghost.top and e.x or nil, e.y
    local sx, sy = overlay.screen_xy(e)
    local hidden = covered(e, inside)
    if sx then
      if hidden and not ghost.top then ghost.move(-10000, -10000) else ghost.move(sx, sy) end
    end
    if overlay.on() and tiledrag.dragging and sx and sy then
      local key = not hidden and overlay.hover_at(sx, sy) or nil
      if mode.is_pad() then pad.hover_overlay(key) else bar.hover_overlay(key) end
    end
    draw_canvas(c)
  elseif e.kind == "drag_end" and L.drag then
    local id = tiledrag.dragging and tiledrag.dragging.id or tiledrag.last
    local inside = on_canvas(c, e.x, e.y)
    local sx, sy = overlay.screen_xy(e)
    if overlay.on() and id and sx and sy and not covered(e, inside) then
      overlay.drop_at(sx, sy, id)
    end
    L.drag = nil
    ghost.stop()
    tiledrag.stop()
    draw_canvas(c)
  end
end

-- Another job's macros into this job's library: its name from a list, then all of them or one.
local function copy_from()
  local here = S.mj
  if not here then return end
  local others = {}
  for _, j in ipairs(data.jobs_with_macros()) do
    if j.job ~= here then others[#others + 1] = j end
  end
  if #others == 0 then
    say("no other job has macros to copy yet.")
    return
  end
  local rows = {}
  for i, j in ipairs(others) do rows[i] = ("%s (%d macro%s)"):format(j.job, j.n, j.n == 1 and "" or "s") end
  ui.menu(rows, function(i)
    local from = i and others[i]
    if not from then return end
    C.job = from.job
    refresh_copy()
  end)
end

local function delete_selected()
  local id = L.sel
  local entry = id and data.get_entry(S.mj, id)
  if not entry then return end
  local mj = S.mj
  local n = data.uses(mj, id)
  local what = data.is_sticky(id) and ('the shared macro "%s" from every job'):format(entry.name)
    or ('"%s"'):format(entry.name)
  local text = n > 0 and ('Delete %s? It is on %d slot%s, which will be emptied.'):format(what, n,
    n == 1 and "" or "s") or ('Delete %s?'):format(what)
  -- At the pointer, by the Delete it answers (clients before that centre it).
  ui.prompt(text, { "Delete", "Cancel" }, function(i)
    if i == 1 then
      local ok, why = data.delete_entry(mj, id)
      if not ok then say(('deleting "%s" could not be saved: %s'):format(entry.name, why or "?")) end
      M.hooks.changed()
      refresh_library()
    end
  end, "pointer")
end

-- ---------------------------------------------------------------------------
-- Builder
-- ---------------------------------------------------------------------------

local function type_def()
  return builder.TYPES[builder.type_index(E.kind)]
end

-- What Save keeps: a group, one action (run as it is, no lines), or a macro's typed lines.
local function builder_entry()
  local name = ui.get(WIN, "name").text
  -- The icon picked for it (none: its action's shows).
  local icon = E.icon
  if E.group then return { name = name, group = E.group, icon = icon } end
  if E.kind then
    return { name = name, icon = icon,
      action = E.pick and { kind = E.kind, name = E.pick.name, target = E.target } or nil }
  end
  return { name = name, icon = icon, lines = data.split_lines(ui.get(WIN, "extra").text, SEP) }
end

-- The Builder's icon tile, beside the name: the icon picked, else the pick's (E.auto_icon), else blank.
local function draw_bicon()
  local look = E.icon or E.auto_icon
  local d = {
    { type = "rect", x = 0, y = 0, w = T, h = T, round = 7, colour = E.icon_hover and TILE_HOVER or TILE },
    { type = "rect", x = 0, y = 0, w = T, h = T, round = 7, line = 1.5, colour = TILE_EDGE },
    { type = "tip", x = 0, y = 0, w = T, h = T, title = "Icon",
      text = E.icon and "Picked for this macro. Click to choose another."
        or "Click to choose the macro's icon (blank: its action's, once you pick one)." },
  }
  if look then
    local isz = math.floor(T * 0.7)
    table.insert(d, 3, { type = "icon", x = (T - isz) / 2, y = (T - isz) / 2, size = isz, kind = look.kind, id = look.id })
  end
  ui.update(WIN, "bicon", { draw = d })
end

local function pick_icon()
  iconpick.open(E.icon or E.auto_icon, function(icon)
    E.icon = icon or nil
    draw_bicon()
  end)
end

local function refresh_bbar()
  ui.update(WIN, "bbar", {
    items = { E.sticky and "Shared: On" or "Shared: Off", "Save", "Cancel" },
    marked = false,
  })
end

-- Target choices for the kind and pick, plus the current target if it is not among them.
local function target_choices()
  if not E.kind then return { { "", "(not used)" } } end
  local choices = {}
  for i, c in ipairs(builder.targets_for(E.kind, E.pick and E.pick.name)) do choices[i] = c end
  local held = false
  for _, c in ipairs(choices) do
    if c[1] == E.target then held = true end
  end
  if not held and data.trim(E.target) ~= "" then
    choices[#choices + 1] = { E.target, builder.target_label(E.kind, E.target, E.pick and E.pick.name) }
  end
  return choices
end

local GROUP_TIP = "One slot for all of them: pressing it opens a list over it to pick one (Up / Down, Enter)."
local UNGROUP_TIP = "Undoes the group: the slot holds one action again, picked from the results."

local function refresh_choice()
  ui.update(WIN, "grp", { items = { E.group and "Ungroup These" or "Make a Group of These" },
    tip = E.group and UNGROUP_TIP or GROUP_TIP })
  ui.update(WIN, "btype", { at = builder.type_index(E.kind) })
  local labels, at = {}, 1
  for i, c in ipairs(target_choices()) do
    labels[i] = c[2]
    if c[1] == (E.target or "") then at = i end
  end
  ui.update(WIN, "target", { label = E.kind == "equip" and "Equipment slot" or "Target", items = labels, at = at })
  -- A macro is its typed lines alone: no search, list, group or Target; an action has no lines.
  for _, id in ipairs({ "q", "found", "fcount", "grp", "target" }) do
    ui.update(WIN, id, { hidden = not E.kind })
  end
  ui.update(WIN, "extra", { hidden = E.kind ~= nil })
end

local function refresh_results(query)
  local t = type_def()
  ui.update(WIN, "q", { label = t.search })
  ui.update(WIN, "found", { columns = builder.columns(E.kind) })
  if not E.kind then
    E.results = {}
    ui.update(WIN, "found", { rows = {} })
    ui.update(WIN, "fcount", { text = "" })
    return
  end
  -- A group with no search lists its members; a click removes one.
  if E.group and data.trim(query) == "" and not E.opened then
    E.results = {}
    local rows = {}
    for k, m in ipairs(E.group) do
      local a = actions.of({ lines = { data.command(m.action) or "" }, action = m.action })
      E.results[k] = { member = k }
      local icon = a and { kind = a.kind, id = a.id } or nil
      rows[k] = { m.name, "", "", icon = icon, hover = tooltip.action(icon, m.name) }
    end
    ui.update(WIN, "found", { rows = rows, at = 1 })
    ui.update(WIN, "fcount", { text = #rows == 0 and "No members yet: search, then click one to add it."
      or "Its members: click one to take it out. Search to add more." })
    return
  end
  local found, more = builder.find(E.kind, query, E.opened and E.opened.list)
  E.results = {}
  local rows = {}
  if E.opened then
    E.results[1] = { back = true }
    rows[1] = { "\u{2039} Back", "", "", colour = { 200, 180, 130 } }
  end
  for _, e in ipairs(found) do
    E.results[#E.results + 1] = e
    local name = e.list and (e.name .. "  \u{203a}") or e.name
    rows[#rows + 1] = { name, e.cells[1] or "", e.cells[2] or "", icon = e.icon,
      colour = e.list and { 232, 210, 156 } or nil, tip = e.tip, hover = tooltip.action(e.icon, e.title or e.name, e.tip) }
  end
  ui.update(WIN, "found", { rows = rows, at = 1 })
  local count = #found == 0 and "Nothing matches." or (more and ("First %d matches."):format(#found)
    or ("%d found."):format(#found))
  if E.opened then
    count = ("%s: %s Back returns to the list."):format(E.opened.name, count)
  end
  if E.group then
    count = count .. " Click one to add it to the group."
  end
  ui.update(WIN, "fcount", { text = count })
end

local function set_kind(kind)
  if kind == E.kind then return end
  -- A group keeps its members across types.
  E.kind, E.pick, E.opened, E.auto_icon = kind, nil, nil, nil
  E.target = kind == "equip" and "main" or kind == "cmd" and "" or "t"
  draw_bicon()
  refresh_results(ui.get(WIN, "q").text)
  refresh_choice()
end

-- On library entry `id`, or a new macro.
-- `shared`: a new macro starts shared (New on the Shared Macros page).
function open_builder(id, shared)
  if not S.mj then
    say("jobs are not loaded yet.")
    return
  end
  local entry = id and data.get_entry(S.mj, id) or nil
  E.id = entry and tostring(id) or nil
  E.sticky = E.id ~= nil and data.is_sticky(E.id) or (E.id == nil and shared == true)
  E.kind, E.pick, E.target, E.auto_name, E.opened, E.group = "ma", nil, "t", nil, nil, nil
  local name, extra = "", ""
  if entry then
    name = entry.name or ""
    -- An action with lines after it, or a line of its own, opens as a macro of all its lines: nothing is lost.
    local lined = type(entry.action) == "table" and (#(entry.lines or {}) > 0 or data.trim(entry.action.line) ~= "")
    extra = data.join_lines(lined and data.entry_lines(entry) or entry.lines, SEP)
    if lined then
      E.kind = nil
    elseif type(entry.group) == "table" then
      E.group = {}
      for k, m in ipairs(entry.group) do
        E.group[k] = { name = m.name, action = m.action and { kind = m.action.kind, name = m.action.name,
          target = m.action.target, line = m.action.line }, lines = m.lines }
      end
      local first = E.group[1] and E.group[1].action
      E.kind = first and first.kind or "ja"
      E.target = first and first.target or "t"
    elseif entry.action then
      E.kind = entry.action.kind
      if E.kind == "ma" and builder.is_trust(entry.action.name) then E.kind = "trust" end
      E.pick = { name = entry.action.name }
      E.target = entry.action.target
    else
      E.kind = nil
    end
  end
  layout(true)
  -- Its icon: the one picked for it, else (shown, not kept) its action's.
  E.icon = entry and type(entry.icon) == "table" and { kind = entry.icon.kind, id = entry.icon.id } or nil
  local m = entry and data.macro_of(S.mj, id)
  local a = m and actions.of(m)
  E.auto_icon = a and { kind = a.kind, id = a.id } or nil
  draw_bicon()
  ui.update(WIN, "name", { text = name })
  ui.update(WIN, "extra", { text = extra })
  ui.update(WIN, "q", { text = "" })
  refresh_bbar()
  refresh_results("")
  refresh_choice()
end

-- Into an opener's list (`e`), or back out (nil).
local function open_list(e)
  E.opened = e and { list = e.list, name = e.name } or nil
  ui.update(WIN, "q", { text = "" })
  refresh_results("")
end

local function member_of(e)
  return { name = e.title or e.name, action = { kind = E.kind, name = e.name, target = e.target or "" } }
end

local function pick(index)
  local e = E.results[index]
  if not e then return end
  if e.member then
    table.remove(E.group, e.member)
    refresh_results("")
    refresh_choice()
    return
  end
  if e.back or e.list then
    open_list(not e.back and e or nil)
    return
  end
  if E.group then
    for _, m in ipairs(E.group) do
      if m.action and m.action.kind == E.kind and m.action.name == e.name then
        say(e.name .. " is in the group already.")
        return
      end
    end
    E.group[#E.group + 1] = member_of(e)
    refresh_choice()
    say(("%s added (%d in the group)."):format(e.name, #E.group))
    return
  end
  local name = data.trim(ui.get(WIN, "name").text)
  -- The name follows the pick until typed over.
  if name == "" or name == E.auto_name then
    ui.update(WIN, "name", { text = e.title or e.name })
    E.auto_name = e.title or e.name
  end
  if not E.pick or E.pick.name ~= e.name then
    E.target = e.target
  end
  E.pick = { name = e.name }
  -- The icon follows the pick until one is picked for it.
  E.auto_icon = e.icon and { kind = e.icon.kind, id = e.icon.id } or nil
  draw_bicon()
  refresh_choice()
end

local function set_target(index)
  local c = target_choices()[index]
  if not c or not E.kind then return end
  E.target = c[1]
  -- Members that target only you keep <me>.
  for _, m in ipairs(E.group or {}) do
    if m.action and m.action.kind ~= "equip" and m.action.kind ~= "cmd"
      and not builder.self_only(m.action.kind, m.action.name) then
      m.action.target = c[1]
    end
  end
  refresh_choice()
end

-- Makes the results a group, or a group back into one action.
local function toggle_group()
  if E.group then
    E.group = nil
    refresh_results(ui.get(WIN, "q").text)
    refresh_choice()
    return
  end
  if not E.kind or E.kind == "cmd" then
    say("search spells, abilities or items first, then make a group of what is found.")
    return
  end
  local members = {}
  for _, e in ipairs(E.results) do
    if not e.back and not e.list and not e.member then members[#members + 1] = member_of(e) end
  end
  if #members == 0 then
    say("nothing to group: search first, or open a list such as Rune Enchantment.")
    return
  end
  E.group, E.pick = members, nil
  E.target = members[1].action.target
  local name = data.trim(ui.get(WIN, "name").text)
  if name == "" or name == E.auto_name then
    local q = data.trim(ui.get(WIN, "q").text)
    local auto = E.opened and E.opened.name or (q ~= "" and (q:sub(1, 1):upper() .. q:sub(2))) or members[1].name
    ui.update(WIN, "name", { text = auto })
    E.auto_name = auto
  end
  E.opened = nil
  ui.update(WIN, "q", { text = "" })
  refresh_results("")
  refresh_choice()
end

local function save_builder()
  if not S.mj then
    return
  end
  local entry = builder_entry()
  if entry.group then
    if #entry.group == 0 then
      say("the group has no members: search, then click to add.")
      return
    end
    if data.trim(entry.name) == "" then entry.name = entry.group[1].name end
  elseif E.kind and not entry.action then
    say("pick one from the list first.")
    return
  end
  if not entry.group and not entry.action and #entry.lines == 0 then
    say("add at least one line to the macro.")
    return
  end
  if data.trim(entry.name) == "" and not entry.group then
    local c = entry.action and entry.action.kind == "cmd" and builder.command(entry.action.name)
    entry.name = c and c.title or entry.action and entry.action.name or entry.lines[1]:sub(1, 24)
  end
  local id, why = data.save_entry(S.mj, E.id, entry, E.sticky)
  if why then
    -- A new shared macro was not made: the builder stays open with it. An edit is kept where the macro was.
    say((id and "the macro was saved, but could not be moved: " or "the macro could not be saved: ") .. why)
    if not id then return end
  end
  M.hooks.changed()
  leave_builder(id)
end

-- ---------------------------------------------------------------------------
-- Help
-- ---------------------------------------------------------------------------

local HEAD = { 232, 210, 156 }
local INK = { 236, 230, 214 }

-- Per mode; a table is a heading.
local HELP = {
  [mode.HOTBARS] = {
    { "Keyboard hotbars" },
    "Each bar's slots fire with their keys. Right-click the bars, Assign Keys:",
    "point at a slot and press its key; Esc ends it.",
    "or with a click. A group slot opens a list: Up / Down, then Enter.",
    { "Putting macros on the bars" },
    "Library tab: New builds a macro for the job; drag a tile onto a slot.",
    "Copy From brings another job's macros into this job's library.",
    "Drag a slot onto another to swap them. Right-click a slot for its menu.",
    "A shared macro (Builder: Shared) shows in every job's library.",
    { "More" },
    "//hb show, hide, more, fewer: the bars; //hb mode: the gamepad's.",
    "The Mods page: bar count, slots, size, key hints, Individual Config.",
  },
  [mode.GAMEPAD] = {
    { "Gamepad crossbars" },
    "Hold LT, RT or both, then press a d-pad direction, A / B / X / Y,",
    "L1 / R1 or L3 / R3: three sets of twelve slots.",
    "A group slot opens a list: d-pad Up / Down, A runs it, B closes it.",
    "Trigger + Start opens this manager.",
    { "Assigning with the controller" },
    "Trigger + Back (or //hb assign) starts assigning: all sets show.",
    "Trigger + a slot's button opens the library: A puts it on the slot,",
    "its Clear row empties it. Back, or B with no trigger, ends it.",
    { "With the mouse" },
    "Drag a Library or Palette tile onto a crossbar slot.",
    "Library: Copy From brings another job's macros into this one's.",
    { "More" },
    "//hb mode: the keyboard's. The Mods page: Auto-hide, Detect Gamepad.",
  },
}
local HELP_LINES = 14

local function refresh_help()
  local lines = HELP[mode.get()] or HELP[mode.HOTBARS]
  for i = 1, HELP_LINES do
    local l = lines[i]
    local head = type(l) == "table"
    ui.update(WIN, "help" .. i, { text = (head and l[1] or l) or "", colour = head and HEAD or INK })
  end
end

-- ---------------------------------------------------------------------------
-- Window
-- ---------------------------------------------------------------------------

-- The Builder (`build`) or the tabs. `drop`: Type and Target's element type.
local function spec(drop, build, area)
  if build then
    return {
      title = "Hotbars",
      width = BUILD_W,
      help = "Shared: every job and character. Save keeps the macro; Cancel goes back to the library.",
      elements = {
        { type = drop, id = "btype", label = "Type", items = builder.TYPE_LABELS, at = 1,
          tip = "Choose what the macro does." },
        { type = "canvas", id = "bicon", w = T, h = T, float = "right",
          tip = "The macro's icon: click to choose one." },
        { type = "input", id = "name", label = "Name", text = "", max = 32,
          tip = "Filled in from what you pick until you type your own." },
        { type = "input", id = "q", label = "Search", text = "", max = 32, tip = "Part of a name, e.g. cure." },
        { type = "list", id = "found", rows = {}, height = 4, columns = builder.columns("ma"),
          tip = "Enter or click one to build the macro on it." },
        { type = "text", id = "fcount", text = "", dim = true },
        { type = "buttons", id = "grp", items = { "Make a Group of These" }, tip = GROUP_TIP },
        { type = drop, id = "target", label = "Target", items = { "<t> Current target" }, at = 1 },
        area == "textarea"
            and { type = "textarea", id = "extra", label = "Lines", text = "", lines = 6, max = 1024,
              tip = "One command a line, e.g. /p Casting <t>, then /wait 2. Tab ends the typing." }
          or { type = "input", id = "extra", label = "Lines (| between lines)", text = "", max = 1024,
            tip = 'Example: /p Casting <t> | /wait 2 | /echo done' },
        { type = "buttons", id = "bbar", items = { "Shared: Off", "Save", "Cancel" },
          tip = "Shared: Off until you turn it on (every job and character). Save keeps the macro; Cancel goes back." },
      },
    }
  end
  local elements = {
      { type = "tabs", id = "tabs", items = { "Library", "Help" }, at = TAB_LIB },
  }
  for i = 1, HELP_LINES do
    elements[#elements + 1] = { type = "text", id = "help" .. i, text = "", page = TAB_HELP }
  end
  -- Library, last: its own tabs (the job's macros, the shared ones) are on the Library page, and the elements after
  -- them follow them (page = that tab), hidden with them on the other pages.
  local library = {
    { type = "tabs", id = "ltabs", items = { (S.mj or "Job") .. " Macros", "Shared Macros" }, at = L.page,
      page = TAB_LIB },
    { type = "text", id = "lctx", text = "", page = PAGE_JOB },
    { type = "canvas", id = "tiles", w = CW, h = T, page = PAGE_JOB },
    { type = "text", id = "lsel", text = "", dim = true, page = PAGE_JOB },
    { type = "buttons", id = "lbtn", items = { "New", "Edit", "Delete", "Copy From" }, page = PAGE_JOB,
      tip = "Copy From: another job's macros into this job's library." },
    { type = "text", id = "cctx", text = "", page = PAGE_JOB, hidden = not C.job },
    { type = "canvas", id = "copy", w = CW, h = T, page = PAGE_JOB, hidden = not C.job },
    { type = "buttons", id = "cbtn", items = { "Copy All", "Close" }, page = PAGE_JOB, hidden = not C.job,
      tip = "Copy All: every macro shown, but those this job has already. Close: put the list away." },
    { type = "text", id = "sctx", text = "", page = PAGE_SHARED },
    { type = "canvas", id = "shared", w = CW, h = T, page = PAGE_SHARED },
    { type = "text", id = "ssel", text = "", dim = true, page = PAGE_SHARED },
    { type = "buttons", id = "sbtn", items = { "New", "Edit", "Delete" }, page = PAGE_SHARED,
      tip = "New makes a shared macro: in every job's library, for every character." },
  }
  for _, e in ipairs(library) do elements[#elements + 1] = e end
  return {
    title = "Hotbars",
    width = 520,
    help = "Drag a macro onto a slot of the bars. Right-click a slot to clear it.",
    elements = elements,
  }
end

local function define()
  -- Clients 1.0.10 and older have no dropdown.
  DROP = pcall(ui.window, WIN, spec("dropdown", true, "input")) and "dropdown" or "choice"
  -- Clients before the text area have none.
  AREA = pcall(ui.window, WIN, spec(DROP, true, "textarea")) and "textarea" or "input"
  SEP = AREA == "textarea" and "\n" or "|"
  ui.window(WIN, spec(DROP, false, AREA))
  building = false

  ui.on(WIN, function(_, e)
    if e.id == "tiles" or e.id == "shared" or e.id == "copy" then
      on_tiles(e)
    elseif e.id == "bicon" then
      if e.kind == "click" then
        pick_icon()
      elseif e.kind == "hover" or e.kind == "leave" then
        if E.icon_hover ~= (e.kind == "hover") then
          E.icon_hover = e.kind == "hover"
          draw_bicon()
        end
      end
    elseif e.kind == "set" and e.id == "ltabs" then
      L.page = e.index
      settle_selection()
      draw_tiles()
      show_selected()
    elseif e.kind == "set" then
      if e.id == "btype" then
        set_kind(builder.TYPES[e.index].kind)
      elseif e.id == "target" then
        set_target(e.index)
      end
    elseif e.kind == "edit" then
      if e.id == "q" then
        refresh_results(e.text)
      end
    elseif e.kind == "select" then
      if e.id == "found" then
        pick(e.index)
      end
    elseif e.kind == "press" then
      if e.id == "cbtn" then
        if e.index == 1 then
          copy_these(nil)
        else
          C.job = nil
          refresh_copy()
        end
      elseif e.id == "lbtn" or e.id == "sbtn" then
        if e.index == 1 then
          open_builder(nil, e.id == "sbtn")
        elseif e.index == 2 then
          if L.sel then open_builder(L.sel) end
        elseif e.index == 3 then
          delete_selected()
        elseif e.index == 4 then
          copy_from()
        end
      elseif e.id == "grp" then
        toggle_group()
      elseif e.id == "bbar" then
        if e.index == 1 then
          E.sticky = not E.sticky
          refresh_bbar()
        elseif e.index == 2 then
          save_builder()
        else
          leave_builder()
        end
      end
    end
  end)
  defined = true
end

local function ensure()
  if not defined then
    define()
  end
end

local function refresh_all(select_id)
  refresh_library(select_id)
  refresh_help()
end

function layout(build)
  building = build
  ui.window(WIN, spec(DROP, build, AREA))
end

-- Builder -> Library, choosing `select_id`.
function leave_builder(select_id)
  layout(false)
  refresh_all(select_id)
end

function M.open(select_id)
  ensure()
  C.job = nil
  S.mj = jobs.current() or S.mj
  layout(false)
  refresh_all(select_id)
  ui.open(WIN)
end

function M.is_open()
  return defined and ui.is_open(WIN) or false
end

function M.toggle()
  ensure()
  if ui.is_open(WIN) then
    ui.close(WIN)
  else
    M.open()
  end
end

function M.edit_entry(id)
  ensure()
  S.mj = jobs.current() or S.mj
  open_builder(id)
  ui.open(WIN)
end

-- After outside changes; the Builder keeps its macro.
function M.refresh()
  -- Always the job we are on: a job change while the window is open shows the new job's library.
  S.mj = jobs.current() or S.mj
  if C.job == S.mj then C.job = nil end
  if defined and not building then
    refresh_all()
  end
end

return M
