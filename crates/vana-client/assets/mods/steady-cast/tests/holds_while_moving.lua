-- Steady Cast: a spell sent while moving waits with what follows it, and all of it goes in order once we stand
-- still; a spell sent standing goes at once; a hold ends after three seconds even while moving.

local hold = require("src.hold")
local function act(cat) return string.pack("<I2I2I4I2I2I4", 0x01A, 0, 200, 50, cat, 1) end
local equip = string.pack("<I2I2BBBB", 0x050, 0, 1, 4, 0, 0)
local sent = {}
local send = function(d) sent[#sent + 1] = d end
-- Moving: the cast and the midcast gear behind it wait.
hold.frame({ x = 0, z = 0 }, 1.00, send)
hold.frame({ x = 1, z = 0 }, 1.05, send)
assert(hold.out(0x01A, act(3), 1.06) == false and hold.out(0x050, equip, 1.06) == false and hold.held() == 2)
hold.frame({ x = 1, z = 0 }, 1.10, send)
assert(#sent == 0, "still too short")
hold.frame({ x = 1, z = 0 }, 1.25, send)
assert(#sent == 2 and sent[1] == act(3) and sent[2] == equip)
-- What we send again passes once.
assert(hold.out(0x01A, sent[1], 1.25) == nil and hold.out(0x050, sent[2], 1.25) == nil)
-- Standing: at once; a weapon skill never waits.
assert(hold.out(0x01A, act(3), 2.0) == nil)
hold.frame({ x = 2, z = 0 }, 2.1, send)
assert(hold.out(0x01A, act(7), 2.1) == nil)
-- Moving all along: three seconds, then it goes anyway.
sent = {}
assert(hold.out(0x01A, act(3), 2.1) == false)
for t = 2.2, 5.3, 0.1 do hold.frame({ x = t, z = 0 }, t, send) end
assert(#sent == 1)
