-- Battle Text's short lines: the common forms; an unknown one stays as printed; names in their colours.

local M = load(TEST.source("init.lua"), "@init.lua")()
local function who(name, pc, colour) return { name = name, pc = pc, colour = colour } end
local me, rabbit = who("Lnxcsfix", true, 0), who("Wild Rabbit", false, 0)
local function check(lines, a, t, want)
  local got = M.shorten(lines, a, t)
  assert(got == want, tostring(got))
end
check({ "Lnxcsfix uses Fast Blade.", "The Wild Rabbit takes 123 points of damage." }, me, rabbit,
  "Lnxcsfix [Fast Blade] → Wild Rabbit: 123")
check({ "Lnxcsfix hits the Wild Rabbit for 18 points of damage." }, me, rabbit, "Lnxcsfix → Wild Rabbit: 18")
check({ "Lnxcsfix scores a critical hit!", "The Wild Rabbit takes 31 points of damage." }, me, rabbit,
  "Lnxcsfix → Wild Rabbit: 31!")
check({ "Lnxcsfix misses the Wild Rabbit." }, me, rabbit, "Lnxcsfix → Wild Rabbit: miss")
check({ "Lnxcsfix casts Dia.", "The Wild Rabbit gains the effect of dia." }, me, rabbit,
  "Lnxcsfix [Dia] → Wild Rabbit: +Dia")
check({ "Lnxcsfix casts Fire.", "Magic Burst! The Wild Rabbit takes 80 points of damage." }, me, rabbit,
  "Lnxcsfix [Fire] → Wild Rabbit: 80 (Magic Burst)")
check({ "The Wild Rabbit takes 48 points of damage." }, me, rabbit, "Wild Rabbit: 48")
check({ "Lnxcsfix defeats the Wild Rabbit." }, me, rabbit, nil)
check({ "Lnxcsfix uses Provoke on the Wild Rabbit." }, me, rabbit, "Lnxcsfix [Provoke] → Wild Rabbit")
check({ "Lnxcsfix casts Cure.", "Lnxcsfix recovers 45 HP." }, me, me, "Lnxcsfix [Cure] → Lnxcsfix: +45 HP")
-- A name in its colour: the line's text unchanged, the colour's code in it.
local c = M.shorten({ "Lnxcsfix misses the Wild Rabbit." }, me, who("Wild Rabbit", false, 8))
assert(text.strip(c) == "Lnxcsfix → Wild Rabbit: miss", c)
local red = utf8.char(utf8.codepoint(text.colour("x", 8), 1))
assert(c:find(red, 1, true), "the name's colour code")
