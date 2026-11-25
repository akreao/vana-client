-- XIFO Link: one line per gap, a refused line said again (its ">> " echo and the command error hidden), the say echo
-- hidden, tagged replies gathered until their end line and hidden only while asked for.

local link = require("src.link")
local said = {}
local s = link.new(function(line) said[#said + 1] = line end)

-- two commands, the same one queued twice is kept once
assert(link.send(s, "!squad panel", { tag = "squadpanel" }, 0))
assert(link.send(s, "!squad role 1 tank", nil, 0))
assert(not link.send(s, "!squad role 1 tank", nil, 0))
link.pump(s, 0, false)
link.pump(s, 0.5, false)
assert(#said == 1 and said[1] == "/say !squad panel", said[1])
assert(link.filter(s, "Nymh : !squad panel", 0.1) == false, "say echo kept")

-- the reply: hidden and gathered up to its end line, the system message header before each line too
assert(link.filter(s, "----== SystemMessage ==----", 0.6) == false)
assert(link.filter(s, "squadpanel:1|Fayd|WHM|30|BLM|15|healer|balanced|1|15", 0.6) == false)
assert(link.filter(s, "squadpanel:camp|0|0|0|0", 0.6) == false)
assert(link.filter(s, "Some other line", 0.6) == nil)
assert(link.filter(s, "squadpanel:end", 0.7) == false)
local out = link.take(s)
assert(#out == 1 and out[1].tag == "squadpanel" and out[1].complete and #out[1].lines == 2, #out)
assert(out[1].lines[2] == "camp|0|0|0|0")

-- after the reply, a typed "!squad panel"'s lines show
assert(link.filter(s, "----== SystemMessage ==----", 0.8) == nil)
assert(link.filter(s, "squadpanel:end", 0.8) == nil)

-- not before the gap; not while the player types
link.pump(s, 1.0, false)
assert(#said == 1)
link.pump(s, 1.3, true)
assert(#said == 1)
link.pump(s, 1.3, false)
assert(said[2] == "/say !squad role 1 tank")

-- refused by the client: hidden, said again after the next gap
assert(link.filter(s, ">> /say !squad role 1 tank", 1.31) == false)
assert(link.filter(s, "...A command error occurred.", 1.31) == false)
assert(link.filter(s, "...A command error occurred.", 1.4) == nil, "a later error is not ours")
link.pump(s, 2.0, false)
assert(#said == 2)
link.pump(s, 2.6, false)
assert(said[3] == "/say !squad role 1 tank", said[3])

-- a refused line is given up after RETRIES
for i = 1, link.RETRIES do
  local t = 2.6 + i * 2
  assert(link.filter(s, ">> /say !squad role 1 tank", t - 1.9) == false)
  link.pump(s, t, false)
end
link.pump(s, 20, false)
assert(#s.queue == 0 and #said == 2 + link.RETRIES, #said) -- said 1 + RETRIES times in all

-- a reply without its end line is handed on unfinished after WAIT
link.send(s, "!squad sheet 2", { tag = "squadsheet" }, 30)
link.pump(s, 30, false)
assert(link.filter(s, "squadsheet:hdr|Fayd|WHM|30|-|0|auto|balanced|Tarutaru F|15", 30.2) == false)
link.pump(s, 30.2 + link.WAIT + 3, false)
out = link.take(s)
assert(#out == 1 and out[1].tag == "squadsheet" and not out[1].complete and #out[1].lines == 1)
