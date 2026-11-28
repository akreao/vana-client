-- Which slots a main / sub job pair keeps when its set starts from the old layout (one set for every sub job):
-- another sub job's actions go. A spell neither job learns at any level is another job's; an ability not ours goes
-- only at main level 99 (below that it may be the main job's, not learned yet; the game files name no job for an
-- ability). Weapon skills, pet commands (listed only while their pet is out), items, equipment, commands and typed
-- lines without either stay.

local jobs = require("src.jobs")
local actions = require("src.actions")

local M = {}

-- The jobs and the ability list, and since when they have held: the server's list for new jobs lands after the job
-- change, so abilities are judged only once it has settled.
local SETTLE = 2
local last_sig, since, told = nil, 0, false

local function signature()
  local mj, sj = jobs.current()
  return tostring(mj) .. "/" .. tostring(sj) .. ":" .. table.concat(game.abilities() or {}, ",")
end

-- Every half second (init.lua); true once, when the list has just settled.
function M.watch()
  local sig = signature()
  if sig ~= last_sig then
    last_sig, since, told = sig, clock(), false
    return false
  end
  if not told and clock() - since >= SETTLE then
    told = true
    return true
  end
  return false
end

-- Settled for the jobs and list as they are now: a job change the watch has not seen yet (job_change rebuilds at
-- once) is not, so the old pair's settled list never judges the new pair.
function M.ready() return last_sig ~= nil and last_sig == signature() and clock() - since >= SETTLE end

local function ability_ids()
  local have = {}
  for _, id in ipairs(game.abilities() or {}) do have[id] = true end
  return have
end

-- Whether one macro's action is the pair's.
local function ours(m, main_id, sub_id, level, have)
  local a = actions.of(m)
  if not a then return true end
  if a.kind == "spell" then
    local s = res.spell(a.id)
    if not s or s.kind == 8 or type(s.levels) ~= "table" or next(s.levels) == nil then return true end
    return s.levels[main_id] ~= nil or (sub_id ~= nil and s.levels[sub_id] ~= nil)
  elseif a.kind == "ability" and not a.ws and a.id >= 0x200 and a.id < 0x400 then
    if (level or 0) < 99 or next(have) == nil or not M.ready() then return true end
    return have[a.id] == true
  end
  return true
end

-- keep(main, sub, macro) for data.keep: a group stays while any member is the pair's.
function M.keep(main, sub, macro)
  local main_id, sub_id = jobs.id(main), sub and jobs.id(sub) or nil
  if not main_id then return true end
  local p = game.player() or {}
  local level = p.main_job == main_id and p.main_level or nil
  local have = ability_ids()
  if type(macro.group) == "table" then
    for _, member in ipairs(macro.group) do
      if ours(member, main_id, sub_id, level, have) then return true end
    end
    return #macro.group == 0
  end
  return ours(macro, main_id, sub_id, level, have)
end

return M
