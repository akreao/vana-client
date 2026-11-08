-- The hotbars in an edge window, as the Timers and the training regime are: on a screen edge, dragged along the edges,
-- folding to its tab. Its canvas grows a bar at a time: + and - at its lower right show more or fewer bars (bar 1 at
-- the bottom, the next ones above it), easing the window's height. A slot shows its action's icon (or its label), its hotkey, a shade
-- with the seconds left while the action recasts, and its macro in a tip; one whose action cannot be used now is
-- dimmed, and one the battle menu suggests glows. A click fires it, a right click opens its menu, and a slot dragged
-- onto another swaps them. Slots the main job's set fills have a gold edge, the sub job's set's a blue one.

local data = require("src.data")
local runner = require("src.runner")
local jobs = require("src.jobs")
local keys = require("src.keys")
local actions = require("src.actions")

local M = {}

-- Set by init.lua: open_manager(), slot_menu(bar, slot, from), changed()
M.hooks = {}

local WIN = "bars"

local EMPTY_BG = { 14, 16, 24, 170 }
local FILLED_BG = { 20, 24, 38, 235 }
local GLOSS = { 255, 255, 255, 14 }
local EMPTY_EDGE = { 90, 94, 108, 120 }
local EDGE = { main = { 206, 168, 92, 235 }, sub = { 104, 164, 232, 240 } }
local HOVER = { 255, 255, 255, 30 }
local FLASH = { 255, 236, 190, 120 }
local SHADE = { 0, 0, 0, 165 }
local UNUSABLE = { 6, 8, 14, 165 }
local KEY_BG = { 0, 0, 0, 150 }
local GLOW = { 255, 214, 120 }
local GHOST = { 255, 255, 255, 140 }
local GHOST_BG = { 20, 24, 38, 120 }
local DROP = { 255, 214, 120, 200 }
local DRAGGED = { 0, 0, 0, 120 }
local BTN_HOVER = { 255, 255, 255, 22 }
local BTN_EDGE = { 214, 184, 120, 90 }
local BTN_EDGE_OFF = { 214, 184, 120, 30 }
local BTN_TEXT = { 236, 222, 186, 255 }
local BTN_TEXT_OFF = { 150, 150, 160, 110 }

-- Bars per second while the window grows or shrinks.
local SPEED = 4.5
local PAD = 8

local hidden = false   -- //hb show | hide
local built_sig = nil  -- what the window was made for
local reveal = nil     -- the bars shown now, eased toward wanted()
local geo = nil        -- the layout: size, gap, slots, bars, width, button size
local eff, main, sub = nil, nil, nil
local hover = nil      -- "b:i", "plus" or "minus" under the pointer
local flash = {}       -- "b:i" -> os.clock() until its fire flash ends
local drag = nil       -- {b, i, x, y}: the slot a drag started on and where the pointer is now
local suggested = {}   -- "spell:<id>" / "ability:<id>" the battle menu suggests now
local shades = {}      -- "b:i" -> {left, frac, usable} from the last recast tick

local function opts()
  return {
    bars = math.min(settings.get("bars"), data.MAX_BARS),
    slots = math.min(settings.get("slots"), data.MAX_SLOTS),
    size = settings.get("size"),
    gap = settings.get("gap"),
    show_keys = settings.get("show_keys"),
  }
end

-- How many bars show (1 .. the Bars setting), kept per character.
local function wanted()
  local most = math.min(settings.get("bars"), data.MAX_BARS)
  return math.max(1, math.min(most, tonumber(storage.character.hb_shown) or 1))
end

local function signature()
  local mj, sj = jobs.current()
  local o = opts()
  return table.concat({ tostring(mj), tostring(sj), o.bars, o.slots, o.size, o.gap, tostring(o.show_keys) }, ":")
end

-- Runs the macro in a slot for the current jobs.
function M.fire(bar, slot)
  local mj, sj = jobs.current()
  if not mj then return end
  local macro = data.resolve(mj, sj, bar, slot)
  if macro then runner.run_checked(macro) end
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

-- Bar 1 is the bottom row; each next bar stands above it (the top ones come into view as the canvas grows).
local function slot_xy(b, i)
  local h = height(reveal or 1)
  return PAD + (i - 1) * (geo.size + geo.gap), h - PAD - geo.size - (b - 1) * (geo.size + geo.gap)
end

-- What is at (x, y) in the canvas: {b, i}, "plus", "minus" or nil.
local function hit(x, y)
  if not geo then return nil end
  local bx = geo.width - PAD - geo.btn
  local h = height(reveal or 1)
  if x >= bx and x <= bx + geo.btn then
    if y >= h - PAD - geo.btn and y <= h - PAD then return "minus" end
    if y >= h - PAD - 2 * geo.btn - 3 and y < h - PAD - geo.btn - 3 then return "plus" end
  end
  for b = 1, geo.bars do
    for i = 1, geo.slots do
      local sx, sy = slot_xy(b, i)
      if x >= sx and x < sx + geo.size and y >= sy and y < sy + geo.size and sy >= 0 then
        return { b = b, i = i }
      end
    end
  end
end

local function key_of(c) return type(c) == "table" and (c.b .. ":" .. c.i) or c end

-- ---------------------------------------------------------------------------
-- Drawing
-- ---------------------------------------------------------------------------

local function draw()
  if not geo or not eff then return end
  local d = {}
  local function add(c) d[#d + 1] = c end
  local s, round = geo.size, 7
  local h = height(reveal or 1)
  local k = s / 52
  local glow = 0.5 + 0.5 * math.sin(os.clock() * 3.2)
  local now = os.clock()
  for b = 1, geo.bars do
    for i = 1, geo.slots do
      local x, y = slot_xy(b, i)
      if y + s > 0 then
        local key = b .. ":" .. i
        local cell = eff[b] and eff[b][i]
        local macro = cell and cell.macro
        local act = macro and actions.of(macro)
        add({ type = "rect", x = x - 1, y = y - 1, w = s + 2, h = s + 2, round = round + 1, line = 1.5,
          colour = macro and EDGE[cell.from] or EMPTY_EDGE })
        add({ type = "rect", x = x, y = y, w = s, h = s, round = round, colour = macro and FILLED_BG or EMPTY_BG })
        if act then
          local isz = math.floor(s * 0.7)
          add({ type = "icon", x = x + (s - isz) / 2, y = y + s * 0.08, size = isz, kind = act.kind, id = act.id })
        end
        add({ type = "rect", x = x + 2, y = y + 2, w = s - 4, h = s * 0.42, round = round - 2, colour = GLOSS })
        local sh = shades[key]
        if sh and sh.left > 0 then
          add({ type = "rect", x = x, y = y, w = s, h = math.max(2, s * sh.frac), round = round, colour = SHADE })
        elseif sh and not sh.usable then
          add({ type = "rect", x = x, y = y, w = s, h = s, round = round, colour = UNUSABLE })
        end
        if macro then
          local label = data.display_label(macro)
          if act then
            add({ type = "text", x = x + s / 2, y = y + s - 15 * k, text = label:match("^[^\n]*"), size = 0.8 * k,
              align = "center", colour = { 244, 242, 236, 255 }, outline = { 0, 0, 0, 230 } })
          else
            local lines = {}
            for l in label:gmatch("[^\n]+") do lines[#lines + 1] = l end
            local top = y + (s - #lines * 15 * k) / 2
            for n, l in ipairs(lines) do
              add({ type = "text", x = x + s / 2, y = top + (n - 1) * 15 * k, text = l, size = 0.9 * k,
                align = "center", colour = { 255, 255, 255, 255 }, outline = { 0, 0, 0, 230 } })
            end
          end
        end
        if sh and sh.left > 0 then
          local secs = math.ceil(sh.left)
          local t = secs >= 60 and ("%d:%02d"):format(secs // 60, secs % 60) or tostring(secs)
          add({ type = "text", x = x + s / 2, y = y + s * 0.22, text = t, size = 1.25 * k, font = "bold",
            align = "center", colour = { 255, 236, 180, 255 }, outline = { 0, 0, 0, 255 } })
        end
        if geo.show_keys then
          local hint = keys.hint(b, i)
          add({ type = "rect", x = x + 2, y = y + 2, w = 7 * k * #hint + 6, h = 13 * k, round = 3, colour = KEY_BG })
          add({ type = "text", x = x + 5, y = y + 2, text = hint, size = 0.72 * k, colour = { 236, 230, 214, 255 } })
        end
        if act and suggested[act.kind .. ":" .. act.id] then
          add({ type = "rect", x = x - 3, y = y - 3, w = s + 6, h = s + 6, round = round + 3, line = 3,
            colour = { GLOW[1], GLOW[2], GLOW[3], math.floor(25 + 45 * glow) } })
          add({ type = "rect", x = x - 1, y = y - 1, w = s + 2, h = s + 2, round = round + 1, line = 1.5,
            colour = { GLOW[1], GLOW[2], GLOW[3], math.floor(150 + 90 * glow) } })
        end
        if hover == key then
          add({ type = "rect", x = x, y = y, w = s, h = s, round = round, colour = HOVER })
        end
        if flash[key] and flash[key] > now then
          add({ type = "rect", x = x, y = y, w = s, h = s, round = round, colour = FLASH })
        end
        if drag and drag.x then
          if drag.b == b and drag.i == i then
            add({ type = "rect", x = x, y = y, w = s, h = s, round = round, colour = DRAGGED })
          elseif key_of(hit(drag.x, drag.y)) == key then
            add({ type = "rect", x = x - 1, y = y - 1, w = s + 2, h = s + 2, round = round + 1, line = 2, colour = DROP })
          end
        end
        if macro then
          local name = data.trim(macro.name)
          add({ type = "tip", x = x, y = y, w = s, h = s, title = name ~= "" and name or keys.hint(b, i),
            text = table.concat(macro.lines or {}, "\n") })
        end
      end
    end
  end
  -- + over - at the lower right, beside the bottom bar.
  local n, most = wanted(), geo.bars
  local bx = geo.width - PAD - geo.btn
  local buttons = {
    { "plus", "+", h - PAD - 2 * geo.btn - 3, n < most, "Show another bar" },
    { "minus", "-", h - PAD - geo.btn, n > 1, "Show one bar fewer" },
  }
  for _, btn in ipairs(buttons) do
    local name, label, by, live, tip = btn[1], btn[2], btn[3], btn[4], btn[5]
    if live and hover == name then
      add({ type = "rect", x = bx, y = by, w = geo.btn, h = geo.btn, round = 4, colour = BTN_HOVER })
    end
    add({ type = "rect", x = bx, y = by, w = geo.btn, h = geo.btn, round = 4, line = 1,
      colour = live and BTN_EDGE or BTN_EDGE_OFF })
    add({ type = "text", x = bx + geo.btn / 2, y = by + geo.btn * 0.08, text = label, size = geo.btn / 18,
      font = "bold", align = "center", colour = live and BTN_TEXT or BTN_TEXT_OFF })
    add({ type = "tip", x = bx, y = by, w = geo.btn, h = geo.btn, title = "", text = tip })
  end
  -- The dragged slot: a translucent copy under the pointer.
  if drag and drag.x then
    local cell = eff[drag.b] and eff[drag.b][drag.i]
    local macro = cell and cell.macro
    local act = macro and actions.of(macro)
    local gx, gy = drag.x - s / 2, drag.y - s / 2
    add({ type = "rect", x = gx, y = gy, w = s, h = s, round = round, colour = GHOST_BG })
    if act then
      local isz = math.floor(s * 0.7)
      add({ type = "icon", x = gx + (s - isz) / 2, y = gy + s * 0.08, size = isz, kind = act.kind, id = act.id,
        colour = GHOST })
    elseif macro then
      add({ type = "text", x = gx + s / 2, y = gy + s / 2 - 8 * k, text = data.display_label(macro):match("^[^\n]*"),
        size = 0.9 * k, align = "center", colour = GHOST, outline = { 0, 0, 0, 120 } })
    end
  end
  ui.update(WIN, "c", { h = h, draw = d })
end

-- ---------------------------------------------------------------------------
-- Animation: the height easing, the suggestion glow breathing
-- ---------------------------------------------------------------------------

local anim, last = nil, nil

local function busy()
  return reveal ~= wanted() or next(suggested) ~= nil
end

local function step()
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

-- + shows one bar more, - one fewer (1 .. the Bars setting).
function M.grow(d)
  local n = wanted()
  local most = math.min(settings.get("bars"), data.MAX_BARS)
  local to = math.max(1, math.min(most, n + d))
  if to == n then return end
  storage.character.hb_shown = to
  storage.save()
  animate()
  draw()
end

-- ---------------------------------------------------------------------------
-- The window and its events
-- ---------------------------------------------------------------------------

local function on_event(_, e)
  if e.id ~= "c" then return end
  if e.kind == "hover" or e.kind == "leave" then
    local now = e.kind == "hover" and key_of(hit(e.x, e.y)) or nil
    if now ~= hover then
      hover = now
      draw()
    end
  elseif e.kind == "click" then
    local c = hit(e.x, e.y)
    if c == "plus" then
      M.grow(1)
    elseif c == "minus" then
      M.grow(-1)
    elseif type(c) == "table" then
      if e.button == "right" then
        local _, from = data.resolve(main, sub, c.b, c.i)
        M.hooks.slot_menu(c.b, c.i, from)
      else
        flash[c.b .. ":" .. c.i] = os.clock() + 0.12
        draw()
        timer.after(0.13, draw)
        M.fire(c.b, c.i)
      end
    end
  elseif e.kind == "drag_start" then
    local c = hit(e.x, e.y)
    local cell = type(c) == "table" and eff[c.b] and eff[c.b][c.i]
    drag = cell and cell.macro and { b = c.b, i = c.i, x = e.x, y = e.y } or nil
    if drag then draw() end
  elseif e.kind == "drag" and drag then
    drag.x, drag.y = e.x, e.y
    draw()
  elseif e.kind == "drag_end" and drag then
    local to = hit(e.x, e.y)
    local from = drag
    drag = nil
    draw()
    if type(to) ~= "table" or (to.b == from.b and to.i == from.i) or not main then return end
    local _, a = data.resolve(main, sub, from.b, from.i)
    local _, b = data.resolve(main, sub, to.b, to.i)
    data.swap(main, (a == "sub" or b == "sub") and sub or nil, from.b, from.i, to.b, to.i)
    M.rebuild()
    if M.hooks.changed then M.hooks.changed() end
  end
end

function M.rebuild()
  built_sig = signature()
  if hidden or not settings.get("on") or not settings.get("visible") then
    ui.close(WIN)
    return
  end
  main, sub = jobs.current()
  if not main then
    built_sig = nil -- jobs not known yet; the tick tries again
    return
  end
  local o = opts()
  local btn = math.floor(math.max(16, o.size * 0.38))
  geo = { size = o.size, gap = o.gap, slots = o.slots, bars = o.bars, show_keys = o.show_keys, btn = btn,
    width = PAD + o.slots * o.size + (o.slots - 1) * o.gap + PAD + btn + PAD }
  eff = data.effective(main, sub)
  reveal = math.min(reveal or wanted(), o.bars)
  ui.window(WIN, {
    title = "Hotbars",
    heading = main .. "/" .. (sub or "-"),
    edge = true,
    -- The canvas plus the edge frame's margins (16 + 18 at most), so it never overflows.
    width = geo.width + 34,
    elements = { { type = "canvas", id = "c", w = geo.width, h = height(reveal) } },
  })
  ui.on(WIN, on_event)
  ui.open(WIN)
  M.recasts()
  animate()
end

-- The recast shades and what cannot be used now, every quarter second.
function M.recasts()
  if not geo or not eff then return end
  local r = game.recasts()
  local me = game.player()
  shades = {}
  for b = 1, geo.bars do
    for i = 1, geo.slots do
      local cell = eff[b] and eff[b][i]
      local act = cell and cell.macro and actions.of(cell.macro)
      if act then
        local left, frac = actions.recast(act, r)
        shades[b .. ":" .. i] = { left = left, frac = frac, usable = actions.usable(act, me) }
      end
    end
  end
  draw()
end

-- The battle menu's suggestions (while it is open): their slots glow, breathing gently.
events.on("suggestions", function(rows)
  suggested = {}
  for _, r in ipairs(rows or {}) do
    if r.kind and r.id then suggested[r.kind .. ":" .. r.id] = true end
    for _, g in ipairs(r.group or {}) do suggested["spell:" .. g] = true end
  end
  animate()
  draw()
end)

function M.rebind()
  if not settings.get("on") then
    keys.bind_all(0, 0, M.fire)
    return
  end
  local o = opts()
  keys.bind_all(o.bars, o.slots, M.fire)
end

-- Once a second: rebuilds when the jobs or the layout settings changed.
function M.tick()
  if signature() ~= built_sig then M.rebuild() end
end

function M.set_hidden(value)
  hidden = value
  M.rebuild()
end

function M.is_hidden() return hidden end

return M
