-- Keyboard hotbars in an edge window. Bar 1 sits on the docked edge; + / - show more or fewer bars (eased). A slot
-- shows its icon or label, hotkey, recast shade and tip; unusable actions dim, battle-menu suggestions glow. Click
-- fires, right-click opens the slot menu, drag swaps, a dropped library tile assigns. Gold edge: main job's set;
-- blue: sub job's.

local data = require("src.data")
local runner = require("src.runner")
local jobs = require("src.jobs")
local keys = require("src.keys")
local actions = require("src.actions")
local chooser = require("src.chooser")
local menu = require("src.menu")
local mode = require("src.mode")
local slotview = require("src.slotview")
local overlay = require("src.overlay")

local M = {}

-- Set by init.lua: changed(), open_manager().
M.hooks = {}

local WIN = "bars"

local KEY, GOLD = slotview.KEY, slotview.GOLD
local EDGE = { main = { 184, 139, 73, 205 }, sub = { 104, 164, 232, 205 } }
local GLOW = { 255, 214, 120 }
local GOLD_OFF = { 228, 192, 119, 70 }
local INK = { 236, 230, 214, 255 }

local SPEED = 4.5 -- bars per second while growing or shrinking
local PAD = 8

local hidden = false   -- //hb show | hide
local built_sig = nil
local reveal = nil     -- bars shown now, eased toward wanted()
local geo = nil        -- size, gap, slots, bars, width, btn
local eff, main, sub = nil, nil, nil
local st = slotview.state()

-- Set below; named here so everything above their definitions calls them, not globals.
local draw
local open_menu

-- Assign Keys (below): on, the slot chosen ("b:i"), its key filter, a collision question open.
local A = { on = false, sel = nil, filter = nil, asking = false }
local MODIFIER_KEYS = { lctrl = true, rctrl = true, lshift = true, rshift = true, lalt = true, ralt = true,
  lwin = true, rwin = true }
local ASSIGN_EDGE = { 96, 196, 186, 230 }
local SEL_EDGE = { 255, 214, 120, 255 }

local function say(msg) chat.add("Hotbars: " .. msg) end

local function slot_of(key)
  local b, i = tostring(key or ""):match("^(%d+):(%d+)$")
  return b and { b = tonumber(b), i = tonumber(i) } or nil
end
local suggested = {}   -- "kind:id" the battle menu suggests
local shades = {}      -- "b:i" -> recast shade
local top_dock = false

local function opts()
  return {
    bars = math.min(settings.get("bars"), data.MAX_BARS),
    slots = math.min(settings.get("slots"), data.MAX_SLOTS),
    size = settings.get("size"),
    gap = settings.get("gap"),
    show_keys = settings.get("show_keys"),
  }
end

-- Bars shown (1 .. the Bars setting), per character.
local function wanted()
  local most = math.min(settings.get("bars"), data.MAX_BARS)
  return math.max(1, math.min(most, tonumber(storage.character.hb_shown) or 1))
end

local function signature()
  local mj, sj = jobs.current()
  local o = opts()
  return table.concat({ tostring(mj), tostring(sj), o.bars, o.slots, o.size, o.gap, tostring(o.show_keys),
    mode.get(), tostring(settings.get("display")) }, ":")
end

local slot_xy

-- Whether the battle menu suggests the macro (any member of a group).
local function is_suggested(macro, act)
  if macro and macro.group then
    for _, m in ipairs(macro.group) do
      local a = actions.of(m)
      if a and suggested[a.kind .. ":" .. a.id] then return true end
    end
    return false
  end
  return act ~= nil and act ~= false and suggested[act.kind .. ":" .. act.id] == true
end

local function list_place(b, i)
  if overlay.on() then
    return overlay.list_at("ovb_" .. b .. "_" .. i)
  end
  local at = ui.bounds and ui.bounds(WIN, "c")
  if not at or not geo then return nil end
  local x, y = slot_xy(b, i)
  return slotview.list_place(at, x, y, geo.size, top_dock)
end

local function open_group_list(bar, slot, macro, tries)
  local function go(place) chooser.open(macro, place, function(m) return is_suggested(m, actions.of(m)) end, bar) end
  if overlay.on() then
    local place = overlay.list_at("ovb_" .. bar .. "_" .. slot)
    if not place and (tries or 0) < 8 then
      timer.after(0.03, function() open_group_list(bar, slot, macro, (tries or 0) + 1) end)
      return
    end
    go(place)
    return
  end
  go(list_place(bar, slot))
end

-- Runs a slot; a group opens its list. While a list is open its bar's keys pick members by number.
function M.fire(bar, slot)
  -- Assigning keys: a click chooses the slot instead.
  if A.on then
    A.sel = bar .. ":" .. slot
    draw()
    return
  end
  if chooser.is_open() then
    if chooser.bar() == bar then
      chooser.pick(slot)
      return
    end
    chooser.close()
  end
  local mj, sj = jobs.current()
  if not mj then return end
  local macro = data.resolve(mj, sj, bar, slot)
  if not macro then return end
  if macro.group then
    open_group_list(bar, slot, macro, 0)
  else
    runner.run_checked(macro)
  end
end

local function eased(r)
  local whole = math.floor(r)
  local f = r - whole
  return whole + f * f * (3 - 2 * f)
end

local function height(r)
  local rows = math.max(eased(r), 1)
  return rows * geo.size + (rows - 1) * geo.gap + 2 * PAD
end

slot_xy = function(b, i)
  local x = PAD + (i - 1) * (geo.size + geo.gap)
  if top_dock then return x, PAD + (b - 1) * (geo.size + geo.gap) end
  return x, height(reveal or 1) - PAD - geo.size - (b - 1) * (geo.size + geo.gap)
end

-- Whether a slot at y is past the growing side's padding.
local function in_view(y, h)
  if top_dock then return y < h - PAD end
  return y + geo.size > PAD
end

-- The + and - knobs' centres.
local function knobs(h)
  local r = geo.btn / 2
  local cx = geo.width - PAD - r
  if top_dock then return cx, PAD + r, PAD + 3 * r + 4 end
  return cx, h - PAD - 3 * r - 4, h - PAD - r
end

-- {b, i}, "plus", "minus" or nil.
local function hit(x, y)
  if not geo then return nil end
  local h = height(reveal or 1)
  local r = geo.btn / 2
  local cx, plus_y, minus_y = knobs(h)
  local n, most = wanted(), geo.bars
  local function on(cy) return (x - cx) ^ 2 + (y - cy) ^ 2 <= (r + 1) ^ 2 end
  if on(minus_y) and n > 1 then return "minus" end
  if on(plus_y) and n < most then return "plus" end
  for b = 1, geo.bars do
    for i = 1, geo.slots do
      local sx, sy = slot_xy(b, i)
      if x >= sx and x < sx + geo.size and y >= sy and y < sy + geo.size and sy >= 0 and sy + geo.size <= h then
        return { b = b, i = i }
      end
    end
  end
end

local function key_of(c) return type(c) == "table" and (c.b .. ":" .. c.i) or c end

-- ---------------------------------------------------------------------------
-- Drawing
-- ---------------------------------------------------------------------------


local function draw_overlay()
  if not geo or not eff then return end
  local rev = eased(reveal or wanted())
  overlay.bars({
    -- The overlay always stands at the bottom: the closed window's dock does not turn it over.
    size = geo.size, gap = geo.gap, slots = geo.slots, bars = geo.bars, reveal = rev, btn = geo.btn, pad = PAD,
    top_dock = false,
    eff = eff, shades = shades, show_keys = geo.show_keys,
    shown = wanted(), most = geo.bars,
    glow = 0.5 + 0.5 * math.sin(os.clock() * 3.2),
    suggested = is_suggested,
    edge = EDGE, hint = keys.hint, st = st, redraw = draw,
    assigning = A.on, selected = A.sel, assign_edge = ASSIGN_EDGE, selected_edge = SEL_EDGE,
    fire = function(c) M.fire(c.b, c.i) end,
    menu = function(c) open_menu(c and c.b, c and c.i) end,
    swap = function(from, to)
      if not main then return end
      data.swap(main, sub, from.b, from.i, to.b, to.i)
      M.rebuild()
      if M.hooks.changed then M.hooks.changed() end
    end,
    drop = function(c, id)
      if not main or not data.get_entry(main, id) then return end
      data.assign(main, sub, c.b, c.i, id)
      slotview.flash(st, c.b .. ":" .. c.i, draw)
      M.rebuild()
      if M.hooks.changed then M.hooks.changed() end
    end,
    other = function(c)
      if c == "plus" then M.grow(1) elseif c == "minus" then M.grow(-1) end
    end,
  })
end

draw = function()
  if mode.is_pad() or not geo or not eff then return end
  -- Assigning keys: the slot pointed at is the one chosen.
  if A.on and slot_of(st.hover) then A.sel = st.hover end
  if overlay.on() then draw_overlay(); return end
  local d = {}
  local function add(c) d[#d + 1] = c end
  local s, round = geo.size, 7
  local h = height(reveal or 1)
  local glow = 0.5 + 0.5 * math.sin(os.clock() * 3.2)
  local now = os.clock()
  for b = 1, geo.bars do
    for i = 1, geo.slots do
      local x, y = slot_xy(b, i)
      if in_view(y, h) then
        local key = b .. ":" .. i
        local cell = eff[b] and eff[b][i]
        local macro = cell and cell.macro
        local act = macro and actions.of(macro)
        local sugg = is_suggested(macro, act)
        slotview.draw_slot(add, {
          x = x, y = y, s = s, macro = macro, act = act, shade = shades[key], up = top_dock,
          hover = st.hover == key,
          edge = A.on and (A.sel == key and SEL_EDGE or ASSIGN_EDGE) or macro and EDGE[cell.from] or slotview.EMPTY_EDGE,
          hint = (geo.show_keys or A.on) and keys.hint(b, i) or nil, fallback = keys.hint(b, i),
          st = st, key = key, source = st.drag and st.drag.cell.b == b and st.drag.cell.i == i,
          hit_key = function(px, py) return key_of(hit(px, py)) end,
          -- Suggested: a pulsing gold fade inward, and a glow ring outside.
          before = sugg and function()
            local depth = s * (0.22 + 0.06 * glow)
            local rings = 10
            local step_w = depth / rings
            for k = 0, rings - 1 do
              local t = k / rings
              local inset = k * step_w
              add({ type = "rect", x = x + inset, y = y + inset, w = s - 2 * inset, h = s - 2 * inset,
                round = math.max(0, round - inset), line = step_w + 0.5,
                colour = { GLOW[1], GLOW[2], GLOW[3], math.floor((55 + 55 * glow) * (1 - t) * (1 - t)) } })
            end
          end or nil,
          after = (sugg or (A.on and A.sel == key)) and function()
            if A.on and A.sel == key then
              -- The slot the next key goes to.
              add({ type = "rect", x = x - 2, y = y - 2, w = s + 4, h = s + 4, round = round + 2, line = 2.5,
                colour = SEL_EDGE })
              return
            end
            add({ type = "rect", x = x - 3, y = y - 3, w = s + 6, h = s + 6, round = round + 3, line = 3,
              colour = { GLOW[1], GLOW[2], GLOW[3], math.floor(25 + 45 * glow) } })
            add({ type = "rect", x = x - 1, y = y - 1, w = s + 2, h = s + 2, round = round + 1, line = 1.5,
              colour = { GLOW[1], GLOW[2], GLOW[3], math.floor(150 + 90 * glow) } })
          end or nil,
        })
      end
    end
  end
  local n, most = wanted(), geo.bars
  local r = geo.btn / 2
  local cx, plus_y, minus_y = knobs(h)
  local function knob_live(name)
    return name == "plus" and n < most or name == "minus" and n > 1
  end
  local buttons = {
    { "plus", plus_y, "Show another bar" },
    { "minus", minus_y, "Show one bar fewer" },
  }
  for _, btn in ipairs(buttons) do
    local name, cy, tip = btn[1], btn[2], btn[3]
    local live = knob_live(name)
    local ink = not live and GOLD_OFF or (st.hover == name and INK or GOLD)
    add({ type = "circle", x = cx, y = cy, r = r, colour = KEY })
    add({ type = "circle", x = cx, y = cy, r = r - 0.75, line = 1.5, colour = ink })
    local a = r * 0.45
    add({ type = "line", x1 = cx - a, y1 = cy, x2 = cx + a, y2 = cy, width = 1.6, colour = ink })
    if name == "plus" then
      add({ type = "line", x1 = cx, y1 = cy - a, x2 = cx, y2 = cy + a, width = 1.6, colour = ink })
    end
    add({ type = "tip", x = cx - r, y = cy - r, w = 2 * r, h = 2 * r, title = "", text = tip })
  end
  slotview.draw_ghost(add, st, s, function(c) local cell = eff[c.b] and eff[c.b][c.i]; return cell and cell.macro end)
  ui.update(WIN, "c", { h = h, draw = d })
end

-- ---------------------------------------------------------------------------
-- Animation
-- ---------------------------------------------------------------------------

local anim, last = nil, nil

local function busy()
  return reveal ~= wanted() or next(suggested) ~= nil
end

local function step()
  if mode.is_pad() then return end
  local now = os.clock()
  local dt = math.min(0.05, math.max(0, now - (last or now)))
  last = now
  local target = wanted()
  reveal = reveal or target
  if reveal < target then
    reveal = math.min(target, reveal + dt * SPEED)
  elseif reveal > target then
    reveal = math.max(target, reveal - dt * SPEED)
  end
  draw()
  if not busy() and anim then
    events.off(anim)
    anim, last = nil, nil
  end
end

local function animate()
  if not anim and busy() then
    last = os.clock()
    anim = events.on("frame", step)
  end
end

function M.grow(d)
  local n = wanted()
  local most = math.min(settings.get("bars"), data.MAX_BARS)
  local to = math.max(1, math.min(most, n + d))
  if to == n then return end
  storage.character.hb_shown = to
  storage.save()
  reveal = to
  animate()
  draw()
end

-- ---------------------------------------------------------------------------
-- Window
-- ---------------------------------------------------------------------------

-- ---------------------------------------------------------------------------
-- Assign Keys: the bars take no clicks or hotkeys; the slot pointed at (or clicked) takes the next key pressed.
-- ---------------------------------------------------------------------------


-- "ctrl+shift+q" from a key event.
local function combo_of(e)
  local p = {}
  if e.ctrl then p[#p + 1] = "ctrl" end
  if e.alt then p[#p + 1] = "alt" end
  if e.shift then p[#p + 1] = "shift" end
  p[#p + 1] = e.key
  return table.concat(p, "+")
end

-- "ctrl+shift+q" as people write it: "Ctrl+Shift+Q".
local function pretty(combo)
  local out, rest = {}, combo
  while true do
    local m, tail = rest:match("^(%a+)%+(.+)$")
    if m == "ctrl" or m == "alt" or m == "shift" then
      out[#out + 1] = m:sub(1, 1):upper() .. m:sub(2)
      rest = tail
    else
      break
    end
  end
  out[#out + 1] = #rest == 1 and rest:upper() or (rest:sub(1, 1):upper() .. rest:sub(2))
  return table.concat(out, "+")
end

-- What else takes `combo`: this mod's other slots, the game's key functions, other mods' binds and the client's
-- keys (binds.users; clients before it: the slots alone).
local function users_of(combo, b, i)
  local out = {}
  local o = opts()
  for b2 = 1, o.bars do
    for s2 = 1, o.slots do
      if (b2 ~= b or s2 ~= i) and keys.combo(b2, s2) == combo then
        out[#out + 1] = ("bar %d, slot %d"):format(b2, s2)
      end
    end
  end
  if binds.users then
    local ok, list = pcall(binds.users, combo)
    for _, u in ipairs(ok and list or {}) do
      out[#out + 1] = u.kind == "mod" and ("the %s mod"):format(u.name) or u.name
    end
  end
  return out
end

local function set_key(b, i, combo)
  keys.set_slot(b, i, combo)
  say(("bar %d, slot %d: %s."):format(b, i, keys.hint_or(b, i)))
  draw()
end

-- A collision asks first, at the pointer (over the slot).
local function assign_key(b, i, combo)
  local users = users_of(combo, b, i)
  if #users == 0 then
    set_key(b, i, combo)
    return
  end
  A.asking = true
  ui.prompt(("%s is also %s. Use it for bar %d, slot %d anyway?"):format(pretty(combo), table.concat(users, "; "), b, i),
    { "Use It", "Cancel" }, function(k)
      A.asking = false
      if k == 1 then set_key(b, i, combo) end
    end, "pointer")
end

-- While assigning: Escape ends it; any other key (modifiers alone aside) goes to the slot chosen.
local function on_key(e)
  if not A.on or A.asking or e.typing or not e.down then return false end
  if MODIFIER_KEYS[e.key] then return false end
  if e.repeated then return true end
  if e.key == "escape" and not (e.ctrl or e.alt or e.shift) then
    M.set_assigning(false)
    return true
  end
  local c = slot_of(A.sel)
  if not c then
    say("point at a slot, then press its key.")
    return true
  end
  assign_key(c.b, c.i, combo_of(e))
  return true
end

-- Off without a word (the bars went away).
local function stop_assigning()
  A.on, A.sel, A.asking = false, nil, false
  if A.filter then
    input.off(A.filter)
    A.filter = nil
  end
end

function M.assigning() return A.on end

function M.set_assigning(on)
  if on == A.on then return end
  if on then
    A.on, A.sel, A.asking = true, nil, false
    A.filter = input.on_key(on_key)
    say("assigning keys: point at a slot (or click it) and press its key. Esc, or the right-click menu, ends it.")
  else
    stop_assigning()
    say("done assigning keys.")
  end
  M.rebind()
  M.rebuild()
end

-- The right-click menu, on slot (b, i) or (b nil) off the slots.
open_menu = function(b, i)
  local items = {}
  local macro, from
  if b and main then
    macro, from = data.resolve(main, sub, b, i)
    if A.on then
      if keys.get_slot(b, i) ~= "" then
        items[#items + 1] = { "Clear this slot's key", function() set_key(b, i, "") end }
      end
      if keys.get_slot(b, i) then
        items[#items + 1] = { "Use the bar's key", function()
          keys.set_slot(b, i, nil)
          say(("bar %d, slot %d: %s."):format(b, i, keys.hint_or(b, i)))
          draw()
        end }
      end
    elseif macro then
      items[#items + 1] = { "Clear the slot", function()
        data.clear(main, sub, b, i)
        M.rebuild()
        if M.hooks.changed then M.hooks.changed() end
      end }
    end
  end
  items[#items + 1] = { "Open the manager", function() M.hooks.open_manager() end }
  if A.on then
    items[#items + 1] = { "Reset Every Key", function()
      A.asking = true
      ui.prompt("Put every bar and slot back on its default key?", { "Reset", "Cancel" }, function(k)
        A.asking = false
        if k == 1 then
          keys.reset()
          draw()
        end
      end, "pointer")
    end }
    items[#items + 1] = { "Stop Assigning Keys", function() M.set_assigning(false) end }
  else
    items[#items + 1] = { "Assign Keys", function() M.set_assigning(true) end }
  end
  if b then
    menu.open(("Bar %d, Slot %d"):format(b, i), macro, keys.hint_or(b, i), items)
  else
    menu.open(A.on and "Assigning Keys" or "Hotbars", nil, nil, items)
  end
end

local function on_event(_, e)
  -- A right-click on the window off the slots' canvas (its heading, its edges): the menu without a slot.
  if e.kind == "context" then
    open_menu()
    return
  end
  if e.kind == "dock" then
    top_dock = e.side == "top"
    draw()
    return
  end
  if e.id ~= "c" then return end
  slotview.handle(e, {
    st = st, hit = hit, key_of = key_of, draw = draw,
    macro_of = function(c) local cell = eff[c.b] and eff[c.b][c.i]; return cell and cell.macro end,
    same = function(a, b) return a.b == b.b and a.i == b.i end,
    drop = function(c, id)
      -- Into the set the slot shows.
      if not main then return end
      data.assign(main, sub, c.b, c.i, id)
      slotview.flash(st, key_of(c), draw)
      M.rebuild()
      if M.hooks.changed then M.hooks.changed() end
    end,
    fire = function(c) M.fire(c.b, c.i) end,
    menu = function(c) open_menu(c and c.b, c and c.i) end,
    swap = function(from, to)
      if not main then return end
      data.swap(main, sub, from.b, from.i, to.b, to.i)
      M.rebuild()
      if M.hooks.changed then M.hooks.changed() end
    end,
    other = function(c)
      if c == "plus" then M.grow(1) elseif c == "minus" then M.grow(-1) end
    end,
  })
end

function M.rebuild()
  built_sig = signature()
  if hidden or not settings.get("on") or not settings.get("visible") or mode.is_pad() then
    if A.on then
      stop_assigning()
      M.rebind()
    end
    ui.close(WIN)
    overlay.clear("ovb")
    geo, eff, last_sig = nil, nil, nil
    if anim then events.off(anim); anim, last = nil, nil end
    return
  end
  main, sub = jobs.current()
  if not main then
    built_sig = nil
    overlay.clear("ovb")
    return
  end
  local o = opts()
  local btn = 18 -- knob diameter
  geo = { size = o.size, gap = o.gap, slots = o.slots, bars = o.bars, show_keys = o.show_keys, btn = btn,
    width = PAD + o.slots * o.size + (o.slots - 1) * o.gap + PAD + btn + PAD }
  eff = data.effective(main, sub)
  reveal = math.min(reveal or wanted(), o.bars)
  if overlay.on() then
    ui.close(WIN)
    M.recasts()
    animate()
    return
  end
  overlay.clear("ovb")
  ui.window(WIN, {
    title = "Hotbars",
    heading = A.on and "Assigning keys: point at a slot, press its key (Esc: done)" or (main .. "/" .. (sub or "-")),
    edge = true,
    grow = "up",
    width = geo.width + 34, -- plus the edge frame's margins
    elements = { { type = "canvas", id = "c", w = geo.width, h = height(reveal) } },
  })
  ui.on(WIN, on_event)
  ui.open(WIN)
  M.recasts()
  animate()
end

-- Every quarter second.
function M.recasts()
  if mode.is_pad() or not geo or not eff then return end
  local r = game.recasts()
  local me = game.player()
  shades = {}
  for b = 1, geo.bars do
    for i = 1, geo.slots do
      local cell = eff[b] and eff[b][i]
      shades[b .. ":" .. i] = cell and cell.macro and slotview.shade(cell.macro, r, me) or nil
    end
  end
  draw()
end

events.on("suggestions", function(rows)
  if mode.is_pad() then return end
  suggested = {}
  for _, r in ipairs(rows or {}) do
    if r.kind and r.id then suggested[r.kind .. ":" .. r.id] = true end
    for _, g in ipairs(r.group or {}) do suggested["spell:" .. g] = true end
  end
  animate()
  draw()
end)

chooser.hooks.picked = function() M.recasts() end

function M.rebind()
  -- Assigning keys: none bound, so their presses reach the assignment.
  if not settings.get("on") or mode.is_pad() or A.on then
    keys.bind_all(0, 0, M.fire)
    return
  end
  local o = opts()
  keys.bind_all(o.bars, o.slots, M.fire)
end

-- Every second.
function M.tick()
  if signature() ~= built_sig then M.rebuild() end
end

function M.set_hidden(value)
  hidden = value
  M.rebuild()
end

function M.is_hidden() return hidden end

function M.tile_drag_ended()
  if mode.is_pad() or st.tile_at == nil then return end
  st.tile_at = nil
  draw()
end

function M.clear_drag()
  slotview.reset(st)
end

function M.hover_overlay(key)
  if not overlay.on() or key == st.hover then return end
  st.hover = key
  draw()
end

return M
