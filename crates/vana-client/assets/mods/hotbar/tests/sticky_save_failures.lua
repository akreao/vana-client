-- Hotbars: a sticky switch never loses the macro when a file cannot be written. Making it sticky writes the shared
-- file first, and the job keeps it when that fails; unsticking writes the world's file first, and the shared entry
-- stays when that fails.

local files, deny = {}, {}
local data = require("src.data")
data.init({ read = function(f) return files[f] end, decode = json.decode, encode = json.encode,
  write = function(f, t)
    if deny[f] then error("denied") end
    files[f] = t
  end })

local warp = data.save_entry("RDM", nil, { name = "Warp", lines = { '/item "Warp Ring" <me>' } })
data.assign("RDM", nil, 1, 9, warp)

-- sticky.json cannot be written: nothing moves, in memory or after a reload.
deny["sticky.json"] = true
assert(data.set_sticky("RDM", warp, true) == warp and not data.is_sticky(warp))
assert(data.get_raw("RDM", nil, 1, 9).ref == warp)
data.load()
assert(data.resolve("RDM", nil, 1, 9).name == "Warp" and data.get_raw("RDM", nil, 1, 9).ref == warp)
deny["sticky.json"] = nil

-- Made sticky, then the world's file cannot be written while unsticking: the shared entry stays, so a reload still
-- finds the macro its slot names.
local s = data.set_sticky("RDM", warp, true)
assert(data.is_sticky(s) and data.get_raw("RDM", nil, 1, 9).ref == s)
deny[data.file()] = true
data.set_sticky("RDM", s, false)
deny[data.file()] = nil
data.load()
assert(data.get_raw("RDM", nil, 1, 9).ref == s and data.resolve("RDM", nil, 1, 9).name == "Warp")
