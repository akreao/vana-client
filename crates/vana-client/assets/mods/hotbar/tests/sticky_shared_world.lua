-- Hotbars: the characters on one world share its file. A sticky-only edit never writes the world's file, and a sticky
-- change that does reads it afresh first, so what another character saved there meanwhile stays.

local files = {}
storage = { character = { hb_last = {} }, save = function() end }
local data = require("src.data")
data.init({ read = function(f) return files[f] end, decode = json.decode, encode = json.encode,
  write = function(f, t) files[f] = t end })
data.load()

local warp = data.save_entry("RDM", nil, { name = "Warp", lines = { "/echo warp" } }, true)
assert(warp and data.is_sticky(warp))
data.save_entry("RDM", nil, { name = "Mine", lines = { "/echo" } })

-- Another character on the world saves a macro and a slot (its client wrote the world's file; ours did not read it).
local other = json.decode(files[data.file()])
other.RDM.library["90"] = { name = "Cure", lines = { "/ma Cure <t>" } }
files[data.file()] = json.encode(other)
local theirs = files[data.file()]

-- A sticky-only edit leaves the world's file as the other client wrote it.
local id, why = data.save_entry("RDM", warp, { name = "Warp 2", lines = { "/echo warp" } })
assert(id == warp and not why)
assert(files[data.file()] == theirs, "a sticky-only edit leaves the world's file")

-- Moving the sticky macro into the job's list writes the world's file, read afresh first: their macro stays.
local moved = data.set_sticky("RDM", warp, false)
assert(moved and not data.is_sticky(moved))
data.load()
assert(data.get_entry("RDM", "90") and data.get_entry("RDM", "90").name == "Cure")
assert(data.get_entry("RDM", moved).name == "Warp 2")

-- A job macro's edit that the world's file did not take stays in memory; a sticky change after it keeps it (not the
-- file's older copy) and writes it.
local deny = false
data.init({ read = function(f) return files[f] end, decode = json.decode, encode = json.encode,
  write = function(f, t)
    if deny and f == data.file() then error("denied") end
    files[f] = t
  end })
data.load()
local job = data.save_entry("RDM", nil, { name = "Warp", lines = { "/echo" } })
deny = true
local _, jwhy = data.save_entry("RDM", job, { name = "Warp Home", lines = { "/echo" } })
assert(jwhy and data.get_entry("RDM", job).name == "Warp Home")
deny = false
local sid = data.save_entry("RDM", nil, { name = "Shared 2", lines = { "/echo" } }, true)
assert(sid and data.get_entry("RDM", job).name == "Warp Home")
data.load()
assert(data.get_entry("RDM", job).name == "Warp Home", "the kept edit was written with the sticky change")
