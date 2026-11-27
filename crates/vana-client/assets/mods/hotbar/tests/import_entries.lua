-- Hotbars: macros another mod hands over (import_macros) go into a job's library or the shared one, each unless the
-- same is there already; empty ones are left out; /lua lines never come in; a failed save keeps none.

local files = {}
local fail = false
local data = require("src.data")
data.init({ read = function(f) return files[f] end,
  write = function(f, t)
    if fail then error("disk full") end
    files[f] = t
  end,
  decode = json.decode, encode = json.encode })

local function e(name, ...) return { name = name, lines = { ... } } end
local cure, raise = e("Cure", '/ma "Cure" <t>'), e("Raise", '/ma "Raise" <t>', "/p Raising <t>")

local added, skipped = data.import_entries("WHM", { cure, cure, e("Blank", "", " "), raise })
assert(added == 2 and skipped == 1, ("added %s, skipped %s"):format(added, skipped))
added, skipped = data.import_entries("WHM", { cure })
assert(added == 0 and skipped == 1, "a second import added the same macro")
-- The shared library is apart: the same macro goes there once.
added, skipped = data.import_entries("WHM", { cure }, true)
assert(added == 1 and skipped == 0, "the shared library judged by WHM's")
local names = {}
for _, it in ipairs(data.library("WHM")) do names[#names + 1] = it.entry.name .. (it.sticky and "*" or "") end
assert(table.concat(names, ",") == "Cure,Raise,Cure*", table.concat(names, ","))
assert(#data.library("WAR") == 1, "WAR sees only the shared one")
assert(data.library("WHM")[2].entry.lines[2] == "/p Raising <t>")

-- A save that fails keeps none of them.
fail = true
local why
added, skipped, why = data.import_entries("WHM", { e("Haste", '/ma "Haste" <t>') })
assert(added == 0 and why and why:find("disk full"), tostring(why))
fail = false
assert(#data.library("WHM") == 3, "the failed import left a macro behind")

-- /lua lines are left out and counted; a macro of only /lua lines is not added.
local d
added, skipped, why, d = data.import_entries("WHM", { e("Load", "/lua load x", "//LUA unload y"),
  e("Mixed", "  /lua run z", '/ma "Protect" <t>') })
assert(added == 1 and d == 3 and why == nil, ("added %s, dropped %s, why %s"):format(added, d, tostring(why)))
local mixed
for _, it in ipairs(data.library("WHM")) do
  if it.entry.name == "Mixed" then mixed = it.entry end
  assert(it.entry.name ~= "Load", "a macro of only /lua lines was added")
end
assert(mixed and #mixed.lines == 1 and mixed.lines[1] == '/ma "Protect" <t>', json.encode(mixed))
