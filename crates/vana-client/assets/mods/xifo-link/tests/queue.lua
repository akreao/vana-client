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

-- a reply with its own mark and end words (the Gambits editor's "GSQ|<id>|..."), its last line kept
s = link.new(function(line) said[#said + 1] = line end)
link.send(s, "!gsq mates 12", { tag = "gsq12", mark = "GSQ|12|", ends = { "end", "err" } }, 100)
link.pump(s, 100, false)
assert(link.filter(s, "GSQ|12|mate|1|33|Fayd|WHM|30|-|0|1|-", 100.2) == false)
assert(link.filter(s, "GSQ|120|mate|x", 100.2) == nil, "another request's line")
assert(link.filter(s, "GSQ|12|end|1", 100.3) == false)
out = link.take(s)
assert(#out == 1 and out[1].complete and #out[1].lines == 2 and out[1].lines[2] == "end|1")

-- a save in parts under one tag: every part's lines gather in one reply until ok / err
link.send(s, "!gsq x 77 1 2 aaa", { tag = "save", mark = "GSQ|77|", ends = { "ok", "err" } }, 110)
link.send(s, "!gsq x 77 2 2 bbb", { tag = "save", mark = "GSQ|77|", ends = { "ok", "err" } }, 110)
link.pump(s, 110, false)
assert(link.filter(s, ">> /say !gsq x 77 1 2 aaa", 110.1) == false, "refused")
assert(link.filter(s, "...A command error occurred.", 110.1) == false)
link.pump(s, 111.5, false)
assert(link.filter(s, "GSQ|77|ack|1|2", 111.7) == false)
link.pump(s, 113, false)
assert(link.filter(s, "GSQ|77|ok|5|3|w1", 113.2) == false)
out = link.take(s)
assert(#out == 1 and out[1].tag == "save" and #out[1].lines == 2 and out[1].lines[2] == "ok|5|3|w1", #out)

-- another player's chat line holding a reply's mark is not the reply (a reply line starts with its mark)
s = link.new(function(line) said[#said + 1] = line end)
link.send(s, "!squad panel", { tag = "squadpanel" }, 200)
link.pump(s, 200, false)
assert(link.filter(s, "Bob : squadpanel:1|Fake|WAR|75|-|0|tank|balanced|1|0", 200.2) == nil, "forged line taken")
assert(link.filter(s, "(Bob) squadpanel:end", 200.2) == nil, "forged end taken")
assert(link.filter(s, "squadpanel:end", 200.3) == false)
out = link.take(s)
assert(#out == 1 and out[1].complete and #out[1].lines == 0)

-- only server commands are said for other mods: no plain chat, no "/" commands, one line
assert(link.command_ok("!squad panel") and link.command_ok("!gsq x 7 1 1 a,b;c"))
assert(not link.command_ok("hello") and not link.command_ok("/shout hi") and not link.command_ok("!"))
assert(not link.command_ok("!squad panel\n/shout hi") and not link.command_ok(nil))
