-- MD5 (RFC 1321) of a string, as 16 raw bytes: the macro containers' check (the client's reader checks it too).

local M = {}

local MASK = 0xffffffff

local S = {
  7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
  5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
  4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
  6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
}

local K = {}
for i = 0, 63 do K[i + 1] = math.floor(math.abs(math.sin(i + 1)) * 2 ^ 32) & MASK end

local function rotl(x, n) return ((x << n) | (x >> (32 - n))) & MASK end

function M.sum(msg)
  local len = #msg
  msg = msg .. "\x80" .. string.rep("\0", (55 - len) % 64) .. string.pack("<I8", (len * 8) & 0xffffffffffffffff)
  local a0, b0, c0, d0 = 0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476
  for chunk = 1, #msg, 64 do
    local m = { string.unpack("<" .. string.rep("I4", 16), msg, chunk) }
    local a, b, c, d = a0, b0, c0, d0
    for i = 0, 63 do
      local f, g
      if i < 16 then
        f, g = (b & c) | (~b & d), i
      elseif i < 32 then
        f, g = (d & b) | (~d & c), (5 * i + 1) % 16
      elseif i < 48 then
        f, g = b ~ c ~ d, (3 * i + 5) % 16
      else
        f, g = c ~ (b | ~d), (7 * i) % 16
      end
      f = (f + a + K[i + 1] + m[g + 1]) & MASK
      a, d, c = d, c, b
      b = (b + rotl(f, S[i + 1])) & MASK
    end
    a0, b0, c0, d0 = (a0 + a) & MASK, (b0 + b) & MASK, (c0 + c) & MASK, (d0 + d) & MASK
  end
  return string.pack("<I4I4I4I4", a0, b0, c0, d0)
end

return M
