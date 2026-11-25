-- Starting rule sheets by main job: our own small picks, a place to start editing from. A job without one gets the
-- weapon skill rule alone. Rules that act on their own in a way you may not want (nukes) start switched off.

local function r(target, when, action, when2, on)
  return { on = on ~= false, target = target, when = when, when2 = when2, action = action }
end
local function c(kind, value) return { kind = kind, value = value } end
local function spell(name, best) return { kind = "spell", name = name, best = best or false } end
local function ability(name) return { kind = "ability", name = name } end
local BEST_WS = { kind = "ws", best = true }

local ws = r("target", c("tp_at_least", 1000), BEST_WS, c("engaged"))

local P = {}

P.WHM = {
  r("party", c("hp_below", 50), spell("Cure", true)),
  r("party", c("hp_below", 75), spell("Cure", true), c("not_engaged")),
  r("party", c("has", "paralysis"), spell("Paralyna")),
  r("party", c("has", "silence"), spell("Silena")),
  r("party", c("has", "poison"), spell("Poisona")),
  r("party", c("has", "blindness"), spell("Blindna")),
  r("me", c("missing", "Protect"), spell("Protect", true), c("not_engaged")),
  r("me", c("missing", "Shell"), spell("Shell", true), c("not_engaged")),
}

P.RDM = {
  r("party", c("hp_below", 50), spell("Cure", true)),
  r("me", c("missing", "Refresh"), spell("Refresh", true)),
  r("me", c("missing", "Haste"), spell("Haste")),
  r("me", c("missing", "Phalanx"), spell("Phalanx")),
  r("me", c("missing", "Protect"), spell("Protect", true), c("not_engaged")),
  r("me", c("missing", "Shell"), spell("Shell", true), c("not_engaged")),
  ws,
}

P.BLM = {
  r("me", c("missing", "Stoneskin"), spell("Stoneskin"), c("not_engaged")),
  r("target", c("my_mp_above", 30), spell("Thunder", true), nil, false),
}

P.PLD = {
  r("me", c("hp_below", 50), spell("Cure", true)),
  r("target", c("engaged"), spell("Flash")),
  r("me", c("missing", "Protect"), spell("Protect", true), c("not_engaged")),
  ws,
}

P.WAR = {
  ws,
  r("me", c("engaged"), ability("Berserk"), c("missing", "Berserk")),
  r("me", c("engaged"), ability("Aggressor"), c("missing", "Aggressor")),
}

P.SAM = {
  ws,
  r("me", c("engaged"), ability("Hasso"), c("missing", "Hasso")),
}

P.DRK = {
  ws,
  r("me", c("engaged"), ability("Last Resort"), c("missing", "Last Resort")),
}

local M = {}

-- A fresh copy of the job's sheet.
function M.for_job(job)
  local out = {}
  for _, rule in ipairs(P[job] or { ws }) do
    local copy = {}
    for k, v in pairs(rule) do
      if type(v) == "table" then
        local t = {}
        for kk, vv in pairs(v) do t[kk] = vv end
        copy[k] = t
      else
        copy[k] = v
      end
    end
    out[#out + 1] = copy
  end
  return out
end

return M
