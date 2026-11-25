-- Auto Ranged: keeps shooting /ra while you are engaged. Manual (nothing automatic), Auto, or Until hate (holds fire
-- while the monster is attacking you, and shoots again when it turns to someone else). It waits for each shot to
-- finish, holds while you move, cast or use an item, and pauses a moment when the game refuses a shot (out of range,
-- out of sight). It only does what /ra does.

local C = require("src.core")

local s = C.new()
local saved = storage.character
s.mode = C.LABELS[saved.mode or ""] and saved.mode or "manual"
s.delay = tonumber(saved.delay) or 0.5

local function save()
  saved.mode, saved.delay = s.mode, s.delay
  storage.save()
end

local W = "autora"
local function marked() return ({ manual = 1, auto = 2, hate = 3 })[s.mode] end

ui.window(W, {
  title = "Auto RA",
  edge = true,
  heading = "Auto ranged attack",
  width = 260,
  elements = {
    { type = "buttons", id = "mode", items = { "Manual", "Auto", "Until hate" }, marked = marked(),
      tip = "Auto keeps shooting while you are engaged. Until hate holds while the monster attacks you." },
    { type = "text", id = "why", text = "", dim = true },
  },
})

local function set_mode(mode)
  s.mode = mode
  C.reset(s)
  save()
  ui.update(W, "mode", { marked = marked() })
  ui.update(W, "why", { text = "" })
  chat.add("Auto RA: " .. C.LABELS[mode] .. ".")
end

ui.on(W, function(_, e)
  if e.kind == "set" and e.id == "mode" then set_mode(C.MODES[e.index]) end
end)

if saved.shown ~= false then ui.show(W) end

local shown_why
timer.every(0.2, function()
  local me = game.player()
  local eq = game.equipment()
  local shoot = C.tick(s, { now = clock(), me = me, target = game.target(), ranged = eq[2] ~= nil or eq[3] ~= nil })
  if s.holding ~= shown_why then
    shown_why = s.holding
    ui.update(W, "why", { text = s.holding and ("Holding: " .. s.holding) or "" })
  end
  if shoot then actions.ranged("<t>") end
end)

events.on("action", function(a)
  local me = game.player()
  if me then C.action(s, a, me.id, clock()) end
end)

events.on("battle_message", function(actor, _, message)
  local me = game.player()
  if me then C.message(s, actor, message, me.id, clock()) end
end)

events.on("zone_change", function() C.reset(s) end)

commands.register("autora", function(what, value)
  what = what and what:lower() or ""
  if what == "" then
    if ui.is_open(W) then ui.close(W) else ui.show(W) end
    saved.shown = ui.is_open(W)
    storage.save()
  elseif what == "manual" or what == "off" then
    set_mode("manual")
  elseif what == "auto" or what == "on" then
    set_mode("auto")
  elseif what == "hate" or what == "safe" then
    set_mode("hate")
  elseif what == "delay" and tonumber(value) then
    s.delay = math.max(0, math.min(5, tonumber(value)))
    save()
    chat.add(("Auto RA: %.1f s after each shot."):format(s.delay))
  else
    for _, line in ipairs(commands.usage("autora")) do chat.add(line) end
  end
end)
