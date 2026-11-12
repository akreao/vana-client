-- The Skillchain mod's tracking: LSB's chain rules and data, a Burning Blade opening Liquefaction on a
-- monster, Naji's Flat Blade closing it into Fusion (the proc's chain), and the view of it with the party's chains.

local SC = require("sc")
-- Liquefaction then Impaction: Fusion; Fusion then Fragmentation: Light; Light then Light: Light II.
assert(SC.form({ 3 }, { 8 }) == 11 and SC.form({ 11 }, { 12 }) == 13 and SC.form({ 13 }, { 13 }) == 15)
-- The resonance's first property that pairs wins.
assert(SC.form({ 1, 3 }, { 4 }) == 10 and SC.form({ 3, 1 }, { 4 }) == 4 and SC.form({ 1 }, { 1 }) == nil)
local d = SC.data()
assert(d.weapon[128].props[1] == 1 and d.weapon[134].props[1] == 6 and d.weapon[134].props[2] == 8)
assert(SC.trust("Naji") and SC.fold("D. Shantotto") == "dshantotto")
assert(SC.chain_text(11) == "Fusion (Light, Fire)")
local me = { id = 1, index = 10, name = "Lnxcsfix", target = 0x1000, tp = 1000 }
local ents = { [0x1000] = { id = 0x1000, name = "Valkurm Emperor", kind = "mob", hpp = 80, master = 0 },
  [2] = { id = 2, name = "Naji", kind = "player", hpp = 100, master = 10, trust = true } }
game = {
  player = function() return me end, entity = function(id) return ents[id] end,
  party = function() return { { id = 1, name = "Lnxcsfix", party = 0, tp = 1000 },
    { id = 2, name = "Naji", party = 0, tp = 1200 } } end,
  abilities = function() return { 33, 35 } end,
}
res = { ability = function(id)
  return ({ [33] = { kind = 3, name = "Burning Blade" }, [35] = { kind = 3, name = "Flat Blade" },
    [34] = { kind = 3, name = "Red Lotus Blade" }, [40] = { kind = 3, name = "Vorpal Blade" } })[id] end,
  spell = function() end }
local T = require("tracker")
T.on_action({ actor = 1, category = 3, param = 33,
  targets = { { id = 0x1000, results = { { miss = 0, message = 185, value = 100, info = 0 } } } } })
local r = T.on[0x1000]
assert(r and r.chain == nil and r.props[1] == 3, "opened")
-- Naji (a trust: his skills from LSB's list) closes it: the proc's kind 6 is Fusion.
T.on_action({ actor = 2, category = 11, param = 35,
  targets = { { id = 0x1000, results = { { miss = 0, message = 185, value = 90, info = 0,
    proc = { kind = 6, message = 288, value = 50, info = 0 } } } } } })
r = T.on[0x1000]
assert(r.chain == 11 and r.links == 1 and r.life == 9, "closed")
local v = T.view()
assert(v.now.mob == "Valkurm Emperor" and v.now.chain == 11 and v.now.by == "Naji: Flat Blade", v.now.by)
assert(#v.chains > 0)
-- Fusion then Fragmentation (Naji's Vorpal Blade: Scission / Impaction is not; his list has Red Lotus
-- Blade): every closer named, the strongest first.
for _, c in ipairs(v.now.closers) do assert(c.who and c.skill and c.chain) end
