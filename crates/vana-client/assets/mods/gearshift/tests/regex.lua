-- (Gearshift's XML front ends and regex: regex.)

local R = require("src.regex").test
local cases = {
  { "^.* x2$", "Fire x2", true }, { "^.* x2$", "Fire", false }, { "8[5-9]|9.", "87", true }, { "8[5-9]|9.", "84", false },
  { "8[5-9]|9.", "93", true }, { "^(Cure|Curaga)( I+V?)?$", "cure iii", true }, { "^(Cure|Curaga)( I+V?)?$", "Cura", false },
  { "a{2,3}b", "caaab", true }, { "^a{2}b", "aaab", false }, { "\\d+\\.\\d", "v2.45", true }, { "\\bfire\\b", "firestorm", false },
  { "\\bfire\\b", "the fire burns", true }, { "[^aeiou]x", "ax", false }, { "[^aeiou]x", "bx", true }, { "a.*?b", "aXbYb", true },
  { "Dynamis.*", "Dynamis - Bastok", true }, { "^$", "", true }, { "x?", "", true }, { "(ab)+c", "ababc", true },
}
for _, c in ipairs(cases) do
  local got = R(c[1], c[2])
  assert(got == c[3], c[1] .. " ~ " .. c[2] .. " gave " .. tostring(got))
end
assert(R("ABC", "abc", true) == false and R("ABC", "abc") == true)
assert(not pcall(R, "(ab", "ab"))
