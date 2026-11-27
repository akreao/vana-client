-- The game's macro files in a USER/<content id> folder: mcr.ttl / mcr_2.ttl (the 40 book titles) and mcr<N>.dat
-- (N = book * 10 + set, 0-based; book 1 set 1 is mcr.dat), each a container {u32 1, u32, MD5, payload}. A set is
-- 20 records of 0x17c bytes: u32, six 61-byte lines, a 10-byte title; one with no NUL in its title's first 9 bytes or
-- in a line is cleared, as the game does.

local md5 = require("src.md5")

local M = {}

local HEADER = 0x18
local RECORD = 0x17c
M.BOOKS, M.SETS = 40, 10

local function cstr(b) return (b:match("^([^\0]*)")) end

-- The payload of a container of `size` bytes whose MD5 matches, or nil (the client reads such a file as empty too).
local function payload(raw, size)
  if type(raw) ~= "string" or #raw ~= HEADER + size or string.unpack("<I4", raw) ~= 1 then return nil end
  local p = raw:sub(HEADER + 1)
  if md5.sum(p) ~= raw:sub(9, HEADER) then return nil end
  return p
end

local decode = function(b) return b end
function M.init(dec) decode = dec end

function M.file(book, set) -- 1-based
  local n = (book - 1) * 10 + (set - 1)
  return n == 0 and "mcr.dat" or ("mcr%d.dat"):format(n)
end

-- The 40 titles from the two title files' bytes (nil: missing), "BookNN" where there is none.
function M.titles(ttl1, ttl2)
  local out = {}
  for k = 1, 2 do
    local p = payload(k == 1 and ttl1 or ttl2, 0x140)
    for i = 0, 19 do
      local t = p and cstr(p:sub(16 * i + 1, 16 * i + 16)) or ""
      local b = (k - 1) * 20 + i + 1
      out[b] = t ~= "" and decode(t) or ("Book%02d"):format(b)
    end
  end
  return out
end

-- A set's 20 macros ({title, lines}, lines without the empty ones), or nil when the file is missing or damaged.
function M.set(raw)
  local p = payload(raw, 20 * RECORD)
  if not p then return nil end
  local out = {}
  for r = 0, 19 do
    local rec = p:sub(r * RECORD + 1, (r + 1) * RECORD)
    local title = rec:sub(0x173, 0x17c)
    local lines, ok = {}, title:sub(1, 9):find("\0", 1, true) ~= nil
    for k = 0, 5 do
      local l = rec:sub(5 + 61 * k, 4 + 61 * (k + 1))
      ok = ok and l:find("\0", 1, true) ~= nil
      local s = cstr(l)
      if s:match("%S") then lines[#lines + 1] = decode(s) end
    end
    out[r + 1] = ok and { title = decode(cstr(title)), lines = lines } or { title = "", lines = {} }
  end
  return out
end

-- Whether a macro holds anything.
function M.used(m) return m ~= nil and (m.title ~= "" or #m.lines > 0) end

-- A line that runs /lua (the player's to write: not copied).
function M.lua_line(l) return l:lower():match("^%s*/+lua") ~= nil end

return M
