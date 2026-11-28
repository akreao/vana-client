-- Retail Macros: the game's macro files read as the game reads them: titles (BookNN where none), sets of 20 records,
-- a damaged record cleared, a damaged or missing file nil (its MD5 wrong too), /lua lines told apart.

local mcr = require("src.mcr")

local function container(p) return string.pack("<I4I4", 1, 0) .. require("src.md5").sum(p) .. p end
local function record(title, lines)
  local r = string.rep("\0", 4)
  for k = 1, 6 do
    local l = lines[k] or ""
    r = r .. l .. string.rep("\0", 61 - #l)
  end
  return r .. title .. string.rep("\0", 10 - #title)
end

assert(mcr.file(1, 1) == "mcr.dat" and mcr.file(1, 2) == "mcr1.dat" and mcr.file(3, 10) == "mcr29.dat")
assert(mcr.file(40, 10) == "mcr399.dat")

local ttl = string.rep("\0", 16) .. "WAR" .. string.rep("\0", 13) .. string.rep("\0", 16 * 18)
local titles = mcr.titles(container(ttl), nil)
assert(#titles == 40 and titles[1] == "Book01" and titles[2] == "WAR" and titles[21] == "Book21", titles[2])
-- The first file missing: books 1 to 20 have no titles, the second file's stay books 21 to 40.
titles = mcr.titles(nil, container(ttl))
assert(titles[2] == "Book02" and titles[22] == "WAR", titles[2])

local recs = { record("Provoke", { '/ja "Provoke" <t>' }), record("123456789X", { "/p bad" }) }
for _ = 3, 20 do recs[#recs + 1] = record("", {}) end
local set = mcr.set(container(table.concat(recs)))
assert(set and #set == 20, "a set is 20 macros")
assert(set[1].title == "Provoke" and #set[1].lines == 1 and set[1].lines[1] == '/ja "Provoke" <t>')
assert(not mcr.used(set[2]), "a title without a NUL in 9 bytes is not cleared")
assert(not mcr.used(set[3]))
assert(mcr.set(nil) == nil and mcr.set(container("short")) == nil, "a damaged file is read")
assert(mcr.set(string.pack("<I4", 2) .. container(table.concat(recs)):sub(5)) == nil, "another version is read")
-- A set whose MD5 does not match is damaged, as the client reads it.
local good = container(table.concat(recs))
assert(mcr.set(good:sub(1, 8) .. string.rep("\0", 16) .. good:sub(25)) == nil, "a set with a wrong MD5 is read")

-- RFC 1321's vectors.
local md5 = require("src.md5")
local function hex(b) return (b:gsub(".", function(c) return ("%02x"):format(c:byte()) end)) end
assert(hex(md5.sum("")) == "d41d8cd98f00b204e9800998ecf8427e")
assert(hex(md5.sum("abc")) == "900150983cd24fb0d6963f7d28e17f72")
assert(hex(md5.sum(string.rep("1234567890", 8))) == "57edf4a22be3c955ac49da2e2107b67a")

assert(mcr.lua_line("/lua r hotbar") and mcr.lua_line("  //LUA load x") and not mcr.lua_line("/ma Cure <t>"))
