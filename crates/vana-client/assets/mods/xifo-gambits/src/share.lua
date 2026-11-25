-- Rule Set share codes in XI: Forever Offline's "GS1" format, so a code moves between this mod and their Gambits
-- add-on both ways: "GS1" + Base58 (Bitcoin alphabet) of a bit stream (most significant bit first) and a 4-byte check.
--
-- The stream: job (5 bits, index in JOBS), the set's name (text), rules - 1 (6 bits), a pool of texts (count, then each
-- text), then per rule: on (1 bit), target (token), a role filter flag (1 bit; when set, 4 bits included roles and 4
-- bits excluded roles, bit n = ROLES[n + 1]), cond, arg, cond2, arg2 (tokens), action kind (3 bits, index in KINDS),
-- name, retry, level, id (tokens). Then zero bits to a whole byte. The check is the editor's running number
-- (h = h * 31 + byte, mod 1000000007, from 7) over those bytes, appended big-endian.
-- A number is written in 4-bit groups, lowest first, each after a "more follows" bit. A text is its length (number)
-- and 6 bits per character (an index in TEXT; 63 = the next 8 bits are the byte itself). A token is 0 for none, 1 and a
-- number for a plain whole number ("0" or "1".."999999999"), 2.. for a word of VOCABULARY, then the pool's texts.
-- The lists below are part of the format and must stay exactly as they are.

local M = {}

M.MAX_CODE, M.MAX_RULES = 6000, 32
M.JOBS = { "WAR", "MNK", "WHM", "BLM", "RDM", "THF", "PLD", "DRK", "BST", "BRD", "RNG", "SAM", "NIN", "DRG", "SMN", "BLU",
  "COR", "PUP", "DNC", "SCH", "GEO", "RUN" }
M.KINDS = { "f", "s", "j", "r", "n", "b" } -- family, spell, job ability, ranged, nuke, burst
M.ROLES = { "TANK", "MELEE", "CASTER", "RANGED" }
M.VOCABULARY = {
  "SELF", "PARTY", "PARTY_LOWEST_HPP", "TANK", "CASTER", "MELEE", "RANGED", "TOP_ENMITY", "TARGET", "MASTER",
  "TRIGGER_SELF_ACTION_TARGET", "TRIGGER_TARGET_ACTION_SELF", "PARTY_DEAD", "PARTY_MULTI",
  "ALWAYS", "HPP_LT", "HPP_GTE", "MPP_LT", "MPP_GTE", "TP_LT", "TP_GTE", "LVL_LT", "LVL_GTE", "STATUS", "NOT_STATUS",
  "STATUS_FLAG", "HAS_TOP_ENMITY", "NOT_HAS_TOP_ENMITY", "SC_AVAILABLE", "NOT_SC_AVAILABLE", "MB_AVAILABLE",
  "READYING_WS", "READYING_MS", "READYING_JA", "CASTING_MA", "CASTING_DEBUFF", "RANDOM", "NO_SAMBA", "NO_STORM",
  "PT_HAS_TANK", "NOT_PT_HAS_TANK", "IS_ECOSYSTEM", "HP_MISSING", "CASTING_ELEMENT_MA", "CAST_ELE_MA_SELF",
  "CASTING_ELE_MA_AOE", "NEED_ELE_BAREFFECT", "NO_MAX_RUNE", "HAS_RUNES", "LUNGE_MB_AVAILABLE", "SUB_ANIMATION",
  "JA_ON_COOLDOWN", "VAL_URIEL_CHECK", "TIMER", "HAS_MP", "HAS_MP_MPP_LT", "HAS_MP_MPP_GTE", "MOD_LT",
}
M.TEXT = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.- abcdefghijklmnopqrstuvw"
M.B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"

local function positions(list)
  local t = {}
  for i, v in ipairs(list) do t[v] = i end
  return t
end
local JOB, KIND, ROLE, WORD = positions(M.JOBS), positions(M.KINDS), positions(M.ROLES), positions(M.VOCABULARY)
local CHAR, B58 = {}, {}
for i = 1, #M.TEXT do CHAR[M.TEXT:byte(i)] = i - 1 end
for i = 1, #M.B58 do B58[M.B58:byte(i)] = i - 1 end

local function running(bytes)
  local h = 7
  for _, b in ipairs(bytes) do h = (h * 31 + b) % 1000000007 end
  return h
end

local function plain_number(s) return s == "0" or (s:match("^[1-9]%d*$") ~= nil and #s <= 9) end

-- bits out / in ------------------------------------------------------------------------------------------------------

local function bit_writer()
  local out, cur, used = {}, 0, 0
  local w = {}
  function w.put(n, v)
    for i = n - 1, 0, -1 do
      cur, used = (cur << 1) | ((v >> i) & 1), used + 1
      if used == 8 then out[#out + 1], cur, used = cur, 0, 0 end
    end
  end
  function w.num(v)
    repeat
      local low = v & 15
      v = v >> 4
      w.put(1, v > 0 and 1 or 0)
      w.put(4, low)
    until v == 0
  end
  function w.text(s)
    w.num(#s)
    for i = 1, #s do
      local c = CHAR[s:byte(i)]
      if c then w.put(6, c) else w.put(6, 63); w.put(8, s:byte(i)) end
    end
  end
  function w.bytes()
    if used > 0 then w.put(8 - used, 0) end
    return out
  end
  return w
end

local function bit_reader(bytes)
  local pos, total = 0, #bytes * 8
  local r = {}
  function r.get(n)
    local v = 0
    for _ = 1, n do
      if pos >= total then error("the code ends too early (it was cut)", 0) end
      v = (v << 1) | ((bytes[pos // 8 + 1] >> (7 - pos % 8)) & 1)
      pos = pos + 1
    end
    return v
  end
  function r.num()
    local v, shift = 0, 0
    for _ = 1, 8 do
      local more = r.get(1)
      v, shift = v | (r.get(4) << shift), shift + 4
      if more == 0 then return v end
    end
    error("a number in the code is too long", 0)
  end
  function r.text(max)
    local n = r.num()
    if n > max then error("a text in the code is too long", 0) end
    local s = {}
    for i = 1, n do
      local c = r.get(6)
      s[i] = c == 63 and string.char(r.get(8)) or M.TEXT:sub(c + 1, c + 1)
    end
    return table.concat(s)
  end
  function r.left() return total - pos end
  return r
end

-- Base58 -------------------------------------------------------------------------------------------------------------

local function to_b58(bytes)
  local digits, lead = {}, 0 -- digits lowest first
  while lead < #bytes and bytes[lead + 1] == 0 do lead = lead + 1 end
  for _, b in ipairs(bytes) do
    local carry = b
    for j = 1, #digits do
      carry = carry + digits[j] * 256
      digits[j], carry = carry % 58, carry // 58
    end
    while carry > 0 do digits[#digits + 1], carry = carry % 58, carry // 58 end
  end
  local s = { ("1"):rep(lead) }
  for j = #digits, 1, -1 do s[#s + 1] = M.B58:sub(digits[j] + 1, digits[j] + 1) end
  return table.concat(s)
end

local function from_b58(text)
  local bytes, lead = {}, 0 -- bytes lowest first
  while lead < #text and text:sub(lead + 1, lead + 1) == "1" do lead = lead + 1 end
  for i = 1, #text do
    local carry = B58[text:byte(i)]
    if not carry then return nil, "the code has a character a share code never has (" .. text:sub(i, i) .. ")" end
    for j = 1, #bytes do
      carry = carry + bytes[j] * 58
      bytes[j], carry = carry % 256, carry // 256
    end
    while carry > 0 do bytes[#bytes + 1], carry = carry % 256, carry // 256 end
  end
  local out = {}
  for _ = 1, lead do out[#out + 1] = 0 end
  for j = #bytes, 1, -1 do out[#out + 1] = bytes[j] end
  return out
end

-- Rule Sets ----------------------------------------------------------------------------------------------------------

local TOKENS = { "target", "cond", "arg", "cond2", "arg2", "name", "retry", "level", "id" }

-- set = { job, name, rules = { gsq rule tables } } -> the code, or nil + why
function M.encode(set)
  local rules = set and set.rules or {}
  if #rules == 0 or #rules > M.MAX_RULES then return nil, ("a Rule Set has 1-%d rules"):format(M.MAX_RULES) end
  if not JOB[set.job or ""] then return nil, "unknown job " .. tostring(set.job) end
  local pool, at = {}, {}
  for i, r in ipairs(rules) do
    if r.raw or not KIND[r.kind or ""] then return nil, ("rule %d cannot be shared exactly"):format(i) end
    for _, k in ipairs(TOKENS) do
      local v = r[k]
      if v ~= nil and v ~= "" and not WORD[v] and not plain_number(v) and not at[v] then
        pool[#pool + 1] = v
        at[v] = #pool
      end
    end
  end
  local w = bit_writer()
  w.put(5, JOB[set.job] - 1)
  w.text(set.name or "")
  w.put(6, #rules - 1)
  w.num(#pool)
  for _, s in ipairs(pool) do w.text(s) end
  local function token(v)
    if v == nil or v == "" then w.num(0)
    elseif plain_number(v) then w.num(1); w.num(tonumber(v))
    elseif WORD[v] then w.num(1 + WORD[v])
    else w.num(1 + #M.VOCABULARY + at[v]) end
  end
  local function role_bits(list)
    local b = 0
    for _, role in ipairs(list or {}) do
      if not ROLE[role] then return nil end
      b = b | (1 << (ROLE[role] - 1))
    end
    return b
  end
  for i, r in ipairs(rules) do
    w.put(1, r.on == false and 0 or 1)
    token(r.target)
    local inc, exc = role_bits(r.include), role_bits(r.exclude)
    if not inc or not exc then return nil, ("rule %d has an unknown role"):format(i) end
    if inc + exc > 0 then w.put(1, 1); w.put(4, inc); w.put(4, exc) else w.put(1, 0) end
    token(r.cond); token(r.arg); token(r.cond2); token(r.arg2)
    w.put(3, KIND[r.kind] - 1)
    token(r.name); token(r.retry); token(r.level); token(r.id)
  end
  local bytes = w.bytes()
  local h = running(bytes)
  for shift = 24, 0, -8 do bytes[#bytes + 1] = (h >> shift) & 255 end
  local code = "GS1" .. to_b58(bytes)
  if #code > M.MAX_CODE then return nil, "the Rule Set is too large for one code" end
  return code
end

-- text (spaces, line breaks and ` ignored) -> { job, name, rules = { gsq rule tables } }, or nil + why
function M.decode(text)
  local clean = tostring(text or ""):gsub("[%s`]", "")
  if #clean > M.MAX_CODE then return nil, "that is longer than any share code" end
  local version, body = clean:match("^GS(%d)(.*)$")
  if not version then return nil, "this is not a Gambits share code (those start with GS1)" end
  if version ~= "1" then return nil, "this code is of a newer format (GS" .. version .. ") than this mod reads" end
  if #body < 6 then return nil, "the code is too short (it was cut)" end
  local bytes, why = from_b58(body)
  if not bytes then return nil, why end
  if #bytes < 5 then return nil, "the code is too short (it was cut)" end
  local want = 0
  for i = #bytes - 3, #bytes do want = (want << 8) | bytes[i] end
  for _ = 1, 4 do table.remove(bytes) end
  if running(bytes) ~= want then
    return nil, "the check number does not match: the code was cut or changed (copy all of it again)"
  end
  local ok, res = pcall(function()
    local r = bit_reader(bytes)
    local job = M.JOBS[r.get(5) + 1]
    if not job then error("unknown job in the code", 0) end
    local name = r.text(24)
    local count = r.get(6) + 1
    local n = r.num()
    if n > 4 * 11 * M.MAX_RULES then error("the code has too many names", 0) end
    local pool = {}
    for i = 1, n do pool[i] = r.text(48) end
    local function token()
      local t = r.num()
      if t == 0 then return nil end
      if t == 1 then return tostring(r.num()) end
      if t <= 1 + #M.VOCABULARY then return M.VOCABULARY[t - 1] end
      local s = pool[t - 1 - #M.VOCABULARY]
      if not s then error("the code names something that is not in it", 0) end
      return s
    end
    local function roles(b)
      local out = {}
      for i, role in ipairs(M.ROLES) do
        if (b >> (i - 1)) & 1 == 1 then out[#out + 1] = role end
      end
      return out
    end
    local rules = {}
    for i = 1, count do
      local rule = { on = r.get(1) == 1, include = {}, exclude = {} }
      rule.target = token()
      if r.get(1) == 1 then rule.include, rule.exclude = roles(r.get(4)), roles(r.get(4)) end
      rule.cond, rule.arg, rule.cond2, rule.arg2 = token(), token(), token(), token()
      rule.kind = M.KINDS[r.get(3) + 1]
      if not rule.kind then error("unknown action kind in rule " .. i, 0) end
      rule.name, rule.retry, rule.level, rule.id = token(), token(), token(), token()
      rule.arg, rule.arg2 = rule.arg or "", rule.arg2 or ""
      rules[i] = rule
    end
    if r.left() >= 8 then error("the code has data left over (it was changed)", 0) end
    return { job = job, name = name, rules = rules }
  end)
  if not ok then return nil, tostring(res) end
  return res
end

return M
