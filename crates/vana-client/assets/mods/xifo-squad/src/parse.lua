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

-- "!squad panel": { mates = { {slot, name, job, level, sub, sub_level, role, mode, out, switches, levels, cooldown,
-- rules} }, camp = {on, puller, paused, pulls}, bots = {names}, bot_puller, caps = {word = true} }
-- rules (servers from v0.3.11, line "rs"): what the mate runs and what its next call will run: {state, why, live,
-- live_name, sel, sel_name, fallback, dropped, live_job}. state: off (not out), set, job, fallback, pending (why: job,
-- choice, gone, edited, role), addon, unknown. caps (line "cap"): what the server can do beyond the first menus.
function M.panel(lines)
  local p = { mates = {}, camp = { on = false, puller = 0, paused = false, pulls = 0 }, bots = {}, caps = {} }
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
    elseif f[1] == "rs" and by_slot[num(f[2])] and #f >= 11 then
      by_slot[num(f[2])].rules = { state = f[3], why = f[4], live = num(f[5]), live_name = f[6], sel = num(f[7]),
        sel_name = f[8], fallback = num(f[9]), dropped = num(f[10]), live_job = f[11] ~= "-" and f[11] or nil }
    elseif f[1] == "cap" then
      for _, word in ipairs(list(f[2])) do p.caps[word] = true end
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

-- "!squad sheet <slot>": { slot (servers from v0.3.11: whose answer this is), name, job, level, sub, sub_level, role, mode, race, switches, jobs = {{job, level}},
-- exp = {job, level, now, need}, vitals = {HP = {now, max}, ...}, stats = {{name, value}}, combat = {{name, value}},
-- haste = {{kind, hundredths}}, gear = {{slot, slot_name, item, id}}, look = {...}, notes = {}, spells = {{name, mp}},
-- abilities = {{name, level}} }
function M.sheet(lines)
  local s = { jobs = {}, vitals = {}, stats = {}, combat = {}, haste = {}, gear = {}, look = {}, notes = {},
    spells = {}, abilities = {} }
  for _, line in ipairs(lines) do
    local f = split(line)
    local k = f[1]
    if k == "slot" then
      s.slot = tonumber(f[2])
    elseif k == "hdr" then
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

-- Why a chosen Rule Set is not running (the server's fallback reasons).
local FALLBACK = { "the chosen set is gone", "that number is now another set", "the chosen set cannot be used here",
  "no set for this job" }
local PENDING = { job = "its job changed", choice = "another set was chosen", gone = "its set was deleted",
  edited = "its set was edited", role = "its role changed" }

local function set_name(num, name)
  if num == 0 then return "its job rules" end
  if name == "?" then return ("set %d (no longer usable)"):format(num) end
  return ("set %d %s"):format(num, name ~= "-" and name or "")
end

-- A mate's rules as one line for the window ("Rules: ..."), or nil when the server does not say.
function M.rules_text(r)
  if not r then return nil end
  local live = set_name(r.live, r.live_name)
  local t
  if r.state == "off" then
    t = "Rules: " .. set_name(r.sel, r.sel_name) .. " at its next call"
  elseif r.state == "set" then
    t = "Rules: " .. live
  elseif r.state == "job" then
    t = "Rules: its job rules"
  elseif r.state == "fallback" then
    t = "Rules: its job rules (" .. (FALLBACK[r.fallback] or "the chosen set could not be used") .. ")"
  elseif r.state == "pending" then
    t = ("Rules: %s now; %s, so the next call changes them (Apply)"):format(live, PENDING[r.why] or "settings changed")
  elseif r.state == "addon" then
    t = "Rules: an add-on runs this mate"
  elseif r.state == "unknown" then
    t = "Rules: not known (called before the server kept track)"
  else
    t = "Rules: " .. tostring(r.state)
  end
  if r.dropped > 0 and r.state ~= "off" then
    t = t .. (" (%d rule%s left out)"):format(r.dropped, r.dropped == 1 and "" or "s")
  end
  return t
end

return M
