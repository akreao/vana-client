-- Retail Macros: an install found is read only once allowed; its characters listed (this one's first); books picked
-- by click; Copy to Macro Palettes hands the picked books to actions.macro_import; Send to Hotbars hands their macros,
-- /lua lines left out, to Hotbars by the job each book names.

local function container(p) return string.pack("<I4I4", 1, 0) .. require("src.md5").sum(p) .. p end
local function record(title, lines)
  local r = string.rep("\0", 4)
  for k = 1, 6 do
    local l = lines[k] or ""
    r = r .. l .. string.rep("\0", 61 - #l)
  end
  return r .. title .. string.rep("\0", 10 - #title)
end
local function set_of(...)
  local recs = { ... }
  for _ = #recs + 1, 20 do recs[#recs + 1] = record("", {}) end
  return container(table.concat(recs))
end

-- A retail install with two characters; 1234's book 1 is "WHM" (two macros), book 2 "Ninja" (one, and a /lua line).
local ROOT = "/retail/FINAL FANTASY XI"
local FILES = {
  [ROOT .. "/USER/abc/mcr.ttl"] = container(string.rep("\0", 0x140)),
  [ROOT .. "/USER/1234/mcr.ttl"] = container("WHM" .. string.rep("\0", 13) .. "Ninja" .. string.rep("\0", 11)
    .. string.rep("\0", 16 * 18)),
  [ROOT .. "/USER/1234/mcr.dat"] = set_of(record("Cure", { '/ma "Cure" <t>' }), record("Haste", { '/ma "Haste" <t>' })),
  [ROOT .. "/USER/1234/mcr10.dat"] = set_of(record("Utsu", { '/ma "Utsusemi: Ichi" <me>', "/lua r x" })),
}
local ALLOWED = {}
local ASKED = 0
local function allowed(p)
  for root in pairs(ALLOWED) do
    if p:sub(1, #root) == root then return true end
  end
  error(p .. ": not allowed", 0)
end
fs.game_installs = function() return { { path = ROOT, source = "test", own = false } } end
fs.open_folder = function(p, _, h)
  ASKED = ASKED + 1
  ALLOWED[p] = true
  h(true)
end
fs.read_outside = function(p) allowed(p) return FILES[p] end
fs.exists_outside = function(p)
  allowed(p)
  if FILES[p] then return true end
  for k in pairs(FILES) do
    if k:sub(1, #p + 1) == p .. "/" then return true end
  end
  return false
end
fs.list_outside = function(p)
  allowed(p)
  local seen, out = {}, {}
  for k in pairs(FILES) do
    local rest = k:sub(1, #p + 1) == p .. "/" and k:sub(#p + 2)
    local name = rest and (rest:match("^([^/]+)/") and rest:match("^([^/]+)/") .. "/" or rest)
    if name and not seen[name] then seen[name], out[#out + 1] = true, name end
  end
  table.sort(out)
  return out
end
game = setmetatable({ player = function() return { id = 0x1234, main_job = 6 } end },
  { __index = function() return function() end end })
res = setmetatable({ job = function(id) return ({ [6] = "THF" })[id] end },
  { __index = function() return function() end end })
local IMPORT
actions.macro_import = function(folder, books) IMPORT = { folder = folder, books = books } end
ui.prompt = function(_, _, h) h(1) end
local CALLS = {}
mods.list = function() return { { id = "hotbar" } } end
mods.call = function(mod, name, job, list, shared)
  CALLS[#CALLS + 1] = { mod = mod, name = name, job = job, list = list, shared = shared }
  return #list, 0
end
chat = { add = function() end }
local H
local on = ui.on
ui.on = function(id, f) H = f on(id, f) end
local CMD
commands.register = function(_, f) CMD = f end

local ok, err = pcall(require, "init")
assert(ok, tostring(err))
CMD()
assert(ASKED == 0, "read before it was allowed")
H("retailmacros", { kind = "set", id = "install", index = 2 })
assert(ASKED == 1 and ALLOWED[ROOT], "the install was not asked for")
-- This character's folder (0x1234) is the one chosen; book 1 and 2 picked by click.
H("retailmacros", { kind = "select", id = "books", index = 1 })
H("retailmacros", { kind = "select", id = "books", index = 2 })
H("retailmacros", { kind = "press", id = "act", index = 2 })
assert(IMPORT and IMPORT.folder == ROOT .. "/USER/1234", IMPORT and IMPORT.folder)
assert(#IMPORT.books == 2 and IMPORT.books[1] == 1 and IMPORT.books[2] == 2)
-- Hotbars: WHM gets Cure and Haste, NIN gets Utsu without its /lua line.
H("retailmacros", { kind = "press", id = "act", index = 3 })
table.sort(CALLS, function(a, b) return a.job < b.job end)
assert(#CALLS == 2 and CALLS[1].job == "NIN" and CALLS[2].job == "WHM", #CALLS .. " " .. tostring(CALLS[1].job))
assert(CALLS[1].mod == "hotbar" and CALLS[1].name == "import_macros" and not CALLS[1].shared)
assert(#CALLS[2].list == 2 and CALLS[2].list[1].name == "Cure")
assert(#CALLS[1].list == 1 and #CALLS[1].list[1].lines == 1, "the /lua line went to Hotbars")
-- Book 2 alone, its job changed to Shared: the shared library, through the player's main job.
CALLS = {}
H("retailmacros", { kind = "select", id = "books", index = 1 })
H("retailmacros", { kind = "select", id = "books", index = 2 })
H("retailmacros", { kind = "select", id = "books", index = 2 })
H("retailmacros", { kind = "set", id = "job", index = 23 })
H("retailmacros", { kind = "press", id = "act", index = 3 })
assert(#CALLS == 1 and CALLS[1].shared == true and CALLS[1].job == "THF", tostring(CALLS[1] and CALLS[1].job))
