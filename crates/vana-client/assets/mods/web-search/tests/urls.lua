-- Web Search: the words given, else the item under a list's cursor, else the target, go to the site, encoded.

local C, opened, high, target = {}, {}, nil, nil
settings = { get = function() return true end }
commands = { register = function(n, f) C[n] = f end }
net = { open_url = function(u) opened[#opened + 1] = u end }
chat = { add = function() end }
ui = { native = { highlighted = function() return high end } }
game = { target = function() return target end }
res = { item = function(id) return id == 4096 and { name = "Fire Crystal" } or nil end }
load(TEST.source("init.lua"), "@init.lua")()
C.wiki("Kupipi's", "Trust")
assert(opened[1] == "https://www.bg-wiki.com/index.php?search=Kupipi%27s%20Trust", opened[1])
high = { kind = "item", id = 4096 }
C.ah()
assert(opened[2] == "https://www.ffxiah.com/search/item?q=Fire%20Crystal", opened[2])
high, target = nil, { name = "Forest Hare" }
C.wiki()
assert(opened[3] == "https://www.bg-wiki.com/index.php?search=Forest%20Hare", opened[3])
