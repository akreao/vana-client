-- Hotbars: each main / sub job pair has its own bars and crossbars. A pair's first use starts its set from the old
-- layout (one main set, sub sets over it): THF/NIN keeps Utsusemi, THF/DNC drops it (a spell neither THF nor DNC
-- learns) and keeps Spectral Jig only while the server lists it; abilities are judged once the list has settled,
-- and at main level 99 only. Pet commands stay though unlisted (the list has them only while the pet is out).
-- Until then the pair's set is shown, not saved.

local spells = {
  [338] = { name = "Utsusemi: Ichi", kind = 4, levels = { [13] = 12 } },
  [353] = { name = "Tonko: Ichi", kind = 4, levels = { [13] = 9 } },
}
local abilities = {
  [0x2b5] = { name = "Spectral Jig", kind = 1, targets = 1 },
  [0x240] = { name = "Steal", kind = 1, targets = 0x20 },
  [0x480] = { name = "Searing Light", kind = 1, targets = 1 },
}
res = setmetatable({ spell = function(id) return spells[id] end, ability = function(id) return abilities[id] end,
  job = function(id) return ({ [6] = "THF", [13] = "NIN", [19] = "DNC" })[id] end },
  { __index = function() return function() end end })
local ME = { main_job = 6, main_level = 99, sub_job = 13, sub_level = 49 }
local LISTED = { 0x240 }
game = setmetatable({ player = function() return ME end, abilities = function() return LISTED end,
  spells = function() return { 338 } end }, { __index = function() return function() return {} end end })
chat = { add = function() end }

-- The old layout: THF's main set holds Steal, Utsusemi and Spectral Jig; its NIN sub set Tonko on slot 4 and a
-- blank over slot 5; its crossbars Utsusemi.
local function slot(name, line) return { name = name, lines = { line } } end
local OLD = { THF = {
  main = { ["1"] = { ["1"] = slot("Steal", '/ja "Steal" <t>'), ["2"] = slot("Utsu", '/ma "Utsusemi: Ichi" <me>'),
    ["3"] = slot("Jig", '/ja "Spectral Jig" <me>'), ["5"] = slot("Gone", "/p x"),
    ["7"] = slot("Pact", '/pet "Searing Light" <me>') } },
  sub = { NIN = { ["1"] = { ["4"] = slot("Tonko", '/ma "Tonko: Ichi" <me>'), ["5"] = false } } },
  pad = { lt = { up = slot("Utsu", '/ma "Utsusemi: Ichi" <me>') } },
} }
local files = { ["macros.json"] = json.encode(OLD) }
local writes = 0
local data = require("src.data")
data.init({ read = function(f) return files[f] end, write = function(f, t) files[f] = t; writes = writes + 1 end,
  decode = json.decode, encode = json.encode })
local subjob = require("src.subjob")
data.keep, data.keep_ready = subjob.keep, subjob.ready
local now = 0
clock = function() return now end
local function settle()
  subjob.watch()
  now = now + 3
  assert(subjob.watch(), "the list did not settle")
end
local function names(main, sub)
  local out = {}
  for b, row in pairs(data.effective(main, sub)) do
    for s, c in pairs(row) do out[#out + 1] = b .. "." .. s .. "=" .. c.macro.name end
  end
  table.sort(out)
  return table.concat(out, ",")
end

-- THF/NIN before the list settled: shown (Jig kept: abilities not judged yet), not saved.
subjob.watch()
assert(names("THF", "NIN") == "1.1=Steal,1.2=Utsu,1.3=Jig,1.4=Tonko,1.7=Pact", names("THF", "NIN"))
assert(writes == 0, "a pair's set was saved before the list settled")
-- Settled: Spectral Jig is not listed for THF/NIN at 99: another sub job's; the pact stays; saved now.
settle()
assert(names("THF", "NIN") == "1.1=Steal,1.2=Utsu,1.4=Tonko,1.7=Pact", names("THF", "NIN"))
assert(writes > 0, "the pair's set was not saved")
-- THF/DNC: Utsusemi is neither THF's nor DNC's; Spectral Jig listed now, kept; the NIN sub set's Tonko is not here.
-- The sub job changed and the server's list for it has not come yet: the old pair's settled list judges nothing, Jig
-- stays and nothing is saved.
ME.sub_job = 19
local before = writes
assert(not subjob.ready(), "the old pair's settled list counted for THF/DNC")
assert(names("THF", "DNC"):find("1.3=Jig", 1, true), names("THF", "DNC"))
assert(writes == before, "THF/DNC's set was saved before its list came")
LISTED = { 0x240, 0x2b5 }
settle()
assert(names("THF", "DNC") == "1.1=Steal,1.3=Jig,1.5=Gone,1.7=Pact", names("THF", "DNC"))
-- The pairs are apart from here on.
data.set_macro("THF", "DNC", 1, 6, slot("Samba", "/p samba"))
assert(not names("THF", "NIN"):find("Samba", 1, true), "THF/NIN shows THF/DNC's slot")
-- The crossbars start the same way: THF/DNC's have no Utsusemi, THF/NIN's do.
assert(data.pad_resolve("THF", "DNC", "lt", "up") == nil, "THF/DNC's crossbars kept Utsusemi")
ME.sub_job = 13
assert(data.pad_resolve("THF", "NIN", "lt", "up").name == "Utsu")
-- Below 99 an ability not listed stays (it may be THF's, not learned yet).
ME.main_level, ME.sub_job, LISTED = 50, 0, { 0x240 }
settle()
-- THF alone: Jig stays (not judged below 99), Utsusemi goes (no NIN), slot 5 is the old main set's.
assert(names("THF", nil) == "1.1=Steal,1.3=Jig,1.5=Gone,1.7=Pact", names("THF", nil))
