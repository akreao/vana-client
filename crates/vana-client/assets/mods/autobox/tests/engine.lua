-- Autobox's engine: the first rule that holds and that the game would take is done, on the right target; it waits
-- while moving, casting or after an action; best tiers, best weapon skills, party order and claimed targets.

local core = require("src.core")
local presets = require("src.presets")

local SPELLS = {
  [1] = { name = "Cure", targets = 0x1F }, [2] = { name = "Cure II", targets = 0x1F },
  [3] = { name = "Cure III", targets = 0x1F }, [43] = { name = "Protect", targets = 0x1 },
  [109] = { name = "Refresh", targets = 0x1 }, [164] = { name = "Thunder", targets = 0x20, range = 12 },
  [338] = { name = "Utsusemi: Ichi", targets = 0x1 }, [339] = { name = "空蝉の術:弐", targets = 0x1 },
  [340] = { name = "Utsusemi: San", targets = 0x1 },
  [100] = { name = "Pet Mend", targets = 0x40 },
}
local ABILITIES = { [32] = { name = "Fast Blade", targets = 0x20 }, [42] = { name = "Savage Blade", targets = 0x20 },
  [520] = { name = "Berserk", targets = 1 }, [163] = { name = "Starlight", targets = 1 } }
local STATUS = { protect = { 40 }, refresh = { 43 }, poison = { 3 }, berserk = { 56 }, silence = { 6 }, amnesia = { 16 } }

local function env(o)
  local e = {
    now = o.now or 10, on = true,
    me = o.me or { id = 1, pos = { x = 0, z = 0 }, hpp = 100, mpp = 100, tp = 0, status = 0 },
    buffs = o.buffs or {}, known = o.known or { [1] = true, [2] = true, [43] = true, [109] = true, [164] = true },
    party = o.party or {}, ours = o.ours or { [1] = true }, target = o.target,
    has_ability = o.has_ability or { [32] = true, [42] = true, [520] = true }, can_shoot = o.can_shoot,
    entity = function(id) return (o.near or {})[id] end, rules = o.rules,
    casting = o.casting or false, event = false,
    usable = function(kind, id) return not (o.refuse or {})[kind .. id] end,
    spell = function(id) return SPELLS[id] end,
    ability = function(id) return ABILITIES[id] end,
    spell_id = function(n) for id, r in pairs(SPELLS) do if r.name:lower() == n:lower() then return id end end end,
    ability_id = function(n) for id, r in pairs(ABILITIES) do if r.name:lower() == n:lower() then return id end end end,
    status_ids = function(n) return STATUS[n:lower()] end,
    has_tool = o.has_tool, pet = o.pet, my_hitbox = 0.5,
  }
  return e
end

-- An engine that has already seen us standing at (0, 0).
local function fresh()
  local s = core.new()
  s.last_pos = { x = 0, z = 0 }
  return s
end

local cure = { target = "party", when = { kind = "hp_below", value = 60 }, action = { kind = "spell", name = "Cure", best = true } }
local protect = { target = "me", when = { kind = "missing", value = "Protect" }, action = { kind = "spell", name = "Protect" } }

-- Nobody hurt: the first rule passes over, Protect goes on us.
local s = fresh()
local a = core.tick(s, env({ now = 1, rules = { cure, protect } }))
assert(a and a.kind == "spell" and a.id == 43 and a.tag == "<me>" and a.rule == 2, "protect on me")

-- Right after an action, nothing; Protect on, nothing again.
assert(core.tick(s, env({ now = 1.5, rules = { cure, protect } })) == nil and s.holding == "waiting")
assert(core.tick(s, env({ now = 9, buffs = { [40] = true }, rules = { cure, protect } })) == nil)

-- Two hurt members in range: the lowest first, with the best tier known and usable (Cure III unknown: Cure II).
local party = {
  { slot = 0, party = 0, id = 1, name = "Nymh", hpp = 90, mpp = 80, in_zone = true, buffs = {} },
  { slot = 1, party = 0, id = 2, name = "Ann", hpp = 55, in_zone = true, buffs = {} },
  { slot = 2, party = 0, id = 3, name = "Bo", hpp = 30, in_zone = true, buffs = {} },
  { slot = 3, party = 0, id = 4, name = "Far", hpp = 10, in_zone = true, buffs = {} },
}
local near = { [2] = { distance = 5 }, [3] = { distance = 12 }, [4] = { distance = 30 } }
s = fresh()
a = core.tick(s, env({ now = 1, rules = { cure }, party = party, near = near }))
assert(a and a.id == 2 and a.tag == "<p2>" and a.who == "Bo", "Cure II on the lowest in range")
-- Cure II refused (no MP): Cure.
s = fresh()
a = core.tick(s, env({ now = 1, rules = { cure }, party = party, near = near, refuse = { spell2 = true } }))
assert(a and a.id == 1, "falls back a tier")

-- Moving or casting holds.
s = fresh()
core.tick(s, env({ now = 0, buffs = { [40] = true }, rules = { protect } }))
local moved = { id = 1, pos = { x = 3, z = 0 }, hpp = 100, mpp = 100, tp = 0, status = 0 }
assert(core.tick(s, env({ now = 1, me = moved, rules = { protect } })) == nil and s.holding == "moving")
assert(core.tick(s, env({ now = 2, me = moved, casting = true, rules = { protect } })) == nil and s.holding == "casting")
assert(core.tick(s, env({ now = 3, me = moved, rules = { protect } })) ~= nil, "standing still again")

-- Weapon skills: only on a monster we claimed, at 1000 TP; Best picks the first the game takes.
local ws = presets.for_job("WAR")[1]
local me = { id = 1, pos = { x = 0, z = 0 }, hpp = 100, mpp = 0, tp = 1200, status = 1 }
local mob = { kind = "mob", hpp = 80, claim = 1, name = "Goblin" }
s = fresh()
a = core.tick(s, env({ now = 1, me = me, rules = { ws }, target = mob, refuse = { ability42 = true } }))
assert(a and a.kind == "ability" and a.id == 32 and a.tag == "<t>", "best weapon skill the game takes")
s = fresh()
assert(core.tick(s, env({ now = 1, me = me, rules = { ws }, target = { kind = "mob", hpp = 80, claim = 99 } })) == nil,
  "someone else's monster")

-- A status condition on a monster never holds; an off rule is skipped.
local odd = { target = "target", when = { kind = "missing", value = "poison" }, action = { kind = "spell", name = "Thunder" } }
s = fresh()
assert(core.tick(s, env({ now = 1, me = me, rules = { odd }, target = mob })) == nil)
local off = { on = false, target = "me", when = { kind = "always" }, action = { kind = "spell", name = "Refresh" } }
s = fresh()
assert(core.tick(s, env({ now = 1, rules = { off } })) == nil)

-- Checking a rule for saving.
assert(core.check({ target = "me", when = { kind = "hp_below" }, action = { kind = "spell", name = "Cure" } }) == nil)
assert(core.check({ target = "me", when = { kind = "always" }, action = { kind = "ws", best = true } }))
assert(core.check({ target = "me", when = { kind = "always" }, action = { kind = "ability", name = "" } }) == nil)
assert(core.condition_text({ kind = "hp_below", value = 50 }) == "HP below 50%", core.condition_text({ kind = "hp_below", value = 50 }))
assert(core.action_text({ kind = "ws", best = true }) == "Best weapon skill")
assert(core.condition_text({ kind = "tp_at_least", value = 1000 }) == "My TP at least 1000")

-- Every preset rule passes the save check.
for _, j in ipairs({ "WHM", "RDM", "BLM", "PLD", "WAR", "SAM", "DRK", "THF" }) do
  for i, r in ipairs(presets.for_job(j)) do assert(core.check(r), j .. " rule " .. i) end
end

-- Best weapon skill only from the ability list (the weapon held): a move the game would not list is passed over.
s = fresh()
a = core.tick(s, env({ now = 1, me = me, rules = { ws }, target = mob, has_ability = { [32] = true } }))
assert(a and a.id == 32, "only listed weapon skills")

-- Best tier from a numbered spell: Cure II's family is Cure's.
local cure2 = { target = "me", when = { kind = "always" }, action = { kind = "spell", name = "Cure II", best = true } }
s = fresh()
a = core.tick(s, env({ now = 1, rules = { cure2 } }))
assert(a and a.id == 2, "Cure II best -> Cure II (Cure III unknown)")
s = fresh()
a = core.tick(s, env({ now = 1, rules = { cure2 }, refuse = { spell2 = true } }))
assert(a and a.id == 1, "falls back to Cure")

-- Silence stops spells, amnesia abilities and weapon skills.
s = fresh()
assert(core.tick(s, env({ now = 1, rules = { cure2 }, buffs = { [6] = true } })) == nil)
s = fresh()
assert(core.tick(s, env({ now = 1, me = me, rules = { ws }, target = mob, buffs = { [16] = true } })) == nil)

-- Ranged attack only with something to shoot.
local ra = { target = "target", when = { kind = "engaged" }, action = { kind = "ranged" } }
s = fresh()
assert(core.tick(s, env({ now = 1, me = me, rules = { ra }, target = mob, can_shoot = false })) == nil)
s = fresh()
a = core.tick(s, env({ now = 1, me = me, rules = { ra }, target = mob, can_shoot = true }))
assert(a and a.kind == "ranged" and a.tag == "<t>")

-- The first position seen is a baseline: nothing on that tick, the next one acts if we have not moved.
s = core.new()
assert(core.tick(s, env({ now = 1, rules = { protect } })) == nil and s.holding == "moving")
assert(core.tick(s, env({ now = 1.2, rules = { protect } })) ~= nil, "still on the second look")

-- Blocked by id, whatever the status names: sleep stops everything, silence spells.
s = fresh()
assert(core.tick(s, env({ now = 1, rules = { protect }, buffs = { [2] = true } })) == nil)
s = fresh()
assert(core.tick(s, env({ now = 1, me = me, rules = { ra }, target = mob, can_shoot = true, buffs = { [7] = true } })) == nil,
  "petrified: no ranged attack either")

-- A spell out of its range class's reach is passed over (Thunder: class 12, 20 yalms), so a later rule runs.
local thunder = { target = "target", when = { kind = "always" }, action = { kind = "spell", name = "Thunder" } }
local far = { kind = "mob", hpp = 80, claim = 1, name = "Goblin", distance = 35, pos = { x = 35, y = 0, z = 0 }, hitbox = 1 }
s = fresh()
a = core.tick(s, env({ now = 1, me = me, rules = { thunder, protect }, target = far }))
assert(a and a.text == "Protect", a and a.text)
s = fresh()
a = core.tick(s, env({ now = 1, me = me, rules = { thunder, protect }, target = { kind = "mob", hpp = 80, claim = 1, distance = 15,
  pos = { x = 15, y = 0, z = 0 }, hitbox = 1 } }))
assert(a and a.text == "Thunder")
-- at the edge, as the target arrow measures it: flat distance less both radii, half a yalm past the range still in
local function edge(x) return { kind = "mob", hpp = 80, claim = 1, distance = x, pos = { x = x, y = 3, z = 0 }, hitbox = 1 } end
s = fresh()
a = core.tick(s, env({ now = 1, me = me, rules = { thunder, protect }, target = edge(21.9) })) -- gap 20.4 (radii 1 + 0.5)
assert(a and a.text == "Thunder", a and a.text)
s = fresh()
a = core.tick(s, env({ now = 1, me = me, rules = { thunder, protect }, target = edge(22.1) })) -- gap 20.6
assert(a and a.text == "Protect", a and a.text)

-- A weapon skill on yourself (Starlight) by its own targets; a monster skill not on yourself.
local star = { target = "me", when = { kind = "always" }, action = { kind = "ws", name = "Starlight" } }
s = fresh()
a = core.tick(s, env({ now = 1, me = me, rules = { star }, has_ability = { [163] = true } }))
assert(a and a.id == 163 and a.tag == "<me>")
local blade_me = { target = "me", when = { kind = "always" }, action = { kind = "ws", name = "Fast Blade" } }
s = fresh()
assert(core.tick(s, env({ now = 1, me = me, rules = { blade_me } })) == nil)

-- A pet takes only what may go on an NPC (0x40): no Cure on it.
local pet = { hpp = 30, name = "Wyvern", distance = 3 }
local pet_cure = { target = "pet", when = { kind = "hp_below", value = 50 }, action = { kind = "spell", name = "Cure" } }
local pet_mend = { target = "pet", when = { kind = "hp_below", value = 50 }, action = { kind = "spell", name = "Pet Mend" } }
s = fresh()
a = core.tick(s, env({ now = 1, rules = { pet_cure, pet_mend }, pet = pet, known = { [1] = true, [100] = true } }))
assert(a and a.text == "Pet Mend" and a.tag == "<pet>", a and a.text)

-- Best Utsusemi: San .. Ichi by name; ninjutsu only with its tool.
-- (by id: the picker's names are in the client's language, here a Japanese Ni)
local utsu = { target = "me", when = { kind = "always" }, action = { kind = "spell", name = "空蝉の術:弐", best = true } }
local known = { [338] = true, [339] = true, [340] = true }
s = fresh()
a = core.tick(s, env({ now = 1, rules = { utsu }, known = known }))
assert(a and a.id == 340, "San first")
s = fresh()
a = core.tick(s, env({ now = 1, rules = { utsu }, known = known, refuse = { spell340 = true } }))
assert(a and a.id == 339, "then Ni")
s = fresh()
a = core.tick(s, env({ now = 1, rules = { utsu }, known = known, refuse = { spell339 = true, spell340 = true } }))
assert(a and a.id == 338, "San and Ni refused: Ichi")
s = fresh()
assert(core.tick(s, env({ now = 1, rules = { utsu }, known = known, has_tool = function() return false end })) == nil)
assert(core.has_tool(338, function(i) return i == 1179 end, 3) and not core.has_tool(338, function(i) return i == 2972 end, 3))
assert(core.has_tool(338, function(i) return i == 2972 end, 13) and core.has_tool(1, function() return false end, 3))
