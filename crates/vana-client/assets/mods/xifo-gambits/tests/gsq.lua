-- XIFO Gambits' reading and writing of the server's !gsq lines: rules both ways, a set in segments, the catalog and its
-- check, saves cut into parts.

local G = require("src.gsq")

-- rules
local r = G.decode_rule("1,MELEE.RANGED-TANK,HPP_LT,50,NOT_STATUS,HASTE,f,CURE,,30")
assert(r.on and r.target == "MELEE" and r.include[1] == "RANGED" and r.exclude[1] == "TANK")
assert(r.cond == "HPP_LT" and r.arg == "50" and r.cond2 == "NOT_STATUS" and r.arg2 == "HASTE")
assert(r.kind == "f" and r.name == "CURE" and r.retry == nil and r.level == "30" and r.id == nil)
assert(G.encode_rule(r) == "1,MELEE.RANGED-TANK,HPP_LT,50,NOT_STATUS,HASTE,f,CURE,,30")
assert(G.encode_rule(G.decode_rule("0,SELF,ALWAYS,,,,r,,")) == "0,SELF,ALWAYS,,,,r,,")
assert(G.encode_rule(G.decode_rule("1,SELF,TP_GTE,1000,,,j,BERSERK,5,,w2")) == "1,SELF,TP_GTE,1000,,,j,BERSERK,5,,w2")
assert(G.decode_rule("?") == nil)

-- a set, one long rule in two segments, one the wire cannot carry
local set = G.parse_set({ "hdr|4|WHM|o|3|w4|Fayd heals", "rule|1|1/1|1,PARTY,HPP_LT,50,,,f,CURE,",
  "rule|2|1/2|1,SELF,MPP_LT,30,", "rule|2|2/2|,,j,CONVERT,", "rule|3|1/1|?", "end|3" })
assert(set.num == 4 and set.job == "WHM" and set.rev == "3" and set.name == "Fayd heals" and #set.rules == 3)
assert(set.rules[2].kind == "j" and set.rules[2].name == "CONVERT" and set.rules[3].raw == "?")
assert(G.editable(set.flags) and not G.editable("m") and not G.editable("or"))
local err
set, err = G.parse_set({ "err|missing|no usable Rule Set with that number" })
assert(set == nil and err.code == "missing" and err.text:find("no usable", 1, true))

-- mates and lists
local mates = G.parse_mates({ "mate|2|41|Salia|WAR|28|-|0|0|-", "mate|1|40|Fayd|WHM|30|BLM|4|1|15", "end|2" })
assert(#mates == 2 and mates[1].name == "Fayd" and mates[1].set == 4 and mates[1].out and mates[1].sub_level == 15)
assert(mates[2].sub == nil)
local sets = G.parse_list({ "set|4|WHM|3|o|3|w4|Fayd heals", "set|1|WHM|5|m|1|whm1|Hand|made", "end|2" })
assert(#sets == 2 and sets[2].name == "Hand|made" and sets[2].flags == "m")

-- the catalog: entries and the check over "kind:entry\n" in order
local lists = { t = { "SELF", "PARTY" }, c = { "ALWAYS:n", "HPP_LT:r:0:100", "STATUS:e" }, f = { "CURE" },
  j = { "DIVINE_SEAL:15" }, s = { "CURE:1", "BLIND:4s" }, u = { "CURE_II:11" } }
local lines, h, seq = {}, 7, 0
for _, kind in ipairs({ "t", "c", "f", "j", "s", "u" }) do
  for _, e in ipairs(lists[kind]) do h = G.check(kind .. ":" .. e .. "\n", h) end
  seq = seq + 1
  lines[#lines + 1] = ("cat|%d|%s|%s"):format(seq, kind, table.concat(lists[kind], ","))
end
lines[#lines + 1] = ("end|%d|2.3.1.1.2.1|%d"):format(seq, h)
local cat = G.parse_catalog(lines)
assert(cat and #cat.t == 2 and cat.c[2].kind == "r" and cat.c[2].hi == 100 and cat.c[3].kind == "e")
assert(cat.j[1].name == "DIVINE_SEAL" and cat.j[1].level == 15 and cat.s[2].sub and cat.u[1].level == 11)
lines[2] = "cat|2|c|ALWAYS:n,HPP_LT:r:0:99,STATUS:e"
assert(G.parse_catalog(lines) == nil, "a changed entry fails the check")
table.remove(lines, 3)
assert(G.parse_catalog(lines) == nil, "a missing line")

-- rule problems against the catalog
assert(G.rule_problem({ target = "SELF", cond = "HPP_LT", arg = "150", kind = "f", name = "CURE" }, cat):find("0-100"))
assert(G.rule_problem({ target = "SELF", cond = "STATUS", arg = "", kind = "f", name = "CURE" }, cat))
assert(G.rule_problem({ target = "SELF", cond = "HPP_LT", arg = "50", kind = "s" }, cat) == "needs an action")
assert(G.rule_problem({ target = "SELF", cond = "HPP_LT", arg = "50", kind = "r" }, cat) == nil)

-- saves in parts of at most CHUNK bytes; the name on the wire
local rules = {}
for i = 1, 8 do rules[i] = G.decode_rule("1,PARTY_LOWEST_HPP,HPP_LT,5" .. i .. ",NOT_STATUS,PROTECT,f,CURE,,") end
local out = G.save_lines({ num = 0, job = "WHM", name = "Big heals!", rules = rules }, 901)
assert(#out > 1)
local data = {}
for i, line in ipairs(out) do
  local tok, seq2, total, chunk = line:match("^!gsq x (%d+) (%d+) (%d+) (%S+)$")
  assert(tok == "901" and tonumber(seq2) == i and tonumber(total) == #out and #chunk <= G.CHUNK)
  data[i] = chunk
end
local whole = table.concat(data)
assert(whole:sub(1, 22) == "0,-,WHM,8,Big_heals;1,", whole:sub(1, 30))
assert(select(2, whole:gsub(";", "")) == 8)
assert(G.save_lines({ num = 3, rev = "2", job = "WHM", name = "x", rules = { { raw = "?" } } }, 1) == nil)
assert(G.save_lines({ num = 3, rev = "2", job = "WHM", name = "x", rules = { rules[1] } }, 1)[1]:find(" 3,2,WHM,1,x;", 1, true))

-- what the list shows
local when, what = G.describe(G.decode_rule("1,PARTY,HPP_LT,50,NOT_STATUS,PROTECT,f,CURE,"))
assert(when == "HP below 50% and Missing effect Protect" and what == "Cure (best)", when .. " / " .. what)
assert(select(2, G.describe(G.decode_rule("1,TARGET,ALWAYS,,,,n,,"))) == "Best nuke")
assert(G.tidy("CURE_II") == "Cure II" and G.tidy("DIVINE_SEAL") == "Divine Seal" and G.tidy("VIVID") == "Vivid")

-- a check without its end line (no answer, or cut short) gives no verdict
local chk
chk, err = G.parse_check({})
assert(chk == nil and err.code == "noreply")
chk = G.parse_check({ "chk|2|not learned", "end|1|30|BLM|15" })
assert(#chk.problems == 1 and chk.level == 30)
assert(G.parse_check({ "chk|2|not learned" }) == nil)
