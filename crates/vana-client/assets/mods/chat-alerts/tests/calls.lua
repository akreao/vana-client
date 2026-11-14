-- Chat Alerts: a tell, our name and a chosen word call; our lines, other people's names and a second call
-- within a second do not.

local S = { on = true, sound = 2, tells = true, mentions = true, party = false, linkshell = false, invites = true,
  examined = false, emotes = false, away = false, words = "rare drop, nm" }
local H, calls, clock = {}, {}, 0
settings = { get = function(k) return S[k] end }
events = { on = function(n, f) H[n] = f end }
audio = { play_ui = function(id) calls[#calls + 1] = id end }
game = { player = function() return { name = "Tester", id = 7 } end }
text = { plain = function(t) return t end }
os.clock = function() return clock end
load(TEST.source("init.lua"), "@init.lua")()
local function heard(line, sender, kind)
  local n = #calls
  clock = clock + 2
  H.chat_message(line, sender, kind, false)
  return #calls > n
end
assert(heard("hi", "Friend", 3))
assert(not heard("hi", "Tester", 3))
assert(heard("Tester, over here", "Friend", 0))
assert(not heard("Testers unite", "Friend", 0))
assert(heard("an NM popped", "Friend", 1))
assert(not heard("nothing", "Friend", 4))
assert(calls[1] == 19)
-- Two tells in one moment: one call.
clock = clock + 2
H.chat_message("hi", "Friend", 3, false)
H.chat_message("hi again", "Friend", 3, false)
assert(#calls == 4, #calls)
