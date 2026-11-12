-- The facts the rules read, from the game and from the Skillchain and Timers mods' services (each optional: without
-- one its facts are left out).

local F = {}

-- Monsters' elemental resistance ranks (LSB's data: data/resistances.txt, made by the client's data/resist_gen.py):
-- each species' ranks, and each monster's zone, shown name, species and own ranks. Ranks in the spells' element
-- order (fire .. dark); higher resists more, below 0 is a weakness.
local ELEMENTS = { "fire", "ice", "wind", "earth", "thunder", "water", "light", "dark" }
local monsters
local function ranks_of(zone, name)
  if not monsters then
    monsters = {}
    local species = {}
    for line in (fs.read("data/resistances.txt") or ""):gmatch("[^\n]+") do
      local f = {}
      for w in line:gmatch("%S+") do f[#f + 1] = w end
      if f[1] == "s" and #f == 10 then
        local r = {}
        for k = 1, 8 do r[k] = math.tointeger(tonumber(f[k + 2])) or 0 end
        species[f[2]] = r
      elseif f[1] == "m" and #f >= 4 and tonumber(f[2]) and species[f[4]] then
        local r = {}
        for k = 1, 8 do r[k] = species[f[4]][k] end
        for k = 5, #f do
          local e, v = f[k]:match("^(%a+)=(%-?%d+)$")
          for i, n in ipairs(ELEMENTS) do
            if n == e then r[i] = tonumber(v) end
          end
        end
        monsters[f[2] .. " " .. f[3]:lower()] = r
      end
    end
  end
  local key = tostring(zone) .. " " .. name:gsub("^%s+", ""):gsub("%s+$", ""):gsub(" ", "_"):lower()
  return monsters[key]
end
F.ranks_of = ranks_of

-- Each ninjutsu's tool and the universal tool a NIN main may use instead (LSB's spell_list and HasNinjaTool):
-- {first spell, last spell, tool, universal tool}.
local NINJA_TOOLS = {
  { 318, 318, 2553, 2972 }, { 319, 319, 2555, 2973 }, { 320, 322, 1161, 2971 }, { 323, 325, 1164, 2971 },
  { 326, 328, 1167, 2971 }, { 329, 331, 1170, 2971 }, { 332, 334, 1173, 2971 }, { 335, 337, 1176, 2971 },
  { 338, 340, 1179, 2972 }, { 341, 343, 1182, 2973 }, { 344, 346, 1185, 2973 }, { 347, 349, 1188, 2973 },
  { 350, 352, 1191, 2973 }, { 353, 354, 1194, 2972 }, { 505, 505, 8803, 2972 }, { 506, 506, 8804, 2972 },
  { 507, 507, 2642, 2972 }, { 508, 508, 2643, 2973 }, { 509, 509, 2644, 2972 }, { 510, 510, 2970, 2972 },
}

-- The tool is in the inventory, or the universal one with NIN (13) the main job; true for other spells.
function F.has_tool(spell, held, main)
  for _, t in ipairs(NINJA_TOOLS) do
    if spell >= t[1] and spell <= t[2] then return held(t[3]) or (main == 13 and held(t[4])) end
  end
  return true
end

-- A white-magic main (WHM, PLD) is offered no black magic from its support job, a black-magic main (BLM, DRK) no
-- white; other mains take both.
function F.other_school(main, kind)
  if main == 3 or main == 7 then return kind == 2 end
  if main == 4 or main == 8 then return kind == 1 end
  return false
end

-- Weather ids to the element they strengthen; each day's element (Firesday .. Darksday).
local WEATHER = { [4] = 0, [5] = 0, [6] = 5, [7] = 5, [8] = 3, [9] = 3, [10] = 2, [11] = 2, [12] = 1, [13] = 1,
  [14] = 4, [15] = 4, [16] = 6, [17] = 6, [18] = 7, [19] = 7 }
local DAY_ELEMENTS = { [0] = 0, 3, 5, 2, 1, 4, 6, 7 }

local function call(mod, name, ...)
  local ok, v = pcall(mods.call, mod, name, ...)
  if ok then return v end
end

-- The job ability rows: kind 1 and the lists job abilities open (the runes, Vallation, Lunge, the waltzes ...).
local sub_kinds
local function ability_kinds()
  if not sub_kinds then
    local kinds, seen, any = { 1 }, { [1] = true }, false
    for id = 0x200, 0x7ff do
      local a = res.ability(id)
      if a then
        any = true
        if a.kind == 1 and a.sub ~= 0 and not seen[a.sub] then
          seen[a.sub] = true
          kinds[#kinds + 1] = a.sub
        end
      end
    end
    if not any then return { 1 } end
    sub_kinds = kinds
  end
  return sub_kinds
end

-- The facts now.
function F.gather()
  local p = game.player()
  local f = { party = {}, own = {}, spells = {}, abilities = {}, weapon_skills = {}, partners = {} }
  local slots = game.party()
  local mine
  for _, m in ipairs(slots) do
    if m.id == p.id then mine = m end
  end
  local own_ids = {}
  for _, b in ipairs(game.buffs()) do
    own_ids[#own_ids + 1] = b.id
    f.own[#f.own + 1] = { b.id, b.remaining }
  end
  if p.hp and p.main_job then
    f.party[1] = { name = p.name, me = true, here = true, hp = p.hp, hp_max = mine and mine.hp_max or p.hp_max,
      job = p.main_job, effects = own_ids }
  end
  for _, m in ipairs(slots) do
    if m.party == 0 and m.slot > 0 then
      f.party[#f.party + 1] = { name = m.name, me = false, here = m.in_zone and m.zone == p.zone, hp = m.hp,
        hp_max = m.hp_max or 0, job = m.main_job, effects = m.buffs or {} }
    end
  end
  f.main = { p.main_job or 0, p.main_level or 0 }
  f.sub = { p.sub_job or 0, p.sub_level or 0 }
  f.mpp, f.tp = p.mpp or 0, p.tp or 0
  f.engaged = p.status == 1
  -- Spells castable now (ninjutsu only with its tool in the inventory, the server's check).
  local inv = {}
  for _, it in ipairs(game.items(0)) do
    if it.count > 0 then inv[it.id] = true end
  end
  local function held(item) return inv[item] == true end
  local main = p.main_job or 0
  for _, id in ipairs(game.spells()) do
    if id < 1024 and game.usable("spell", id) and F.has_tool(id, held, main) then
      local s = res.spell(id)
      if s and not F.other_school(main, s.kind) then
        f.spells[#f.spells + 1] = { id = id, name = s.name or "", skill = s.skill, element = s.element, mp = s.mp,
          targets = s.targets, area = s.area == true }
      end
    end
  end
  -- Job abilities and weapon skills usable now, in the lists' order.
  local known = game.abilities()
  for _, kind in ipairs(ability_kinds()) do
    for _, id in ipairs(known) do
      local a = res.ability(id)
      if a and a.kind == kind and game.usable("ability", id) then
        f.abilities[#f.abilities + 1] = { index = id, name = a.name or "" }
      end
    end
  end
  local ws = {}
  for _, id in ipairs(known) do
    local a = res.ability(id)
    if a and a.kind == 3 and game.usable("ability", id) then ws[#ws + 1] = { id, a.name or "" } end
  end
  local ids = {}
  for _, x in ipairs(ws) do ids[#ids + 1] = x[1] end
  local props = call("skillchain", "weapon_skills", ids) or {}
  for _, x in ipairs(ws) do
    local e = props[x[1]]
    f.weapon_skills[#f.weapon_skills + 1] = { index = x[1], name = x[2], props = e and e.props or { 0, 0, 0 },
      power = e and e.power or 0 }
  end
  -- The monster followed, who it goes for, its open resonance, the partners (the Skillchain mod's).
  local sc = call("skillchain", "facts") or {}
  local target_id = sc.focus
  if not target_id and p.target then
    local e = game.entity(p.target)
    if e and e.kind == "mob" then target_id = p.target end
  end
  local e = target_id and game.entity(target_id)
  if e then
    local ours = {}
    for _, x in ipairs(call("timers", "effects_on", target_id) or {}) do ours[#ours + 1] = x.status end
    local hit = sc.hitting
    local hitting
    if hit and hit ~= p.id then
      for _, m in ipairs(slots) do
        if m.party == 0 and m.id == hit then hitting = m.name end
      end
    end
    f.target = { name = e.name, hpp = e.hpp, ours = ours, hitting = hitting, hitting_me = hit == p.id,
      ranks = ranks_of(p.zone, e.name) }
  end
  f.resonance = sc.resonance
  for _, x in ipairs(sc.partners or {}) do
    local skills = {}
    for _, s in ipairs(x.skills) do skills[#skills + 1] = { s.name, s.props } end
    f.partners[#f.partners + 1] = { name = x.name, tp = x.tp, skills = skills }
  end
  local t = game.time()
  f.day_element = t and t.weekday and DAY_ELEMENTS[t.weekday] or nil
  f.weather_element = WEATHER[game.weather() or -1]
  local pet = p.pet and game.entity(p.pet)
  if pet then
    -- The luopans' models.
    f.pet = { name = pet.name, hpp = pet.hpp, luopan = (pet.model or 0) >= 0xb22 and (pet.model or 0) <= 0xb31 }
  end
  -- A monster our party claimed is about: a fight is on, whoever it is on.
  local party = { [p.id] = true }
  for _, m in ipairs(slots) do
    if m.party == 0 then party[m.id] = true end
  end
  f.party_fight = false
  for _, x in ipairs(game.entities({ kinds = { "mob" } })) do
    if x.hpp > 0 and x.claim ~= 0 and party[x.claim] then f.party_fight = true end
  end
  return f
end

return F
