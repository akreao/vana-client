-- Auto Ranged's decisions: shoot only engaged, on a living monster, with a ranged weapon, standing still and free;
-- wait for each shot; pause on a refusal; in "hate" hold while the monster's last action was on you.

local C = require("src.core")
local s = C.new()
local ME = 100
local me = { id = ME, status = 1, pos = { x = 0, y = 0, z = 0 } }
local mob = { id = 500, kind = "mob", hpp = 80 }
local function tick(t, over)
  local g = { now = t, me = me, target = mob, ranged = true }
  for k, v in pairs(over or {}) do g[k] = v end
  return C.tick(s, g)
end

assert(not tick(1), "manual shoots nothing")
s.mode = "auto"
assert(tick(2))
assert(not tick(2.2), "the shot is under way")
C.action(s, { actor = ME, category = 12, param = 0, targets = {} }, ME, 2.3)
C.action(s, { actor = ME, category = 2, param = 0, targets = { { id = 500 } } }, ME, 3)
assert(not tick(3.2), "the pause after a shot")
assert(tick(3.6))

-- not engaged, no target, a dead one, a player, no ranged weapon
C.reset(s)
me.status = 0
assert(not tick(10))
me.status = 1
assert(not tick(10, { target = false }))
assert(not tick(10, { target = { id = 1, kind = "mob", hpp = 0 } }))
assert(not tick(10, { target = { id = 2, kind = "player", hpp = 100 } }))
assert(not tick(10, { ranged = false }) and s.holding:find("ranged", 1, true))

-- moving holds it
me.pos = { x = 3, y = 0, z = 0 }
assert(not tick(11) and s.holding == "moving")
assert(tick(11.5))
C.reset(s)

-- an interrupted shot waits 1.5 s; a spell holds until it finishes
C.action(s, { actor = ME, category = 12, param = C.INTERRUPTED, targets = {} }, ME, 20)
assert(not tick(21) and tick(21.6))
C.reset(s)
C.action(s, { actor = ME, category = 8, param = 0, targets = {} }, ME, 30)
assert(not tick(35))
C.action(s, { actor = ME, category = 4, param = 0, targets = {} }, ME, 36)
assert(not tick(36.5) and tick(37.1))

-- "out of range" pauses three seconds, only as the answer to our shot
C.reset(s)
assert(tick(39.9))
C.reset(s)
C.message(s, ME, 4, ME, 40)
assert(not tick(42) and tick(43.1))
C.reset(s)
s.asked_at = -math.huge
C.message(s, ME, 78, ME, 60)
assert(tick(60.1), "a spell's refusal long after our last shot is not ours")

-- the cast bar holds it, however long the cast
C.reset(s)
C.action(s, { actor = ME, category = 8, param = 0, targets = {} }, ME, 70)
assert(not tick(83, { casting = true }) and tick(83))

-- what shoots: a bow or gun with ammo, a boomerang, throwing ammo alone; not an instrument
local bow, arrow, boomerang, pebble, harp = { skill = 25 }, { skill = 25 }, { skill = 27 }, { skill = 27 }, { skill = 41 }
assert(C.can_shoot(bow, arrow) and not C.can_shoot(bow, nil) and C.can_shoot(boomerang, nil))
assert(C.can_shoot(nil, pebble) and not C.can_shoot(nil, arrow) and not C.can_shoot(harp, nil) and not C.can_shoot(harp, arrow))
-- a utility item in the ammo slot, or ammo of another skill (a bolt with a bow), does not shoot
assert(not C.can_shoot(bow, { id = 4596, skill = 0 }) and not C.can_shoot(bow, { id = 17336, skill = 26 }))

-- Until hate: holds while the monster's last swing was on you, shoots when it turns to someone else
C.reset(s)
s.asked_at = -math.huge
s.mode = "hate"
C.action(s, { actor = 500, category = 1, param = 0, targets = { { id = ME } } }, ME, 50)
assert(not tick(51) and s.holding == "it is attacking you")
C.action(s, { actor = 500, category = 1, param = 0, targets = { { id = 7 } } }, ME, 52)
assert(tick(52.5))
C.reset(s)
C.action(s, { actor = 500, category = 7, param = 0, targets = { { id = ME } } }, ME, 60)
assert(not tick(61), "readying a move on you counts")
assert(tick(71), "forgotten after 10 s")
