-- XIFO HC Icons: the hardcore mode's shield (Iron, Steel, Silver, Random) beside each party member's row in the
-- modern party windows, for an XI: Forever Offline server. The game itself already draws the shield beside names in
-- the world; this adds the party list. The server names the area's hardcore characters when asked ("!hcicons", on
-- arriving in an area) and tells about later arrivals by itself; XIFO Link hides those lines.

local Modes = require("src.modes")

local LINK, MARK = "xifo-link", "[HC-ICONS]"
local known = {}
local drawn = {}   -- hud ids drawn last time
local on = storage.character.on ~= false
local size = 22

local function ask()
  if not pcall(mods.call, LINK, "listen", MARK) then return end
  pcall(mods.call, LINK, "send", "!hcicons")
end

events.on("xifo_link_line", function(mark, text)
  if mark == MARK then Modes.read(known, text) end
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
      local spec = { path = "img/" .. Modes.NAMES[mode] .. ".png", x = panels[r.panel].x - size - 2,
        y = r.y + (r.h - size) // 2, w = size, h = size }
      hud.image(id, spec)
      now[id] = true
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
    ask()
  else
    on = not on
    storage.character.on = on
    storage.save()
    chat.add("XIFO HC Icons: the party list's shields are " .. (on and "on." or "off."))
  end
end)
