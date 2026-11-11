-- The minimap (the minimap add-ons): the zone's map around us in the minimap's edge window (panel "minimap": its
-- place, fold and size; in its corner in the classic UI, Wide Scan under it in the modern UI). The client draws the map
-- and our marker; this mod picks the dots (what the compass radar shows, monsters included, in its colours, the target
-- ringed and each named in its tooltip), the zoom (the wheel over it: the client option minimap_span) and the heading.

-- The wheel's range: map pixels across (the whole map is 512); a notch is a quarter more or less.
local SPAN_MIN, SPAN_MAX = 48, 512

local shown = false
local heading = ""
-- An older client knows no map element: it draws its own minimap in the bare window, as before.
local legacy = false

local function dots()
  local me = game.player()
  local out = {}
  for _, e in ipairs(game.entities()) do
    if e.radar and e.id ~= me.id then
      out[#out + 1] = { x = e.pos.x, z = e.pos.z, colour = e.radar, target = e.id == me.target, tip = e.name }
    end
  end
  return out
end

-- The zone and the grid square, as the location panel words them.
local function words()
  local z = game.zone()
  local name = z and res.zone(z) or ""
  local grid = game.info().map_grid or ""
  if grid == "" then return name end
  return name .. "  " .. grid
end

local function window()
  if legacy then return end
  local ok = pcall(ui.window, "minimap", { edge = true, panel = "minimap", title = "Map", heading = heading,
    elements = { { type = "map", id = "map", dots = dots() } } })
  if not ok then
    legacy = true
    ui.window("minimap", { edge = true, panel = "minimap", title = "Map" })
  end
end

local function show()
  if settings.get("on") then
    heading = words()
    window()
    ui.open("minimap")
    shown = true
  else
    ui.close("minimap")
    shown = false
  end
end

ui.on("minimap", function(_, ev)
  if ev.kind == "wheel" then
    local span = tonumber(client.option("minimap_span")) or 160
    span = math.max(SPAN_MIN, math.min(SPAN_MAX, span * 1.25 ^ -ev.notches))
    client.set_option("minimap_span", span)
  end
end)

-- The dots follow every frame; the heading when the place or the grid changes.
events.on("frame", function()
  if not shown or legacy then return end
  local h = words()
  if h ~= heading then
    heading = h
    window()
  else
    ui.update("minimap", "map", { dots = dots() })
  end
end)

show()
events.on("settings_changed", show)
