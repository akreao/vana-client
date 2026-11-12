-- Multibox Control: //send, //sendtarget and //stna go out as ipc lines; a window that gets them acts.

local C, E, sent, ran, cast, said = {}, {}, {}, {}, {}, {}
settings = { get = function() return true end }
commands = { register = function(n, f) C[n] = f end, run = function(l) ran[#ran + 1] = l end }
events = { on = function(n, f) E[n] = f end }
chat = { add = function(t) said[#said + 1] = t end }
ipc = { send = function(t, to) sent[#sent + 1] = (to or "*") .. "|" .. t end }
client = { focus = function() ran[#ran + 1] = "focus" end }
actions = { cast = function(s, who) cast[#cast + 1] = s .. ">" .. who end, target = function(id) ran[#ran + 1] = "target " .. id end }
game = {
  player = function() return { name = "Mage" } end,
  target = function() return { id = 1234 } end,
  spells = function() return { 14, 15, 20, 143 } end,
  recasts = function() return { spells = {} } end,
  party = function() return {
    { slot = 0, name = "Mage", in_zone = true, hp = 500, buffs = { 3 } },
    { slot = 1, name = "Tank", in_zone = true, hp = 900, buffs = { 4, 13 } },
  } end,
}
load(TEST.source("init.lua"), "@init.lua")()
C.send("Tank", "/ja", "Provoke", "<t>")
C.sendtarget()
C.stna("Mage")
assert(sent[1] == "Tank|run\t/ja Provoke <t>", sent[1])
assert(sent[2] == "*|target\t1234", sent[2])
assert(cast[1] == "15>Tank", tostring(cast[1]))
E.ipc_message("run\t/heal", "Mage")
E.ipc_message("target\t77", "Mage")
E.ipc_message("focus\t", "Mage")
assert(table.concat(ran, ",") == "/heal,target 77,focus", table.concat(ran, ","))
E.chat_message("hello", "Friend", 3, false)
assert(sent[3] == "*|tell\t[Mage] Friend>> hello", sent[3])
