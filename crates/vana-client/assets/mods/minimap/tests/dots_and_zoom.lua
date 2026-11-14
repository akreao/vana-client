-- The Minimap mod: a dot for each entity the radar shows (not us), in its colour, the target ringed, named; the
-- wheel sets the zoom a quarter a notch within 48..512.

ME = { id = 1, target = 3 }
game = {
  player = function() return ME end, zone = function() return 100 end,
  info = function() return { map_grid = "(G-6)" } end,
  map = function() return { px_per_unit = 0.2 } end,
  entities = function(filter)
    -- Within the map shown at 160 map pixels: 160 / 2 / 0.2 x 1.5 + 10 = 610 yalms.
    assert(filter and math.abs(filter.distance - 610) < 0.01, filter and filter.distance)
    return {
    { id = 1, name = "Lnxcsfix", pos = { x = 0, z = 0 }, radar = 0x00b040 },
    { id = 2, name = "Forest Hare", pos = { x = 10, z = 5 }, radar = 0x808080 },
    { id = 3, name = "Orcish Fodder", pos = { x = -4, z = 2 }, radar = 0xe03030 },
    { id = 4, name = "Hidden", pos = { x = 1, z = 1 } } } end,
}
res = { zone = function() return "West Ronfaure" end }
settings = { get = function() return true end }
ui.window = function(id, s) SPEC = s end
ui.open = function() end
ui.update = function(w, id, t) SPEC.elements[1].dots = t.dots end
ui.on = function(id, f) HANDLER = f end
client = { option = function() return SPAN or 160 end, set_option = function(_, v) SPAN = v end }
events.on = function(name, f) if name == "frame" then FRAME = f end end
load(TEST.source("init.lua"), "@init.lua")()

FRAME()
assert(SPEC.heading == "West Ronfaure  (G-6)", SPEC.heading)
local dots = SPEC.elements[1].dots
assert(#dots == 2 and dots[1].tip == "Forest Hare" and dots[1].colour == 0x808080 and not dots[1].target)
assert(dots[2].target and dots[2].x == -4 and dots[2].z == 2)
HANDLER("minimap", { kind = "wheel", notches = 1 })
assert(math.abs(SPAN - 128) < 0.01, SPAN)
SPAN = 60
HANDLER("minimap", { kind = "wheel", notches = 2 })
assert(SPAN == 48, SPAN)
