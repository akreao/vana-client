-- Tell Inbox: tells while /away are kept and each sender gets one away reply; none kept while online; back
-- online, the count is told; //r answers the last sender.

local S = { on = true, reply = "Back soon." }
local H, C, T, ran, said, away = {}, {}, {}, {}, {}, true
settings = { get = function(k) return S[k] end }
events = { on = function(n, f) H[n] = f end }
commands = { register = function(n, f) C[n] = f end, run = function(l) ran[#ran + 1] = l end }
timer = { every = function(_, f) T[#T + 1] = f end }
game = { player = function() return { name = "Tester", away = away } end }
text = { plain = function(t) return t end, strip = function(t) return t end }
chat = { add = function(t) said[#said + 1] = t end }
ui = { window = function() end, open = function() end, on = function() end, close = function() end }
storage = { character = {}, save = function() end }
load(TEST.source("init.lua"), "@init.lua")()
T[1]()
H.chat_message("are you there?", "Friend", 3, false)
H.chat_message("hello?", "Friend", 3, false)
assert(#storage.character.inbox == 2)
assert(#ran == 1 and ran[1] == "/tell Friend Back soon.", ran[1])
away = false
T[1]()
assert(said[#said]:find("2 tells"), said[#said])
H.chat_message("ok", "Other", 3, false)
assert(#storage.character.inbox == 2)
C.r("see", "you")
assert(ran[#ran] == "/tell Other see you", ran[#ran])
