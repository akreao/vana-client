-- Gearshift's set builder: of two held copies of the best in slot (each with its own augments), the one that scores
-- more for the set is the pick, whichever comes first in the bags; the best in slot is worn even when the set's weights
-- give it nothing (its worth is what the weights miss).

local HEAD, BLM, ALL = 1 << 4, 1 << 4, 0x1fe
local ITEMS = {
  [2001] = { name = "Ref Hat", slots = HEAD, jobs = BLM, races = ALL, level = 1, description = "DEF:5" },
  [2002] = { name = "Other Hat", slots = HEAD, jobs = BLM, races = ALL, level = 1, description = "DEF:5 INT+3" },
}
local pick = require("src.build.pick")
pick.H = { item = function(id) return ITEMS[id] end }
pick.pace = function() end
local set = { weights = { int = 1 } }

local function ctx(owned)
  return { job_id = 4, race = 1, level = 75, cap = 75, owned = owned, owns = {}, kept = {},
    at_cap = { [4] = { { id = 2001, key = 2001, rec = ITEMS[2001], s = pick.stats_of(2001) } } },
    at_level = {}, names = {}, mage = false, melee_sets = {}, bis = { head = 2001 } }
end

-- The fast-cast copy first in the bags, the nuking one second: the nuking one is worn.
local r = pick.build(ctx({
  { id = 2001, key = "0:1", augments = { "Fast Cast+5%" } },
  { id = 2001, key = "0:2", augments = { "INT+8" } },
}), set, "Nuke")
local head = r.slots[4]
assert(head.pick and head.pick.key == "0:2" and head.how == "bis", "picked " .. tostring(head.pick and head.pick.key))

-- The only copy adds nothing for the set: it is still the pick, over another held hat that scores more.
r = pick.build(ctx({
  { id = 2001, key = "0:1" },
  { id = 2002, key = "0:2" },
}), set, "Nuke")
head = r.slots[4]
assert(head.pick and head.pick.id == 2001 and head.how == "bis", "picked " .. tostring(head.pick and head.pick.id))
