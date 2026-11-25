-- The XI: Forever Offline server's answers to "!squad panel" and "!squad sheet <slot>", read into tables. Each line
-- comes without its "squadpanel:" / "squadsheet:" tag, its fields split at "|". Lines this mod does not use are
-- skipped, so a newer server's extra lines do no harm.

local M = {}

M.ROLES = { "auto", "tank", "damage", "healer", "support" }
M.MODES = { "balanced", "aggressive", "conserve" }
-- The on/off switches, by their bit in the mask (bit set = allowed).
M.SWITCHES = { "melee", "ranged", "ja", "magic" }
M.JOBS = { "WAR", "MNK", "WHM", "BLM", "RDM", "THF", "PLD", "DRK", "BST", "BRD", "RNG", "SAM", "NIN", "DRG", "SMN",
  "BLU", "COR", "PUP" }

local function split(line)
  local out = {}
  for field in (line .. "|"):gmatch("([^|]*)|") do out[#out + 1] = field end
  return out
end

local function num(v) return tonumber(v) or 0 end

local function list(v)
  local out = {}
  if v and v ~= "-" and v ~= "" then
    for word in v:gmatch("[^,]+") do out[#out + 1] = word end
  end
  return out
end

function M.has_switch(mask, n) return (mask >> (n - 1)) & 1 == 1 end

-- "!squad panel": { mates = { {slot, name, job, level, sub, sub_level, role, mode, out, switches, levels, cooldown} },
-- camp = {on, puller, paused, pulls}, bots = {names}, bot_puller }
function M.panel(lines)
  local p = { mates = {}, camp = { on = false, puller = 0, paused = false, pulls = 0 }, bots = {} }
  local by_slot = {}
  for _, line in ipairs(lines) do
    local f = split(line)
    local slot = tonumber(f[1])
    if slot and #f >= 10 then
      local m = { slot = slot, name = f[2], job = f[3], level = num(f[4]), sub = f[5] ~= "-" and f[5] or nil,
        sub_level = num(f[6]), role = f[7], mode = f[8], out = f[9] == "1", switches = num(f[10]), levels = {},
        cooldown = 0 }
      p.mates[#p.mates + 1] = m
      by_slot[slot] = m
    elseif f[1] == "jl" and by_slot[num(f[2])] then
      for _, e in ipairs(list(f[3])) do
        local job, lv = e:match("^(%u+):(%d+)$")
        if job then by_slot[num(f[2])].levels[job] = tonumber(lv) end
      end
    elseif f[1] == "cd" and by_slot[num(f[2])] then
      by_slot[num(f[2])].cooldown = num(f[3])
    elseif f[1] == "camp" then
      p.camp = { on = f[2] == "1", puller = num(f[3]), paused = f[4] == "1", pulls = num(f[5]) }
    elseif f[1] == "campbots" then
      p.bots = list(f[2])
      p.bot_puller = f[3] ~= "-" and f[3] or nil
    end
  end
  table.sort(p.mates, function(a, b) return a.slot < b.slot end)
  return p
end

-- "!squad sheet <slot>": { name, job, level, sub, sub_level, role, mode, race, switches, jobs = {{job, level}},
-- exp = {job, level, now, need}, vitals = {HP = {now, max}, ...}, stats = {{name, value}}, combat = {{name, value}},
-- haste = {{kind, hundredths}}, gear = {{slot, slot_name, item, id}}, look = {...}, notes = {}, spells = {{name, mp}},
-- abilities = {{name, level}} }
function M.sheet(lines)
  local s = { jobs = {}, vitals = {}, stats = {}, combat = {}, haste = {}, gear = {}, look = {}, notes = {},
    spells = {}, abilities = {} }
  for _, line in ipairs(lines) do
    local f = split(line)
    local k = f[1]
    if k == "hdr" then
      s.name, s.job, s.level, s.sub, s.sub_level = f[2], f[3], num(f[4]), f[5] ~= "-" and f[5] or nil, num(f[6])
      s.role, s.mode, s.race, s.switches = f[7], f[8], f[9], num(f[10])
    elseif k == "job" then
      s.jobs[#s.jobs + 1] = { job = f[2], level = num(f[3]) }
    elseif k == "exp" then
      s.exp = { job = f[2], level = num(f[3]), now = num(f[4]), need = num(f[5]) }
    elseif k == "vital" then
      s.vitals[f[2]] = { now = num(f[3]), max = num(f[4]) }
    elseif k == "stat" then
      s.stats[#s.stats + 1] = { name = f[2], value = num(f[3]) }
    elseif k == "combat" then
      s.combat[#s.combat + 1] = { name = f[2], value = num(f[3]) }
    elseif k == "haste" then
      s.haste[#s.haste + 1] = { kind = f[2], value = num(f[3]) }
    elseif k == "eq" or k == "ls" then
      local into = k == "eq" and s.gear or s.look
      into[#into + 1] = { slot = num(f[2]), slot_name = f[3], item = f[4], id = num(f[5]) }
    elseif k == "note" then
      s.notes[#s.notes + 1] = table.concat(f, "|", 2)
    elseif k == "spell" then
      s.spells[#s.spells + 1] = { name = f[2], mp = num(f[3]) }
    elseif k == "ability" then
      s.abilities[#s.abilities + 1] = { name = f[2], level = num(f[3]) }
    end
  end
  table.sort(s.gear, function(a, b) return a.slot < b.slot end)
  table.sort(s.look, function(a, b) return a.slot < b.slot end)
  return s
end

return M
