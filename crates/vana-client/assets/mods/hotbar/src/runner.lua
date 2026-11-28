-- Runs a slot. One action (a spell, ability, weapon skill, pet command, item, piece of equipment or system command)
-- goes straight to the game's action; a macro's lines run as typed commands: "/wait 2" waits, a trailing "<wait 3>"
-- waits after its line. A macro still running ignores another press.

local data = require("src.data")
local found = require("src.actions")

local M = {}

-- System commands that are game actions; the rest run as their command line.
local COMMAND_ACTION = {
  ["/attack"] = function(t) actions.attack(t) end,
  ["/attackoff"] = function() actions.disengage() end,
  ["/ra"] = function(t) actions.ranged(t) end,
  ["/assist"] = function(t) actions.assist(t) end,
  ["/heal"] = function() actions.heal() end,
  ["/dismount"] = function() actions.dismount() end,
}

-- The game's tag for a stored target ("t" -> "<t>"), or nil for none.
local function tag(target)
  target = data.trim(target)
  return target ~= "" and ("<" .. target .. ">") or nil
end

-- A slot that is one action and nothing else (no lines after it, no line of its own): runs it and returns true.
local function direct(macro)
  local a = type(macro.action) == "table" and macro.action or nil
  if not a or data.trim(a.line) ~= "" or #(macro.lines or {}) > 1 then return false end
  if a.kind == "cmd" then
    local run = COMMAND_ACTION[data.trim(a.name)]
    if not run then return false end
    run(tag(a.target))
    return true
  end
  local hit = found.of(macro)
  if not hit then return false end
  if (a.kind == "ma" or a.kind == "trust") and hit.kind == "spell" then
    actions.cast(hit.id, tag(a.target))
  elseif (a.kind == "ja" or a.kind == "ws" or a.kind == "pet") and hit.kind == "ability" then
    actions.ability(hit.id, tag(a.target))
  elseif a.kind == "item" and hit.kind == "item" then
    actions.use_item(hit.id, tag(a.target))
  elseif a.kind == "equip" and hit.kind == "item" then
    actions.equip(data.trim(a.target), hit.id)
  else
    return false
  end
  return true
end

local running = {}
local MAX_WAIT = 60

local function clamp_wait(text)
  local n = tonumber(text) or 0
  if n < 0 then
    n = 0
  end
  if n > MAX_WAIT then
    n = MAX_WAIT
  end
  return n
end

-- -> "skip" | "wait", seconds | "cmd", line, seconds_to_wait_after?
function M.parse_line(line)
  line = tostring(line or ""):gsub("^%s+", ""):gsub("%s+$", "")
  if line == "" then
    return "skip"
  end
  local only = line:match("^/wait%s+([%d%.]+)$")
  if only then
    return "wait", clamp_wait(only)
  end
  local body, after = line:match("^(.-)%s*<wait%s+([%d%.]+)>%s*$")
  if body then
    if body == "" then
      return "wait", clamp_wait(after)
    end
    return "cmd", body, clamp_wait(after)
  end
  return "cmd", line
end

-- Call from player input (a bind or click) so the actions permission applies.
function M.run(macro)
  if type(macro) ~= "table" or type(macro.lines) ~= "table" or running[macro] then
    return
  end
  running[macro] = true
  local ok, err = pcall(function()
    if direct(macro) then return end
    for _, line in ipairs(macro.lines) do
      local kind, a, b = M.parse_line(line)
      if kind == "wait" then
        sleep(a)
      elseif kind == "cmd" then
        commands.run(a)
        if b and b > 0 then
          sleep(b)
        end
      end
    end
  end)
  running[macro] = nil
  if not ok then
    error(err, 0)
  end
end

local last_warn = nil

-- run, with a refused permission reported in chat (at most every 5 s) instead of raised: repeated errors would
-- suspend the mod.
function M.run_checked(macro)
  local ok, err = pcall(M.run, macro)
  if ok then
    return true
  end
  err = tostring(err)
  if err:find("permission", 1, true) then
    local now = os.time()
    if not last_warn or now - last_warn >= 5 then
      last_warn = now
      chat.add("Hotbars could not run the macro: " .. err)
    end
    return false
  end
  error(err, 0)
end

return M
