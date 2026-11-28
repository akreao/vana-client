-- Hotbars: a shared macro unshared while one job's old layout still names it (on a sub set a pair has not started
-- from yet) gives that job a copy, though another pair of the job already has its own set.

local OLD = { RDM = {
  sub = { NIN = { ["1"] = { ["4"] = { ref = "s1" } } } },
  sets = { WHM = { ["1"] = { ["1"] = { name = "Cure", lines = { "/ma Cure <t>" } } } } },
} }
local files = {
  ["macros.json"] = json.encode(OLD),
  ["sticky.json"] = json.encode({ next_id = 2, library = { s1 = { name = "Warp", lines = { "/echo warp" } } } }),
}
local data = require("src.data")
data.init({ read = function(f) return files[f] end, write = function(f, t) files[f] = t end, decode = json.decode,
  encode = json.encode })
data.load()
assert(data.macro_of("RDM", "s1").name == "Warp")

local nid, why = data.set_sticky("WAR", "s1", false)
assert(nid and not data.is_sticky(nid) and data.macro_of("WAR", "s1") == nil, ("unshare: %s %s"):format(tostring(nid), tostring(why)))
local copy
for _, e in ipairs(data.library("RDM")) do
  if e.entry.name == "Warp" then copy = e.id end
end
assert(copy, "RDM's old NIN sub set names the macro, but RDM got no copy")
assert(data.resolve("RDM", "NIN", 1, 4) and data.resolve("RDM", "NIN", 1, 4).name == "Warp",
  "RDM/NIN's slot lost the macro")
