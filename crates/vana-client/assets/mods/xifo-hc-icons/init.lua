-- XIFO HC Icons: the hardcore mode's shield (Iron, Steel, Silver, Random) beside each party member's row in the
-- modern party windows, for an XI: Forever Offline server. The game itself already draws the shield beside names in
-- the world; this adds the party list. The server names the area's hardcore characters when asked ("!hcicons", on
-- arriving in an area) and tells about later arrivals by itself; XIFO Link hides those lines.

local Modes = require("src.modes")

local LINK, MARK = "xifo-link", "[HC-ICONS]"
local known = {}
local drawn = {}   -- hud ids drawn last time -> what they show ("path x y"), to redraw only what changed
local on = storage.character.on ~= false
local size = 22

-- "!hcicons" is said in /say: on a server that is not XI: Forever Offline everyone near would read it. So it is said
-- by itself only for a character whose server has answered an XIFO mod before (a reply to XIFO Squad or Gambits, or
-- a hardcore line); `//hcicons refresh` says it anyway.
local warned = false
local function xifo() return storage.character.xifo == true end

local function ask(by_player)
  if not pcall(mods.call, LINK, "listen", MARK) then return end
  if by_player or xifo() then pcall(mods.call, LINK, "send", "!hcicons") end
end

local function confirmed()
  if xifo() then return end
  -- (the events come from any mod: the link itself says whether the server answered)
  local ok, seen = pcall(mods.call, LINK, "server_seen")
  if not ok and tostring(seen):find("provides no server_seen", 1, true) and not warned then
    -- an XIFO Link older than 1.2.4: it cannot say, and the events alone are not trusted
    warned = true
    chat.add("XIFO HC Icons: update XIFO Link (1.2.4 or later) in the Mods list so the shields load by themselves; "
      .. "//hcicons refresh asks now.")
  end
  if not (ok and seen) then return end
  storage.character.xifo = true
  storage.save()
  ask()
end

events.on("xifo_link_line", function(mark, text)
  if mark == MARK then
    Modes.read(known, text)
    confirmed()
  end
end)

events.on("xifo_link_reply", function(_, _, complete)
  if complete then confirmed() end
end)

events.on("zone_change", function()
  known = {}
  timer.after(3, ask)
end)

timer.after(2, ask)

local function clear()
  for id in pairs(drawn) do hud.remove(id) end
  drawn = {}
end

timer.every(0.25, function()
  if not on then return clear() end
  local now, panels = {}, {}
  for _, r in ipairs(ui.native.party_rows()) do
    local mode = known[r.name]
    if mode then
      -- left of the window's frame (HUD elements draw under the panels), level with the member's row
      panels[r.panel] = panels[r.panel] or ui.native.rect(r.panel) or r
      local id = "hc" .. r.panel .. r.slot
      -- (a negative x counts from the right edge: a window at the screen's left edge gets them at 0)
      local spec = { path = "img/" .. Modes.NAMES[mode] .. ".png", x = math.max(0, panels[r.panel].x - size - 2),
        y = r.y + (r.h - size) // 2, w = size, h = size }
      -- (hud.image reads the PNG each call: only when the shield or its place changed)
      local shows = ("%s %d %d"):format(spec.path, spec.x, spec.y)
      if drawn[id] ~= shows then hud.image(id, spec) end
      now[id] = shows
    end
  end
  for id in pairs(drawn) do
    if not now[id] then hud.remove(id) end
  end
  drawn = now
end)

commands.register("hcicons", function(what)
  what = what and what:lower() or ""
  if what == "list" then
    local any = false
    for name, mode in pairs(known) do
      chat.add(("%s: %s"):format(name, Modes.LABELS[mode]))
      any = true
    end
    if not any then chat.add("XIFO HC Icons: no hardcore characters known in this area.") end
  elseif what == "refresh" then
    ask(true)
  else
    on = not on
    storage.character.on = on
    storage.save()
    chat.add("XIFO HC Icons: the party list's shields are " .. (on and "on." or "off."))
  end
end)
