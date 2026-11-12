-- Hotbars: a macro's first action names its icon and recast; a drag swaps two slots, in the sub job's set when
-- either slot comes from it.

res = setmetatable({
  spell = function(id) if id == 1 then return { name = "Cure" } end end,
  ability = function(id)
    if id == 0x20 then return { name = "Raging Axe" } end
    if id == 0x210 then return { name = "Provoke", recast_id = 5 } end
  end,
}, { __index = function() return function() return nil end end })
game.find_items = function(name) if name == "potion" then return { { id = 4112 } } end return {} end
local a = require("src.actions")
local p = a.of({ lines = { "/echo go", '/ja "Provoke" <t>' } })
assert(p.kind == "ability" and p.id == 0x210 and p.recast_id == 5)
assert(a.of({ lines = { "/ma Cure <me>" } }).id == 1)
assert(a.of({ lines = { '/ws "Raging Axe" <t>' } }).id == 0x20)
assert(a.of({ lines = { '/item "Potion" <me>' } }).id == 4112)
assert(a.of({ lines = { "/echo nothing" } }) == false)
local r = { spells = {}, abilities = { [5] = 15 }, totals = { spells = {}, abilities = { [5] = 30 } } }
local left, frac = a.recast(p, r)
assert(left == 15 and frac == 0.5)

local saved
local data = require("src.data")
data.init({ read = function() return nil end, write = function(_, t) saved = t end, decode = json.decode,
  encode = json.encode })
local m = function(n) return { name = n, lines = { "/echo " .. n } } end
data.set_macro("WAR", nil, 1, 1, m("one"))
data.set_macro("WAR", nil, 1, 2, m("two"))
data.set_macro("WAR", "NIN", 1, 3, m("sub"))
data.swap("WAR", nil, 1, 1, 1, 2)
assert(data.resolve("WAR", nil, 1, 1).name == "two" and data.resolve("WAR", nil, 1, 2).name == "one")
-- Into the sub set: the main set keeps its macro under the sub set's.
data.swap("WAR", "NIN", 1, 3, 1, 4)
local four, from = data.resolve("WAR", "NIN", 1, 4)
assert(four.name == "sub" and from == "sub")
assert(data.resolve("WAR", "NIN", 1, 3) == nil)
assert(saved ~= nil)

-- The library: an entry on slots of both sets by reference; editing it changes them all.
local id = data.save_entry("WAR", nil, { name = "Prov", action = { kind = "ja", name = "Provoke",
  target = "t" }, lines = { "/p Provoked <t>" } })
data.assign("WAR", nil, 2, 1, id)
data.assign("WAR", "NIN", 2, 2, id)
local got, gf = data.resolve("WAR", "NIN", 2, 2)
assert(got.ref == id and gf == "sub" and got.lines[1] == '/ja "Provoke" <t>'
  and got.lines[2] == "/p Provoked <t>")
assert(data.uses("WAR", id) == 2)
data.save_entry("WAR", id, { name = "Provoke!", action = { kind = "ja", name = "Provoke", target = "bt" } })
assert(data.resolve("WAR", nil, 2, 1).name == "Provoke!")
assert(data.effective("WAR", "NIN")[2][2].macro.lines[1] == '/ja "Provoke" <bt>')
-- A swap keeps the reference (not a copy of its lines).
data.swap("WAR", nil, 2, 1, 2, 5)
assert(data.get_raw("WAR", nil, 2, 5).ref == id and data.get_raw("WAR", nil, 2, 1) == nil)
-- set_macro with a resolved library macro (as the pickers give) puts the reference.
data.set_macro("WAR", nil, 2, 6, data.resolve("WAR", nil, 2, 5))
assert(data.get_raw("WAR", nil, 2, 6).ref == id and data.uses("WAR", id) == 3)
-- A sub slot over a main macro: deleting the entry leaves it blank, not showing the main macro again.
data.set_macro("WAR", nil, 2, 7, { name = "under", lines = { "/ja \"Berserk\" <me>" } })
data.assign("WAR", "NIN", 2, 7, id)
data.delete_entry("WAR", id)
assert(data.resolve("WAR", "NIN", 2, 2) == nil and data.resolve("WAR", nil, 2, 5) == nil)
assert(data.uses("WAR", id) == 0 and data.resolve("WAR", nil, 1, 1).name == "two")
assert(data.get_raw("WAR", "NIN", 2, 7) == false and data.resolve("WAR", "NIN", 2, 7) == nil)
assert(data.resolve("WAR", nil, 2, 7).name == "under")

-- The builder's lines, and a system command's stand-in icon with no recast.
assert(data.command({ kind = "equip", name = "Warp Ring", target = "ring1" }) == '/equip ring1 "Warp Ring"')
assert(data.command({ kind = "cmd", name = "/attack", target = "t" }) == "/attack <t>")
assert(data.command({ kind = "cmd", name = "/heal" }) == "/heal")
local atk = a.of({ lines = { "/attack <t>" } })
assert(atk and atk.kind == "item" and atk.no_recast and a.recast(atk, r) == 0)
