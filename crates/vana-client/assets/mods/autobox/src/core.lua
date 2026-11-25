-- Autobox's engine: ordered rules for your own character. Each tick walks the main job's rules top to bottom; the
-- first rule whose target is there, whose conditions hold and whose action the game would take now is the one done.
-- Nothing here touches the game: init.lua hands in what it read (env) and carries out what comes back.

local M = {}

M.TARGETS = { "me", "party", "target", "pet" }
M.TARGET_LABELS = { me = "Me", party = "Party member", target = "Battle target", pet = "Pet" }

-- Condition kinds, in the editor's order; `value` says what the value is.
M.CONDITIONS = {
  { kind = "always", label = "Always" },
  { kind = "hp_below", label = "HP below %", value = "number" },
  { kind = "hp_above", label = "HP above %", value = "number" },
  { kind = "mp_below", label = "MP below %", value = "number" },
  { kind = "my_mp_above", label = "My MP above %", value = "number" },
  { kind = "tp_at_least", label = "My TP at least", value = "number" },
  { kind = "missing", label = "Lacks status", value = "status" },
  { kind = "has", label = "Has status", value = "status" },
  { kind = "engaged", label = "I am engaged" },
  { kind = "not_engaged", label = "I am not engaged" },
}
M.CONDITION_BY_KIND = {}
for i, c in ipairs(M.CONDITIONS) do
  c.index = i
  M.CONDITION_BY_KIND[c.kind] = c
end

M.ACTIONS = { "spell", "ability", "ws", "ranged", "attack" }
M.ACTION_LABELS = { spell = "Spell", ability = "Job ability", ws = "Weapon skill", ranged = "Ranged attack",
  attack = "Attack" }

-- "Best" weapon skill: the first of these the game would take now (it checks the weapon, the skill and the TP).
-- Our own pick of the level 75 era's strongest per weapon, strongest first.
M.BEST_WS = {
  "Asuran Fists", "Dragon Kick", "Raging Fists", "Howling Fist", "Combo",
  "Evisceration", "Dancing Edge", "Shark Bite", "Viper Bite", "Wasp Sting",
  "Savage Blade", "Swift Blade", "Vorpal Blade", "Red Lotus Blade", "Fast Blade",
  "Ground Strike", "Spinning Slash", "Sickle Moon", "Power Slash", "Hard Slash",
  "Rampage", "Calamity", "Mistral Axe", "Raging Axe",
  "Steel Cyclone", "Raging Rush", "Full Break", "Keen Edge", "Shield Break",
  "Spiral Hell", "Guillotine", "Cross Reaper", "Nightmare Scythe", "Slice",
  "Penta Thrust", "Wheeling Thrust", "Skewer", "Double Thrust",
  "Blade: Ku", "Blade: Jin", "Blade: Ten", "Blade: Rin",
  "Tachi: Gekko", "Tachi: Kasha", "Tachi: Yukikaze", "Tachi: Jinpu", "Tachi: Enpi",
  "Black Halo", "Judgment", "Hexa Strike", "True Strike", "Skullbreaker", "Shining Strike",
  "Retribution", "Full Swing", "Spirit Taker", "Shell Crusher", "Heavy Swing",
  "Sidewinder", "Arching Arrow", "Piercing Arrow", "Flaming Arrow",
  "Slug Shot", "Detonator", "Sniper Shot", "Hot Shot",
}

-- How long nothing else is tried after an action (s): a spell's cast bar shows by then and holds the engine itself.
M.AFTER_ACTION = 1.5
-- How long the same rule waits before it fires again (s), so a buff that is landing is not cast twice.
M.RULE_REST = 4
M.STILL = 0.35
M.ZONE_REST = 4
M.SPELL_RANGE = 20.5

function M.new()
  return { hold_until = 0, rule_rest = {}, last_pos = nil, still_since = -math.huge, zone_until = 0, holding = nil, last = nil }
end

function M.zoned(s, now)
  s.zone_until = now + M.ZONE_REST
  s.rule_rest = {}
end

function M.condition_text(c)
  if not c or c.kind == "always" or not M.CONDITION_BY_KIND[c.kind] then return c and c.kind == "always" and "Always" or "" end
  local d = M.CONDITION_BY_KIND[c.kind]
  if d.value == "number" then return d.label:gsub(" %%$", "") .. " " .. tostring(c.value or "?") .. (d.label:find("%%") and "%" or "") end
  if d.value == "status" then return d.label .. " " .. tostring(c.value or "?") end
  return d.label
end

function M.action_text(a)
  if not a then return "" end
  if a.kind == "ranged" or a.kind == "attack" then return M.ACTION_LABELS[a.kind] end
  if a.kind == "ws" and a.best then return "Best weapon skill" end
  return (a.best and "Best " or "") .. tostring(a.name or "?")
end

local ROMAN = { "", "II", "III", "IV", "V", "VI" }
local ninjutsu_family

-- A spell family's ids, highest tier first: "Cure" (or "Cure II") -> Cure VI .. Cure; a ninjutsu (tiers Ichi, Ni,
-- San, by id in any language) -> San .. Ichi.
local function family(spell_id, name)
  local nin = ninjutsu_family(spell_id(name))
  if nin then return nin end
  local base
  for t = #ROMAN, 2, -1 do
    base = name:match("^(.-) " .. ROMAN[t] .. "$")
    if base then name = base break end
  end
  local out = {}
  for t = #ROMAN, 1, -1 do
    local id = spell_id(t == 1 and name or (name .. " " .. ROMAN[t]))
    if id then out[#out + 1] = id end
  end
  return out
end

-- The game's spell range classes in yalms (res.spell's `range` indexes it; 255: no limit).
local RANGES = { 0, 1, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 20, 25, 30, 255 }

-- Each ninjutsu's tool and the universal tool a NIN main may use instead (LSB's spell_list and HasNinjaTool):
-- {first spell, last spell, tool, universal tool}.
local NINJA_TOOLS = {
  { 318, 318, 2553, 2972 }, { 319, 319, 2555, 2973 }, { 320, 322, 1161, 2971 }, { 323, 325, 1164, 2971 },
  { 326, 328, 1167, 2971 }, { 329, 331, 1170, 2971 }, { 332, 334, 1173, 2971 }, { 335, 337, 1176, 2971 },
  { 338, 340, 1179, 2972 }, { 341, 343, 1182, 2973 }, { 344, 346, 1185, 2973 }, { 347, 349, 1188, 2973 },
  { 350, 352, 1191, 2973 }, { 353, 354, 1194, 2972 }, { 505, 505, 8803, 2972 }, { 506, 506, 8804, 2972 },
  { 507, 507, 2642, 2972 }, { 508, 508, 2643, 2973 }, { 509, 509, 2644, 2972 }, { 510, 510, 2970, 2972 },
}

-- A ninjutsu's tiers by id, highest first (each tool's spells are one family's tiers, Ichi first), or nil.
ninjutsu_family = function(id)
  if not id then return nil end
  for _, t in ipairs(NINJA_TOOLS) do
    if id >= t[1] and id <= t[2] and t[2] > t[1] then
      local out = {}
      for i = t[2], t[1], -1 do out[#out + 1] = i end
      return out
    end
  end
end

-- Whether the spell's tool is in the inventory (or the universal one with NIN, job 13, the main); true for others.
function M.has_tool(spell, held, main)
  for _, t in ipairs(NINJA_TOOLS) do
    if spell >= t[1] and spell <= t[2] then return held(t[3]) or (main == 13 and held(t[4])) end
  end
  return true
end

local function moving(s, me, now)
  local p = me.pos
  if not p then return false end
  local l = s.last_pos
  s.last_pos = { x = p.x, z = p.z }
  -- The first position seen (at the start, or after switching on) is only a baseline: one more tick tells.
  if not l then return true end
  if math.abs(l.x - p.x) > 0.05 or math.abs(l.z - p.z) > 0.05 then s.still_since = now end
  return now - s.still_since < M.STILL
end

-- The flat distance from you to an entity less both model radii (the arrow's measure).
local function gap(env, e)
  local me = env.me
  if not (e and e.pos and me.pos) then return nil end
  local dx, dz = e.pos.x - me.pos.x, e.pos.z - me.pos.z
  return math.sqrt(dx * dx + dz * dz) - (e.hitbox or 0) - (env.my_hitbox or 0)
end

-- The candidates a rule's target names, as {tag, hpp, mpp, buffs (a set of status ids, or nil when unknown), kind}.
local function candidates(rule, env)
  local me = env.me
  if rule.target == "me" then
    return { { tag = "<me>", hpp = me.hpp, mpp = me.mpp, buffs = env.buffs, kind = "me" } }
  elseif rule.target == "party" then
    local out = {}
    for _, m in ipairs(env.party or {}) do
      if m.party == 0 and m.in_zone and (m.hpp or 0) > 0 then
        local e = m.slot ~= 0 and env.entity and env.entity(m.id) or nil
        if m.slot == 0 or (e and e.distance <= M.SPELL_RANGE) then
          local set = {}
          if m.slot == 0 then set = env.buffs else for _, b in ipairs(m.buffs or {}) do set[b] = true end end
          out[#out + 1] = { tag = m.slot == 0 and "<me>" or ("<p" .. m.slot .. ">"), hpp = m.hpp, mpp = m.mpp,
            buffs = set, kind = m.slot == 0 and "me" or "party", name = m.name }
        end
      end
    end
    table.sort(out, function(a, b) return a.hpp < b.hpp end)
    return out
  elseif rule.target == "target" then
    local t = env.target
    if not t or t.kind ~= "mob" or (t.hpp or 0) <= 0 then return {} end
    if t.claim ~= 0 and not env.ours[t.claim] then return {} end
    if t.claim == 0 and not (me.status == 1) then return {} end
    return { { tag = "<t>", hpp = t.hpp, kind = "mob", name = t.name, distance = t.distance, gap = gap(env, t) } }
  elseif rule.target == "pet" then
    local p = env.pet
    if not p or (p.hpp or 0) <= 0 then return {} end
    return { { tag = "<pet>", hpp = p.hpp, mpp = p.mpp, kind = "pet", name = p.name, distance = p.distance,
      gap = gap(env, p) } }
  end
  return {}
end

local function holds(c, cand, env)
  if not c or c.kind == "always" then return true end
  local me, v = env.me, tonumber(c.value)
  if c.kind == "hp_below" then return v ~= nil and cand.hpp ~= nil and cand.hpp < v end
  if c.kind == "hp_above" then return v ~= nil and cand.hpp ~= nil and cand.hpp > v end
  if c.kind == "mp_below" then return v ~= nil and cand.mpp ~= nil and cand.mpp < v end
  if c.kind == "my_mp_above" then return v ~= nil and (me.mpp or 0) > v end
  if c.kind == "tp_at_least" then return v ~= nil and (me.tp or 0) >= v end
  if c.kind == "engaged" then return me.status == 1 end
  if c.kind == "not_engaged" then return me.status ~= 1 end
  if c.kind == "missing" or c.kind == "has" then
    -- A monster's effects are not known to the client: such a rule never holds.
    if not cand.buffs then return false end
    local ids = env.status_ids(c.value or "")
    if not ids then return false end
    local found = false
    for _, id in ipairs(ids) do
      if cand.buffs[id] then found = true end
    end
    return (c.kind == "has") == found
  end
  return false
end

-- Whether a spell or ability can go on that candidate by its valid-target bits, as the game's target prompt takes
-- them (target.rs SUB_FLAGS, FUN_10159170): yourself by 1 self, 4 party or 0x10 PC; a party member by 4 or 0x10; a
-- monster by 0x20; your pet (an NPC kind) by 0x40 only (0x80 leaves pets out).
local function fits(targets, cand)
  targets = targets or 0
  if cand.kind == "me" then return targets & 0x15 ~= 0 end
  if cand.kind == "party" then return targets & 0x14 ~= 0 end
  if cand.kind == "mob" then return targets & 0x20 ~= 0 end
  if cand.kind == "pet" then return targets & 0x40 ~= 0 end
  return false
end

-- Whether the spell reaches the candidate, as the game's target arrow tells it (target.rs range_tint, FUN_10150610):
-- out of reach past range + 0.5 yalms of flat distance less both model radii (`gap`), or at 25 yalms and more. One
-- without a known gap (you, a party member already within SPELL_RANGE) counts as reached.
local function reaches(r, cand)
  local reach = RANGES[((r.range or 15) & 0xf) + 1]
  if reach >= 255 or not cand.gap then return true end
  return cand.gap <= reach + 0.5 and (cand.distance or 0) < 25
end


-- What the rule's action is on this candidate now: {kind, id, tag, text}, or nil.
-- Status effects that stop actions, by id as the client's menus check them: sleep, petrification, stun, charm, terror,
-- lullaby... stop everything; silence and mute spells; amnesia abilities and weapon skills.
local HALTED = { 2, 7, 10, 14, 17, 19, 28, 193 }
local NO_SPELLS, NO_ABILITIES = { 6, 29 }, { 16 }

local function blocked(env, ids)
  for _, list in ipairs({ HALTED, ids }) do
    for _, id in ipairs(list) do
      if env.buffs[id] then return true end
    end
  end
  return false
end

local function resolve(rule, cand, env)
  local a = rule.action or {}
  local only = a.kind == "spell" and NO_SPELLS or ((a.kind == "ability" or a.kind == "ws") and NO_ABILITIES or {})
  if blocked(env, only) then return nil end
  if a.kind == "ranged" then
    if cand.kind ~= "mob" or not env.can_shoot then return nil end
    return { kind = "ranged", tag = cand.tag, text = "Ranged attack" }
  elseif a.kind == "attack" then
    if cand.kind ~= "mob" or env.me.status == 1 then return nil end
    return { kind = "attack", tag = cand.tag, text = "Attack" }
  elseif a.kind == "spell" then
    local ids = a.best and family(env.spell_id, a.name or "") or { env.spell_id(a.name or "") }
    for _, id in ipairs(ids) do
      local r = env.spell(id)
      if env.known[id] and r and fits(r.targets, cand) and reaches(r, cand) and env.usable("spell", id)
        and (not env.has_tool or env.has_tool(id)) then
        return { kind = "spell", id = id, tag = cand.tag, text = r.name }
      end
    end
  elseif a.kind == "ability" then
    local id = env.ability_id(a.name or "")
    local r = id and env.has_ability[id] and env.ability(id)
    if r and fits(r.targets, cand) and env.usable("ability", id) then
      return { kind = "ability", id = id, tag = cand.tag, text = r.name }
    end
  elseif a.kind == "ws" then
    if (env.me.tp or 0) < 1000 then return nil end
    local names = a.best and M.BEST_WS or { a.name or "" }
    for _, n in ipairs(names) do
      local id = env.ability_id(n)
      -- by the skill's own targets: most go on a monster, a few on yourself (Starlight, Moonlight)
      local r = id and env.ability(id)
      if id and id < 512 and env.has_ability[id] and fits(r and r.targets or 0x20, cand) and env.usable("ability", id) then
        return { kind = "ability", id = id, tag = cand.tag, text = n }
      end
    end
  end
  return nil
end

-- One tick: returns the action to carry out ({kind, id, tag, text, rule}) or nil; s.holding says why nothing runs.
function M.tick(s, env)
  local now, me = env.now, env.me
  s.holding = nil
  local still = not moving(s, me, now)
  if not env.on then s.holding = "off" return nil end
  if now < s.zone_until then s.holding = "zoning" return nil end
  if me.status == 2 or me.status == 3 or (me.hpp or 0) <= 0 then s.holding = "K.O." return nil end
  if me.status == 5 then s.holding = "riding" return nil end
  if env.event then s.holding = "event" return nil end
  if env.casting then s.holding = "casting" return nil end
  if not still then s.holding = "moving" return nil end
  if now < s.hold_until then s.holding = "waiting" return nil end
  for i, rule in ipairs(env.rules or {}) do
    if rule.on ~= false and now >= (s.rule_rest[i] or 0) then
      for _, cand in ipairs(candidates(rule, env)) do
        if holds(rule.when, cand, env) and holds(rule.when2, cand, env) then
          local act = resolve(rule, cand, env)
          if act then
            act.rule = i
            act.who = cand.name or (cand.kind == "me" and "me") or cand.tag
            s.hold_until = now + M.AFTER_ACTION
            s.rule_rest[i] = now + M.RULE_REST
            s.last = act
            return act
          end
        end
      end
    end
  end
  return nil
end

-- A rule checked for saving: nil and the reason when something is missing.
function M.check(rule)
  if not M.TARGET_LABELS[rule.target or ""] then return nil, "no target" end
  for _, c in ipairs({ rule.when, rule.when2 }) do
    local d = c and M.CONDITION_BY_KIND[c.kind]
    if c and not d then return nil, "unknown condition" end
    if d and d.value == "number" and not tonumber(c.value) then return nil, d.label .. ": a number" end
    if d and d.value == "status" and (c.value or "") == "" then return nil, d.label .. ": a status name" end
  end
  local a = rule.action or {}
  if not M.ACTION_LABELS[a.kind or ""] then return nil, "no action" end
  if (a.kind == "spell" or a.kind == "ability" or (a.kind == "ws" and not a.best)) and (a.name or "") == "" then
    return nil, M.ACTION_LABELS[a.kind] .. ": a name"
  end
  return rule
end

return M
