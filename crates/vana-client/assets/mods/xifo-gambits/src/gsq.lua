-- The XI: Forever Offline server's Rule Set editor commands ("!gsq ..."), kept free of the client for the tests:
-- request lines, their replies read into tables, the rule line format, and saves cut into parts.
--
-- Every reply line is "GSQ|<request id>|<kind>|<fields...>" (XIFO Link hands them on without "GSQ|<id>|"). A rule on
-- the wire is "on,TARGET,COND,ARG,COND2,ARG2,KIND,NAME,RETRY[,LEVEL[,ID]]".

local M = {}

M.CHUNK = 90 -- bytes of save data per line (the server takes 110; the whole chat line stays well under its limit)

M.KINDS = { "f", "s", "j", "r", "n", "b" }
M.KIND_LABELS = { f = "Best of family", s = "Spell", j = "Job ability", r = "Ranged attack", n = "Best nuke",
  b = "Magic burst" }
M.NAMED = { f = true, s = true, j = true }

M.TARGET_LABELS = {
  SELF = "Self", PARTY = "Party member", PARTY_LOWEST_HPP = "Party member with the lowest HP", TANK = "Tank",
  CASTER = "Mage", MELEE = "Melee", RANGED = "Ranged", TOP_ENMITY = "Ally with the enemy's attention",
  TARGET = "Enemy", MASTER = "You", TRIGGER_SELF_ACTION_TARGET = "Self (act on the enemy)",
}

-- Condition labels: the text before the argument.
M.COND_LABELS = {
  ALWAYS = "Always", HPP_LT = "HP below %", HPP_GTE = "HP at least %", MPP_LT = "MP below %", MPP_GTE = "MP at least %",
  TP_LT = "TP below", TP_GTE = "TP at least", LVL_LT = "Level below", LVL_GTE = "Level at least",
  HP_MISSING = "HP missing at least", RANDOM = "Chance %", TIMER = "Every (seconds)", STATUS = "Has effect",
  NOT_STATUS = "Missing effect", HAS_TOP_ENMITY = "Has the enemy's attention",
  NOT_HAS_TOP_ENMITY = "Doesn't have the enemy's attention", PT_HAS_TANK = "Party has a tank", NOT_PT_HAS_TANK = "No tank",
  SC_AVAILABLE = "Skillchain open", NOT_SC_AVAILABLE = "No skillchain open", MB_AVAILABLE = "Magic burst open",
  READYING_WS = "Enemy readies a weapon skill", READYING_MS = "Enemy readies a special ability",
  READYING_JA = "Enemy readies an ability", CASTING_MA = "Enemy casting", CASTING_DEBUFF = "Enemy casting a debuff",
  HAS_MP = "Has MP", HAS_MP_MPP_LT = "Has MP, below %", HAS_MP_MPP_GTE = "Has MP, at least %",
  JA_ON_COOLDOWN = "Ability on cooldown", MOD_LT = "Shadows below",
}

-- "CURE_II" -> "Cure II", "DIVINE_SEAL" -> "Divine Seal".
function M.tidy(name)
  if not name or name == "" then return "" end
  local words = {}
  for w in name:gmatch("[^_%s]+") do
    words[#words + 1] = w:upper():match("^[IVX]+$") and w:upper() or (w:sub(1, 1):upper() .. w:sub(2):lower())
  end
  return table.concat(words, " ")
end

function M.target_label(t) return M.TARGET_LABELS[t] or M.tidy(t) end
function M.cond_label(c) return M.COND_LABELS[c] or M.tidy(c) end

local function split(text, sep)
  local out, start = {}, 1
  while true do
    local at = text:find(sep, start, true)
    if not at then
      out[#out + 1] = text:sub(start)
      return out
    end
    out[#out + 1] = text:sub(start, at - 1)
    start = at + #sep
  end
end
M.split = split

-- The server's catalog check: the same running number over every "kind:entry\n".
function M.check(text, h)
  h = h or 7
  for i = 1, #text do h = (h * 31 + text:byte(i)) % 1000000007 end
  return h
end

-- Requests ------------------------------------------------------------------------------------------------------

function M.hello(id) return ("!gsq hello %d"):format(id) end
function M.mates(id) return ("!gsq mates %d"):format(id) end
function M.list(id, job) return ("!gsq list %d %s"):format(id, job) end
function M.get(id, num) return ("!gsq get %d %d"):format(id, num) end
function M.chk(id, slot, num) return ("!gsq chk %d %d %d"):format(id, slot, num) end
function M.del(token, num, rev) return ("!gsq del %d %d %s"):format(token, num, rev or "-") end

-- The catalog for one mate: "j" / "jm" say which argument kinds we read (versions 5 / 6).
function M.catm(id, slot, version)
  local flags = (version or 0) >= 6 and " jm" or ((version or 0) >= 5 and " j" or "")
  return ("!gsq catm %d %d%s"):format(id, slot, flags)
end

-- Replies -------------------------------------------------------------------------------------------------------

-- The lines of one reply ("kind|fields"), split; an "err" line anywhere makes it an error.
local function rows(lines)
  local out = {}
  for _, line in ipairs(lines) do
    local f = split(line, "|")
    if f[1] == "err" then return nil, { code = f[2], text = table.concat(f, "|", 3) } end
    out[#out + 1] = f
  end
  return out
end

function M.parse_hello(lines)
  local r, err = rows(lines)
  if not r then return nil, err end
  for _, f in ipairs(r) do
    if f[1] == "hello" then
      -- (version 10) field 6: the server's capability words ("live", "reapply1"), "-" for none
      local caps = {}
      for word in (f[6] or ""):gmatch("[^,]+") do
        if word ~= "-" then caps[word] = true end
      end
      return { version = tonumber(f[2]) or 0, max_rules = tonumber(f[3]) or 32, max_line = tonumber(f[4]) or 120,
        caps = caps }
    end
  end
  return nil, { code = "noreply", text = "no answer" }
end

function M.parse_mates(lines)
  local r, err = rows(lines)
  if not r then return nil, err end
  local out = {}
  for _, f in ipairs(r) do
    if f[1] == "mate" then
      out[#out + 1] = { slot = tonumber(f[2]), id = tonumber(f[3]), name = f[4], job = f[5], level = tonumber(f[6]) or 0,
        sub = f[7] ~= "-" and f[7] or nil, set = tonumber(f[8]) or 0, out = f[9] == "1",
        sub_level = tonumber(f[10]) or 0,
        -- (version 10, capability "live") what the mate runs now: state, why, set (0 = job rules), fallback, left out
        live = #f >= 15 and { state = f[11], why = f[12], set = tonumber(f[13]) or 0, fallback = tonumber(f[14]) or 0,
          dropped = tonumber(f[15]) or 0 } or nil }
    end
  end
  table.sort(out, function(a, b) return a.slot < b.slot end)
  return out
end

local FALLBACK = { "the chosen set is gone", "that number is now another set", "the chosen set cannot be used here",
  "no set for this job" }
local PENDING = { job = "its job changed", choice = "another set was chosen", gone = "its set was deleted",
  edited = "its set was edited", role = "its role changed" }

-- What a mate runs now (version 10), as a sentence; nil when the server does not say or the mate is not out.
function M.live_text(m)
  local l = m.live
  if not l or l.state == "off" then return nil end
  local now = l.set > 0 and ("Rule Set %d"):format(l.set) or "its job rules"
  local t
  if l.state == "set" or l.state == "job" then
    t = "Runs " .. now .. "."
  elseif l.state == "fallback" then
    t = "Runs its job rules: " .. (FALLBACK[l.fallback] or "the chosen set could not be used") .. "."
  elseif l.state == "pending" then
    t = ("Runs %s; %s, so the next call changes them (Apply)."):format(now, PENDING[l.why] or "settings changed")
  elseif l.state == "addon" then
    t = "An add-on runs this mate."
  else
    t = "What it runs is not known (called before the server kept track)."
  end
  if l.dropped > 0 then t = t .. (" %d rule%s left out."):format(l.dropped, l.dropped == 1 and "" or "s") end
  return t
end

-- flags: o yours (editable), m written by hand (copy only), x another's, r read-only
function M.parse_list(lines)
  local r, err = rows(lines)
  if not r then return nil, err end
  local out = {}
  for _, f in ipairs(r) do
    if f[1] == "set" then
      out[#out + 1] = { num = tonumber(f[2]), job = f[3], count = tonumber(f[4]) or 0, flags = f[5] or "",
        rev = f[6], id = f[7], name = table.concat(f, "|", 8) }
    end
  end
  return out
end

function M.editable(flags) return flags:find("o", 1, true) ~= nil and not flags:find("r", 1, true) end

-- One rule line -> a table; nil for "?" (a rule the wire cannot carry: kept, never edited).
function M.decode_rule(text)
  if text == "?" then return nil end
  local f = split(text, ",")
  if #f < 8 then return nil end
  local target_field = split(f[2], "-")
  local head = split(target_field[1], ".")
  local include, exclude = {}, {}
  for i = 2, #head do include[#include + 1] = head[i] end
  for i = 2, #target_field do exclude[#exclude + 1] = target_field[i] end
  return { on = f[1] ~= "0", target = head[1], include = include, exclude = exclude, cond = f[3], arg = f[4],
    cond2 = f[5] ~= "" and f[5] or nil, arg2 = f[6], kind = f[7], name = f[8] ~= "" and f[8] or nil,
    retry = f[9] ~= "" and f[9] or nil, level = f[10] ~= "" and f[10] or nil, id = f[11] ~= "" and f[11] or nil }
end

function M.encode_rule(r)
  local target = r.target
  for _, role in ipairs(r.include or {}) do target = target .. "." .. role end
  for _, role in ipairs(r.exclude or {}) do target = target .. "-" .. role end
  local f = { r.on == false and "0" or "1", target, r.cond, r.arg or "", r.cond2 or "", r.cond2 and (r.arg2 or "") or "",
    r.kind, M.NAMED[r.kind] and (r.name or "") or "", r.retry or "" }
  if (r.level and r.level ~= "") or r.id then f[#f + 1] = r.level or "" end
  if r.id then f[#f + 1] = r.id end
  return table.concat(f, ",")
end

-- "get": { num, job, flags, rev, id, name, rules = { rule or { raw = "?" } } }
function M.parse_set(lines)
  local r, err = rows(lines)
  if not r then return nil, err end
  local set, parts, segs, count = { rules = {} }, {}, {}, nil
  for _, f in ipairs(r) do
    if f[1] == "hdr" then
      set.num, set.job, set.flags, set.rev, set.id = tonumber(f[2]), f[3], f[4] or "", f[5], f[6]
      set.name = table.concat(f, "|", 7)
    elseif f[1] == "rule" then
      local i = tonumber(f[2])
      local k, n = (f[3] or ""):match("^(%d+)/(%d+)$")
      if not i or not k then return nil, { code = "badset", text = "a rule line is malformed" } end
      local g = segs[i] or { n = tonumber(n), got = 0 }
      if tonumber(k) ~= g.got + 1 or tonumber(n) ~= g.n then
        return nil, { code = "badset", text = "a rule arrived out of order" }
      end
      g.got, segs[i] = g.got + 1, g
      parts[i] = (parts[i] or "") .. table.concat(f, "|", 4)
    elseif f[1] == "end" then
      count = tonumber(f[2])
    end
  end
  if not set.num then return nil, { code = "noreply", text = "no answer" } end
  -- every rule the end line counts, each with all its segments: a set short of rules would save without them
  local whole = count ~= nil
  for i = 1, count or 0 do
    if not segs[i] or segs[i].got ~= segs[i].n then whole = false end
  end
  for i in pairs(segs) do
    if not count or i > count then whole = false end
  end
  if not whole then
    return nil, { code = "badset", text = "the Rule Set arrived incomplete" }
  end
  for i = 1, #parts do
    set.rules[i] = M.decode_rule(parts[i]) or { raw = parts[i] }
  end
  return set
end

-- "catm": { t = {...}, c = { {name, kind, lo, hi} }, f = {...}, j = { {name, level, sub} }, s = ..., u = ... }; the
-- server's check over the entries must hold.
function M.parse_catalog(lines)
  local r, err = rows(lines)
  if not r then return nil, err end
  local cat = { t = {}, c = {}, f = {}, j = {}, s = {}, u = {} }
  local h, seq, ok = 7, 0, nil
  for _, f in ipairs(r) do
    if f[1] == "cat" then
      seq = seq + 1
      if tonumber(f[2]) ~= seq then return nil, { code = "badcat", text = "a catalog line is missing" } end
      local kind = f[3]
      for _, e in ipairs(split(table.concat(f, "|", 4), ",")) do
        if e ~= "" then
          h = M.check(kind .. ":" .. e .. "\n", h)
          local into = cat[kind]
          if into then
            if kind == "c" then
              local p = split(e, ":")
              into[#into + 1] = { name = p[1], kind = p[2], lo = tonumber(p[3]), hi = tonumber(p[4]) }
            elseif kind == "j" or kind == "s" or kind == "u" then
              local name, lv, sub = e:match("^([^:]+):(%d+)(s?)$")
              into[#into + 1] = { name = name or e, level = tonumber(lv) or 0, sub = sub == "s" }
            else
              into[#into + 1] = e
            end
          end
        end
      end
    elseif f[1] == "end" then
      ok = tonumber(f[4]) == h and tonumber(f[2]) == seq
    end
  end
  if not ok then return nil, { code = "badcat", text = "the catalog arrived incomplete" } end
  return cat
end

-- "chk": { problems = { {rule, why} }, level, sub, sub_level }
function M.parse_check(lines)
  local r, err = rows(lines)
  if not r then return nil, err end
  local out, ended = { problems = {} }, false
  for _, f in ipairs(r) do
    if f[1] == "chk" then
      out.problems[#out.problems + 1] = { rule = f[2], why = table.concat(f, "|", 3) }
    elseif f[1] == "end" then
      ended = true
      out.level, out.sub, out.sub_level = tonumber(f[3]), f[4] ~= "-" and f[4] or nil, tonumber(f[5])
    end
  end
  -- without its end line the list may be cut short (or empty): no verdict
  if not ended then return nil, { code = "noreply", text = "the server did not answer" } end
  return out
end

-- A save's answer: { num, rev, id } on "ok", else nil + err.
function M.parse_saved(lines)
  local r, err = rows(lines)
  if not r then return nil, err end
  for _, f in ipairs(r) do
    if f[1] == "ok" then return { num = tonumber(f[2]), rev = f[3], id = f[4] } end
  end
  return nil, { code = "noreply", text = "the server did not confirm" }
end

-- Saving --------------------------------------------------------------------------------------------------------

-- A set's name on the wire: letters, digits, - and _ (spaces as _), at most 24.
function M.wire_name(name)
  local n = (name or ""):gsub("%s+", "_"):gsub("[^%w_%-]", "")
  return n:sub(1, 24)
end

-- The save of `set` (num 0 = a new one) as "!gsq x" lines; nil + why when it cannot be carried.
function M.save_lines(set, token)
  local name = M.wire_name(set.name)
  if name == "" then return nil, "the Rule Set needs a name" end
  if #set.rules == 0 then return nil, "the Rule Set has no rules" end
  local items = { ("%d,%s,%s,%d,%s"):format(set.num or 0, set.num and set.num > 0 and set.rev or "-", set.job,
    #set.rules, name) }
  for i, r in ipairs(set.rules) do
    if r.raw then return nil, ("rule %d is one this editor cannot change; the set cannot be saved here"):format(i) end
    items[#items + 1] = M.encode_rule(r)
  end
  local data = table.concat(items, ";")
  if not data:match("^[%w_%-%.,;]+$") then return nil, "a rule holds a character the server does not take" end
  local total = math.ceil(#data / M.CHUNK)
  local out = {}
  for seq = 1, total do
    out[seq] = ("!gsq x %d %d %d %s"):format(token, seq, total, data:sub((seq - 1) * M.CHUNK + 1, seq * M.CHUNK))
  end
  return out
end

-- A rule's problem before saving (the server checks again), or nil.
function M.rule_problem(r, cat)
  if r.raw then return nil end
  if not r.target or r.target == "" then return "needs a target" end
  if not r.cond or r.cond == "" then return "needs a condition" end
  for _, pair in ipairs({ { r.cond, r.arg }, { r.cond2, r.arg2 } }) do
    local c = pair[1] and M.find_cond(cat, pair[1])
    if c and c.kind == "r" then
      local v = tonumber(pair[2] or "")
      if not v or v ~= math.floor(v) or v < c.lo or v > c.hi then
        return ("%s needs a whole number %d-%d"):format(M.cond_label(c.name), c.lo, c.hi)
      end
    elseif c and (c.kind == "e" or c.kind == "j") and (pair[2] or "") == "" then
      return M.cond_label(c.name) .. (c.kind == "e" and " needs a status effect" or " needs a job ability")
    end
  end
  if M.NAMED[r.kind] and (r.name or "") == "" then return "needs an action" end
  return nil
end

function M.find_cond(cat, name)
  for _, c in ipairs(cat and cat.c or {}) do
    if c.name == name then return c end
  end
end

-- A rule as one line of the editor's list: when, then what.
function M.describe(r)
  if r.raw then return "(a rule this editor cannot show)", "" end
  local function cond(c, a)
    if not c then return nil end
    local label = M.cond_label(c)
    if a and a ~= "" then
      if label:find("%", 1, true) then return (label:gsub(" %%", "")) .. " " .. a .. "%" end
      return label .. " " .. (tonumber(a) and a or M.tidy(a))
    end
    return label
  end
  local when = cond(r.cond, r.arg)
  local two = cond(r.cond2, r.arg2)
  if two then when = when .. " and " .. two end
  local what = M.KIND_LABELS[r.kind] or r.kind
  if M.NAMED[r.kind] then what = M.tidy(r.name) .. (r.kind == "f" and " (best)" or "") end
  return when, what
end

-- Weapon skills (version 5) --------------------------------------------------------------------------------------

M.WS_MODES = { "Open", "Close", "Open + Close", "Independent" }
M.WS_TPS = { 1000, 1500, 2000, 3000 }

function M.wsg(id, slot) return ("!gsq wsg %d %d"):format(id, slot) end
function M.wsl(id, slot) return ("!gsq wsl %d %d"):format(id, slot) end
function M.wsa(id, slot) return ("!gsq wsa %d %d"):format(id, slot) end
-- control 0 Squad default / 1 Custom, mode 1-4, tp 1000/1500/2000/3000, ws 0 = Best
function M.wss(id, slot, s) return ("!gsq wss %d %d %d %d %d %d"):format(id, slot, s.control, s.mode, s.tp, s.ws) end

local function bool(v) if v == "1" then return true elseif v == "0" then return false end end

-- "ws": { slot, job, source (custom / default / legacy), control, mode, tp, ws, legacy_tp, legacy_ws, out, usable,
-- applied, engine }; usable / applied nil when not known
function M.parse_ws(lines)
  local r, err = rows(lines)
  if not r then return nil, err end
  for _, f in ipairs(r) do
    if f[1] == "ws" then
      return { slot = tonumber(f[2]), job = f[3], source = ({ c = "custom", d = "default", l = "legacy" })[f[4]] or f[4],
        control = tonumber(f[5]) or 0, mode = tonumber(f[6]) or 3, tp = tonumber(f[7]) or 1000, ws = tonumber(f[8]) or 0,
        legacy_tp = tonumber(f[9]) or 0, legacy_ws = f[10], out = f[11] == "1", usable = bool(f[12]),
        applied = bool(f[13]), engine = f[14] == "1" }
    end
  end
  return nil, { code = "noreply", text = "no answer" }
end

-- "wsl": { {id, name} } in the server's order
function M.parse_wsl(lines)
  local r, err = rows(lines)
  if not r then return nil, err end
  local out = {}
  for _, f in ipairs(r) do
    if f[1] == "wsl" then
      for e in table.concat(f, "|", 2):gmatch("[^,]+") do
        local id, name = e:match("^(%d+):(.*)$")
        if id then out[#out + 1] = { id = tonumber(id), name = name } end
      end
    end
  end
  return out
end

-- Party roles (version 7) ----------------------------------------------------------------------------------------

M.ROLE_LABELS = { [0] = "Auto", "Tank", "Melee", "Mage", "Ranged" }
M.ROLE_LETTERS = { T = "Tank", M = "Melee", C = "Mage", R = "Ranged" }
-- The Rule Set targets that are roles, in the role numbers' order.
M.ROLE_TARGETS = { "TANK", "MELEE", "CASTER", "RANGED" }

function M.prl(id) return ("!gsq prl %d"):format(id) end
function M.prs(id, who, role) return ("!gsq prs %d %d %d"):format(id, who, role) end

function M.letters(text)
  local out = {}
  for c in (text or ""):gmatch("%a") do out[#out + 1] = M.ROLE_LETTERS[c] or c end
  return table.concat(out, ", ")
end

-- "prl": { own = { {who, name, job, level, set, auto, out, live} }, others = { {name, job, kind, targets} } }
function M.parse_roles(lines)
  local r, err = rows(lines)
  if not r then return nil, err end
  local out = { own = {}, others = {} }
  for _, f in ipairs(r) do
    if f[1] == "pr" then
      out.own[#out.own + 1] = { who = tonumber(f[2]), name = f[3], job = f[4], level = tonumber(f[5]) or 0,
        set = tonumber(f[6]) or 0, auto = f[7], out = f[8] == "1", live = tonumber(f[9]) }
    elseif f[1] == "pm" then
      out.others[#out.others + 1] = { name = f[2], job = f[3], kind = f[4], targets = f[5] }
    end
  end
  return out
end

return M
