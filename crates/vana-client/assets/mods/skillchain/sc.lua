-- Skillchain rules and data. The server applies the properties, so the game files hold no table of them:
-- data/skillchains.txt comes from LandSandBoat's SQL (`vana-client skillchain-gen`): weapon skill, mob skill (trusts,
-- pets) and blue magic properties and the trusts' skill lists. The chain rules and elements are LSB's battleutils
-- (skillchain_map, FormSkillchain, GetSkillchainMagicElement).

local M = {}

-- A property: LSB's SKILLCHAIN_ELEMENT (1 Transfixion .. 8 Impaction level 1, 9..12 level 2, 13 / 14 Light /
-- Darkness, 15 / 16 their second level). 0 none.
M.NAMES = { "Transfixion", "Compression", "Liquefaction", "Scission", "Reverberation", "Detonation", "Induration",
  "Impaction", "Gravitation", "Distortion", "Fusion", "Fragmentation", "Light", "Darkness", "Light", "Darkness" }

-- The elements a magic burst can use, in GetSkillchainMagicElement's order, as element ids (0 fire, 1 ice, 2 wind,
-- 3 earth, 4 thunder, 5 water, 6 light, 7 dark).
M.ELEMENT_NAMES = { [0] = "Fire", "Ice", "Wind", "Earth", "Thunder", "Water", "Light", "Dark" }
local ELEMENTS = {
  [1] = { 6 }, [2] = { 7 }, [3] = { 0 }, [4] = { 3 }, [5] = { 5 }, [6] = { 2 }, [7] = { 1 }, [8] = { 4 },
  [9] = { 7, 3 }, [10] = { 5, 1 }, [11] = { 6, 0 }, [12] = { 2, 4 },
  [13] = { 6, 0, 2, 4 }, [14] = { 7, 3, 5, 1 }, [15] = { 6, 0, 2, 4 }, [16] = { 7, 3, 5, 1 },
}

function M.elements(p) return ELEMENTS[p] or {} end

-- Light / Darkness of either level as one (their names are the same).
function M.family(p)
  if p == 15 then return 13 end
  if p == 16 then return 14 end
  return p
end

-- GetSkillchainTier: 1 for the level-1 properties, 2, 3 (Light / Darkness), 4.
function M.level(p)
  if p >= 1 and p <= 8 then return 1 end
  if p >= 9 and p <= 12 then return 2 end
  if p == 13 or p == 14 then return 3 end
  if p == 15 or p == 16 then return 4 end
  return 0
end

-- skillchain_map: {the closing skill's property, the resonance, the chain}.
local MAP = {
  { 13, 13, 15 }, { 14, 14, 16 }, { 10, 9, 14 }, { 12, 9, 12 }, { 9, 10, 14 }, { 11, 10, 11 }, { 9, 11, 9 },
  { 12, 11, 13 }, { 10, 12, 10 }, { 11, 12, 13 }, { 4, 1, 10 }, { 8, 3, 11 }, { 2, 6, 9 }, { 5, 7, 12 },
  { 2, 1, 2 }, { 5, 1, 5 }, { 1, 2, 1 }, { 6, 2, 6 }, { 4, 3, 4 }, { 3, 4, 3 }, { 5, 4, 5 }, { 6, 4, 6 },
  { 7, 5, 7 }, { 8, 5, 8 }, { 4, 6, 4 }, { 2, 7, 2 }, { 8, 7, 8 }, { 3, 8, 3 }, { 6, 8, 6 },
}
M.MAP = MAP

-- FormSkillchain: each resonance property in order against each of the closer's; the first pair in the map wins.
function M.form(resonance, closer)
  for _, r in ipairs(resonance) do
    if r ~= 0 then
      for _, c in ipairs(closer) do
        if c ~= 0 then
          for _, e in ipairs(MAP) do
            if e[1] == c and e[2] == r then return e[3] end
          end
        end
      end
    end
  end
  return nil
end

-- "Fusion (Light, Fire)".
function M.chain_text(p)
  local els = {}
  for _, e in ipairs(M.elements(p)) do els[#els + 1] = M.ELEMENT_NAMES[e] end
  return ("%s (%s)"):format(M.NAMES[p] or "", table.concat(els, ", "))
end

-- A skill's properties as text: "Transfixion / Scission".
function M.props_text(props)
  local out = {}
  for _, p in ipairs(props) do
    if p ~= 0 then out[#out + 1] = M.NAMES[p] end
  end
  return table.concat(out, " / ")
end

-- Lower case, letters and digits only: "TRUST_Ayame_UC" and "Ayame (UC)" both give "ayameuc".
function M.fold(name)
  return (name:gsub("^TRUST_", ""):gsub("[^%w]", ""):lower())
end

-- "red_lotus_blade" -> "Red Lotus Blade".
function M.pretty(name)
  local words = {}
  for w in name:gmatch("[^_]+") do words[#words + 1] = w:sub(1, 1):upper() .. w:sub(2) end
  return table.concat(words, " ")
end

-- The data: weapon[id], mob[id], blue[spell] = {props = {p1, p2, p3}, name, skill}; trusts[folded name] = {ids}.
local data
function M.data()
  if data then return data end
  data = { weapon = {}, mob = {}, blue = {}, trusts = {} }
  for line in (fs.read("data/skillchains.txt") or ""):gmatch("[^\n]+") do
    if line:sub(1, 1) ~= "#" then
      local f = {}
      for w in line:gmatch("%S+") do f[#f + 1] = w end
      local function num(k) return math.tointeger(tonumber(f[k])) or 0 end
      local function props(k) return { num(k), num(k + 1), num(k + 2) } end
      if f[1] == "w" then
        data.weapon[num(2)] = { props = props(3), skill = num(7), name = f[8] or "" }
      elseif f[1] == "m" then
        data.mob[num(2)] = { props = props(3), name = f[6] or "" }
      elseif f[1] == "b" then
        data.blue[num(2)] = { props = props(3), name = "" }
      elseif f[1] == "t" then
        local ids = {}
        for k = 3, #f do ids[#ids + 1] = num(k) end
        data.trusts[f[2] or ""] = ids
      end
    end
  end
  return data
end

-- A skill: {kind = "weapon" | "mob" | "blue", id}; its entry and properties.
function M.entry(s)
  local d = M.data()
  if s.kind == "weapon" then return d.weapon[s.id] end
  if s.kind == "mob" then return d.mob[s.id] or (s.id < 256 and d.weapon[s.id] or nil) end
  if s.kind == "blue" then return d.blue[s.id] end
end

function M.props(s)
  local e = M.entry(s)
  return e and e.props or { 0, 0, 0 }
end

function M.has_props(s)
  for _, p in ipairs(M.props(s)) do
    if p ~= 0 then return true end
  end
  return false
end

-- A trust's skills (mob skill ids) by its party name.
function M.trust(name) return M.data().trusts[M.fold(name)] end

function M.key(s) return s.kind .. ":" .. s.id end

return M
