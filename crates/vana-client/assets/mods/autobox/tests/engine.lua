-- Autobox's engine: the first rule that holds and that the game would take is done, on the right target; it waits
-- while moving, casting or after an action; best tiers, best weapon skills, party order and claimed targets.

local core = require("src.core")
local presets = require("src.presets")

local SPELLS = {
  [1] = { name = "Cure", targets = 0x1F }, [2] = { name = "Cure II", targets = 0x1F },
  [3] = { name = "Cure III", targets = 0x1F }, [43] = { name = "Protect", targets = 0x1 },
  [109] = { name = "Refresh", targets = 0x1 }, [164] = { name = "Thunder", targets = 0x20 },
}
local ABILITIES = { [32] = { name = "Fast Blade" }, [42] = { name = "Savage Blade" }, [520] = { name = "Berserk", targets = 1 } }
local STATUS = { protect = { 40 }, refresh = { 43 }, poison = { 3 }, berserk = { 56 } }

local function env(o)
  local e = {
    now = o.now or 10, on = true,
    me = o.me or { id = 1, pos = { x = 0, z = 0 }, hpp = 100, mpp = 100, tp = 0, status = 0 },
    buffs = o.buffs or {}, known = o.known or { [1] = true, [2] = true, [43] = true, [109] = true, [164] = true },
    party = o.party or {}, ours = o.ours or { [1] = true }, target = o.target, pet = nil,
    entity = function(id) return (o.near or {})[id] end, rules = o.rules,
    casting = o.casting or false, event = false,
    usable = function(kind, id) return not (o.refuse or {})[kind .. id] end,
    spell = function(id) return SPELLS[id] end,
    ability = function(id) return ABILITIES[id] end,
    spell_id = function(n) for id, r in pairs(SPELLS) do if r.name:lower() == n:lower() then return id end end end,
    ability_id = function(n) for id, r in pairs(ABILITIES) do if r.name:lower() == n:lower() then return id end end end,
    status_ids = function(n) return STATUS[n:lower()] end,
  }
  return e
end

local cure = { target = "party", when = { kind = "hp_below", value = 60 }, action = { kind = "spell", name = "Cure", best = true } }
local protect = { target = "me", when = { kind = "missing", value = "Protect" }, action = { kind = "spell", name = "Protect" } }

-- Nobody hurt: the first rule passes over, Protect goes on us.
local s = core.new()
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
s = core.new()
a = core.tick(s, env({ now = 1, rules = { cure }, party = party, near = near }))
assert(a and a.id == 2 and a.tag == "<p2>" and a.who == "Bo", "Cure II on the lowest in range")
-- Cure II refused (no MP): Cure.
s = core.new()
a = core.tick(s, env({ now = 1, rules = { cure }, party = party, near = near, refuse = { spell2 = true } }))
assert(a and a.id == 1, "falls back a tier")

-- Moving or casting holds.
s = core.new()
core.tick(s, env({ now = 0, buffs = { [40] = true }, rules = { protect } }))
local moved = { id = 1, pos = { x = 3, z = 0 }, hpp = 100, mpp = 100, tp = 0, status = 0 }
assert(core.tick(s, env({ now = 1, me = moved, rules = { protect } })) == nil and s.holding == "moving")
assert(core.tick(s, env({ now = 2, me = moved, casting = true, rules = { protect } })) == nil and s.holding == "casting")
assert(core.tick(s, env({ now = 3, me = moved, rules = { protect } })) ~= nil, "standing still again")

-- Weapon skills: only on a monster we claimed, at 1000 TP; Best picks the first the game takes.
local ws = presets.for_job("WAR")[1]
local me = { id = 1, pos = { x = 0, z = 0 }, hpp = 100, mpp = 0, tp = 1200, status = 1 }
local mob = { kind = "mob", hpp = 80, claim = 1, name = "Goblin" }
s = core.new()
a = core.tick(s, env({ now = 1, me = me, rules = { ws }, target = mob, refuse = { ability42 = true } }))
assert(a and a.kind == "ability" and a.id == 32 and a.tag == "<t>", "best weapon skill the game takes")
s = core.new()
assert(core.tick(s, env({ now = 1, me = me, rules = { ws }, target = { kind = "mob", hpp = 80, claim = 99 } })) == nil,
  "someone else's monster")

-- A status condition on a monster never holds; an off rule is skipped.
local odd = { target = "target", when = { kind = "missing", value = "poison" }, action = { kind = "spell", name = "Thunder" } }
s = core.new()
assert(core.tick(s, env({ now = 1, me = me, rules = { odd }, target = mob })) == nil)
local off = { on = false, target = "me", when = { kind = "always" }, action = { kind = "spell", name = "Refresh" } }
s = core.new()
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
