-- The hotbars on screen, in a frame that grows upward a bar at a time: + and - at its lower right show more or fewer
-- bars (bar 1 at the bottom), easing the frame and fading the bars in and out. A slot shows its action's icon (or its
-- label), its hotkey, a shade with the seconds left while the action recasts, and its macro in a tip. A click fires
-- it, a right click opens its menu, and a slot dragged onto another swaps them. Slots the main job's set fills have a
-- gold edge, the sub job's set's a blue one.

local data = require("src.data")
local runner = require("src.runner")
local jobs = require("src.jobs")
local keys = require("src.keys")
local actions = require("src.actions")

local M = {}

-- Set by init.lua: open_manager(), slot_menu(bar, slot, from), changed()
M.hooks = {}

local hidden = false   -- //hb show | hide
local built_sig = nil  -- what the drawing was made for
local shown = {}       -- slot id -> {action, x, y, size, round, bar} as drawn, for the recast shades

local EMPTY_BG = { 14, 16, 24, 150 }
local FILLED_BG = { 20, 24, 38, 230 }
local GLOSS = { 255, 255, 255, 14 }
local EMPTY_EDGE = { 90, 94, 108, 110 }
local EDGE = { main = { 206, 168, 92, 235 }, sub = { 104, 164, 232, 240 } }
local HOVER = { 255, 255, 255, 30 }
local FLASH = { 255, 236, 190, 120 }
local SHADE = { 0, 0, 0, 165 }
local KEY_BG = { 0, 0, 0, 150 }

local FRAME_SHADOW = { 0, 0, 0, 70 }
local FRAME_BG = { 8, 10, 18, 135 }
local FRAME_EDGE = { 214, 184, 120, 60 }
local FRAME_LINE = { 255, 255, 255, 18 }
local BTN_BG = { 26, 30, 46, 225 }
local BTN_HOVER = { 58, 66, 96, 245 }
local BTN_OFF = { 26, 30, 46, 90 }
local BTN_TEXT = { 236, 222, 186, 255 }
local BTN_TEXT_OFF = { 150, 150, 160, 110 }

-- Bars per second while the frame grows or shrinks.
local SPEED = 4.5

-- Sizes and offsets are in UI units, so the bars keep their size at any resolution and UI size.
local function opts()
  local u = hud.screen().ui_scale
  return {
    bars = math.min(settings.get("bars"), data.MAX_BARS),
    slots = math.min(settings.get("slots"), data.MAX_SLOTS),
    size = math.floor(settings.get("size") * u),
    gap = math.floor(settings.get("gap") * u),
    ox = settings.get("offset_x") * u,
    oy = settings.get("offset_y") * u,
    show_keys = settings.get("show_keys"),
    u = u,
  }
end

-- How many bars the frame shows (1 .. the Bars setting), kept per character.
local function wanted()
  local most = math.min(settings.get("bars"), data.MAX_BARS)
  return math.max(1, math.min(most, tonumber(storage.character.hb_shown) or 1))
end

local reveal = nil     -- the bars shown now, eased toward wanted()
local geo = nil        -- the last drawing's layout
local bar_ids = {}     -- bar -> its element ids (shown and hidden with it)
local bar_vis = {}     -- bar -> visible now
local hover = {}       -- the + / - button under the pointer

-- The line the bottom bar stands over: the window's bottom, or the top of the modern chat window at its full height
-- when it is under the bars (they stay put while it grows and folds).
local function floor_y(scr, x0, w)
  local c = ui.native.rect("chat")
  c = c and (c.room or c)
  if c and c.x < x0 + w and c.x + c.w > x0 and c.y < scr.height then return c.y end
  return scr.height
end

local function span(o, scr)
  local w = o.slots * o.size + (o.slots - 1) * o.gap
  return math.floor((scr.width - w) / 2 + o.ox), w
end

local function signature()
  local main, sub = jobs.current()
  local scr = hud.screen()
  local x0, w = span(opts(), scr)
  return table.concat({ tostring(main), tostring(sub), scr.width, scr.height, scr.ui_scale, floor_y(scr, x0, w) }, ":")
end

-- Runs the macro in a slot for the current jobs.
function M.fire(bar, slot)
  local main, sub = jobs.current()
  if not main then return end
  local macro = data.resolve(main, sub, bar, slot)
  if macro then runner.run_checked(macro) end
end

-- A slot dropped on another: the two swap, in the sub job's set when either comes from it.
local function dropped(bar, slot, from_id)
  local fb, fs = from_id:match("^s(%d+)_(%d+)$")
  fb, fs = tonumber(fb), tonumber(fs)
  if not fb or (fb == bar and fs == slot) then return end
  local main, sub = jobs.current()
  if not main then return end
  local _, a = data.resolve(main, sub, fb, fs)
  local _, b = data.resolve(main, sub, bar, slot)
  data.swap(main, (a == "sub" or b == "sub") and sub or nil, fb, fs, bar, slot)
  M.rebuild()
  if M.hooks.changed then M.hooks.changed() end
end

local function tip_of(macro)
  local lines = table.concat(macro.lines or {}, "\n")
  local name = data.trim(macro.name)
  if name == "" then return lines end
  return { title = name, text = lines }
end

-- An element of bar `b`: in its HUD window (faded as a whole) and shown or hidden with it.
local function put(kind, b, id, spec)
  spec.window = "hb_bar" .. b
  spec.visible = bar_vis[b] ~= false
  bar_ids[b][#bar_ids[b] + 1] = id
  hud[kind](id, spec)
end

-- Smoothstep inside each bar's step, so every bar eases in and out on its own.
local function eased(r)
  local whole = math.floor(r)
  local f = r - whole
  return whole + f * f * (3 - 2 * f)
end

-- The frame, the header and the bars' fade for `r` bars shown (fractional while it moves).
local function apply(r)
  if not geo then return end
  local g = geo
  local e = eased(r)
  local rows = math.max(e, 1)
  local inner = rows * g.size + (rows - 1) * g.gap
  local top = g.y_bottom + g.size - inner - g.pad
  local h = inner + 2 * g.pad
  hud.set("hb_shadow", { y = top + 2 * g.u, h = h })
  hud.set("hb_frame", { y = top, h = h })
  hud.set("hb_edge", { y = top, h = h })
  hud.set("hb_line", { y = top + 1 * g.u })
  hud.set("hdr", { y = top - 9 * g.u })
  for b = 1, g.bars do
    local a = math.max(0, math.min(1, e - (b - 1)))
    local vis = a > 0.02
    if vis ~= bar_vis[b] then
      bar_vis[b] = vis
      for _, id in ipairs(bar_ids[b]) do hud.set(id, { visible = vis }) end
      if not vis then
        -- Its recast shades go too; recasts() draws them again when it shows.
        for id, s in pairs(shown) do
          if s.bar == b then
            hud.remove(id .. "_r")
            hud.remove(id .. "_c")
          end
        end
      end
    end
    hud.opacity("hb_bar" .. b, vis and a or nil)
  end
end

-- The + / - buttons' look for the bars wanted now.
local function buttons()
  if not geo then return end
  local n, most = wanted(), geo.bars
  for _, k in ipairs({ "plus", "minus" }) do
    local live = (k == "plus" and n < most) or (k == "minus" and n > 1)
    hud.set("hb_" .. k, { colour = not live and BTN_OFF or (hover[k] and BTN_HOVER or BTN_BG) })
    hud.set("hb_" .. k .. "_t", { colour = live and BTN_TEXT or BTN_TEXT_OFF })
  end
end

local anim, last = nil, nil

local function step()
  local now = os.clock()
  local dt = math.min(0.05, math.max(0, now - (last or now)))
  last = now
  local target = wanted()
  reveal = reveal or target
  if reveal < target then
    reveal = math.min(target, reveal + dt * SPEED)
  else
    reveal = math.max(target, reveal - dt * SPEED)
  end
  apply(reveal)
  if reveal == target then
    events.off(anim)
    anim, last = nil, nil
  end
end

local function animate()
  if not anim then
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
  buttons()
  animate()
end

local function button(k, x, y, s, label, d)
  hud.rect("hb_" .. k, { x = x, y = y, w = s, h = s, round = 4 * geo.u, colour = BTN_BG, layer = 5,
    tip = d > 0 and "Show another bar" or "Show one bar fewer" })
  hud.rect("hb_" .. k .. "_e", { x = x, y = y, w = s, h = s, round = 4 * geo.u, colour = FRAME_EDGE,
    line = 1 * geo.u, layer = 6 })
  hud.text("hb_" .. k .. "_t", { x = x + s / 2, y = y + (s - 16 * geo.u * 0.95) / 2 - 1 * geo.u, text = label,
    size = 0.95, font = "bold", align = "center", colour = BTN_TEXT, layer = 7 })
  hud.on("hb_" .. k, function(_, e)
    if e.kind == "mouse_enter" then
      hover[k] = true
      buttons()
    elseif e.kind == "mouse_leave" then
      hover[k] = false
      buttons()
    elseif e.kind == "click" and e.button ~= "right" then
      M.grow(d)
    end
  end)
end

function M.rebuild()
  hud.clear()
  shown, bar_ids, bar_vis, hover = {}, {}, {}, {}
  geo = nil
  built_sig = signature()
  if hidden or not settings.get("on") or not settings.get("visible") then return end
  local main, sub = jobs.current()
  if not main then
    built_sig = nil -- jobs not known yet; the tick tries again
    return
  end

  local o = opts()
  local scr = hud.screen()
  local u = o.u
  local eff = data.effective(main, sub)
  local round = 7 * u
  local x0, w = span(o, scr)
  local y_bottom = floor_y(scr, x0, w) - o.oy - o.size
  local pad = math.floor(7 * u)
  local bsz = math.floor(math.max(16 * u, o.size * 0.38))
  geo = { x0 = x0, w = w, y_bottom = y_bottom, size = o.size, gap = o.gap, pad = pad, u = u, bars = o.bars }
  reveal = math.min(reveal or wanted(), o.bars)

  -- The frame: a soft shadow, a dark glass body, a thin warm edge and a highlight along its top.
  local fx, fw = x0 - pad, w + 2 * pad + bsz + pad
  hud.rect("hb_shadow", { x = fx + 2 * u, y = y_bottom, w = fw, h = 10, round = round + pad, colour = FRAME_SHADOW,
    layer = -3 })
  hud.rect("hb_frame", { x = fx, y = y_bottom, w = fw, h = 10, round = round + pad, colour = FRAME_BG, layer = -2 })
  hud.rect("hb_edge", { x = fx, y = y_bottom, w = fw, h = 10, round = round + pad, colour = FRAME_EDGE,
    line = 1 * u, layer = -1 })
  hud.rect("hb_line", { x = fx + round + pad, y = y_bottom, w = fw - 2 * (round + pad), h = math.max(1, 1 * u),
    colour = FRAME_LINE, layer = -1 })

  -- Text sizes follow the slot size: 1 is the UI's text.
  local k = o.size / (52 * u)
  local name_size, hint_size, text_size = 0.7 * k, 0.62 * k, 0.8 * k
  local line_h = 16 * u

  for b = 1, o.bars do
    bar_ids[b] = {}
    bar_vis[b] = eased(reveal) - (b - 1) > 0.02
    -- Bar 1 is the bottom row; each next bar stands above it.
    local y = y_bottom - (b - 1) * (o.size + o.gap)
    for i = 1, o.slots do
      local x = x0 + (i - 1) * (o.size + o.gap)
      local id = ("s%d_%d"):format(b, i)
      local cell = eff[b] and eff[b][i]
      local macro = cell and cell.macro
      local act = macro and actions.of(macro)
      put("rect", b, id .. "_f", { x = x - 1, y = y - 1, w = o.size + 2, h = o.size + 2, round = round + 1,
        colour = macro and EDGE[cell.from] or EMPTY_EDGE, line = 1.5 * u, layer = 0 })
      put("rect", b, id, { x = x, y = y, w = o.size, h = o.size, round = round,
        colour = macro and FILLED_BG or EMPTY_BG, layer = 1, tip = macro and tip_of(macro) or nil })
      -- A sheen over the top half.
      put("rect", b, id .. "_g", { x = x + 2 * u, y = y + 2 * u, w = o.size - 4 * u, h = o.size * 0.42,
        round = round - 2 * u, colour = GLOSS, layer = 3 })
      hud.on(id, function(_, e)
        if e.kind == "click" then
          if e.button == "right" then
            local _, from = data.resolve(main, sub, b, i)
            M.hooks.slot_menu(b, i, from)
          else
            hud.rect(id .. "_x", { x = x, y = y, w = o.size, h = o.size, round = round, colour = FLASH, layer = 5,
              window = "hb_bar" .. b })
            timer.after(0.12, function() hud.remove(id .. "_x") end)
            M.fire(b, i)
          end
        elseif e.kind == "mouse_enter" then
          hud.rect(id .. "_h", { x = x, y = y, w = o.size, h = o.size, round = round, colour = HOVER, layer = 5,
            window = "hb_bar" .. b })
        elseif e.kind == "mouse_leave" then
          hud.remove(id .. "_h")
        elseif e.kind == "drop" and e.from then
          dropped(b, i, e.from)
        end
      end)
      if act then
        local isz = math.floor(o.size * 0.7)
        put("icon", b, id .. "_i", { x = x + (o.size - isz) / 2, y = y + o.size * 0.08, size = isz, kind = act.kind,
          id = act.id, layer = 2 })
        shown[id] = { action = act, x = x, y = y, size = o.size, round = round, bar = b }
      end
      if macro then
        local label = data.display_label(macro)
        if act then
          -- Over the icon's foot: the label's first line.
          put("text", b, id .. "_t", { x = x + o.size / 2, y = y + o.size - line_h * name_size - 3 * u,
            text = label:match("^[^\n]*"), size = name_size, align = "center", colour = { 244, 242, 236 },
            outline = { 0, 0, 0, 230 }, layer = 4 })
        else
          local lines = {}
          for l in label:gmatch("[^\n]+") do lines[#lines + 1] = l end
          local ty = y + (o.size - #lines * line_h * text_size) / 2
          for n, l in ipairs(lines) do
            put("text", b, id .. "_t" .. n, { x = x + o.size / 2, y = ty + (n - 1) * line_h * text_size, text = l,
              size = text_size, align = "center", colour = { 255, 255, 255 }, outline = { 0, 0, 0, 230 },
              layer = 4 })
          end
        end
      end
      if o.show_keys then
        put("text", b, id .. "_k", { x = x + 4 * u, y = y + 3 * u, text = keys.hint(b, i), size = hint_size,
          colour = { 236, 230, 214 }, bg = KEY_BG, pad = 2, layer = 4 })
      end
    end
  end

  -- + over - at the frame's lower right, beside the bottom bar.
  local bx = x0 + w + pad
  button("plus", bx, y_bottom + o.size - 2 * bsz - 3 * u, bsz, "+", 1)
  button("minus", bx, y_bottom + o.size - bsz, bsz, "-", -1)

  -- The jobs on a tab over the frame's top edge; a click opens the manager.
  hud.text("hdr", { x = x0, y = y_bottom, text = main .. "/" .. (sub or "-"), size = 0.75, font = "bold",
    colour = { 232, 210, 156 }, bg = { 8, 10, 18, 200 }, pad = 5, layer = 2, tip = "Open the Hotbars manager" })
  hud.on("hdr", function() M.hooks.open_manager() end)

  apply(reveal)
  buttons()
  if reveal ~= wanted() then animate() end
  M.recasts()
end

-- The recast shades: the part still to go, from the top, and the seconds over it.
function M.recasts()
  if not next(shown) then return end
  local r = game.recasts()
  for id, s in pairs(shown) do
    local left, frac = actions.recast(s.action, r)
    if left > 0 and bar_vis[s.bar] then
      hud.rect(id .. "_r", { x = s.x, y = s.y, w = s.size, h = math.max(2, s.size * frac), round = s.round,
        colour = SHADE, layer = 3, window = "hb_bar" .. s.bar })
      local secs = math.ceil(left)
      local text = secs >= 60 and ("%d:%02d"):format(secs // 60, secs % 60) or tostring(secs)
      hud.text(id .. "_c", { x = s.x + s.size / 2, y = s.y + s.size * 0.24, text = text,
        size = s.size / (52 * hud.screen().ui_scale), align = "center", colour = { 255, 236, 180 },
        outline = { 0, 0, 0 }, layer = 4, window = "hb_bar" .. s.bar })
    else
      hud.remove(id .. "_r")
      hud.remove(id .. "_c")
    end
  end
end

function M.rebind()
  if not settings.get("on") then
    keys.bind_all(0, 0, M.fire)
    return
  end
  local o = opts()
  keys.bind_all(o.bars, o.slots, M.fire)
end

-- Once a second: redraws when the jobs or the window size changed.
function M.tick()
  if signature() ~= built_sig then M.rebuild() end
end

function M.set_hidden(value)
  hidden = value
  M.rebuild()
end

function M.is_hidden() return hidden end

return M
