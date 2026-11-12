-- Chat Filter: a listed player, a listed word, a hidden kind and a repeat are dropped; others pass.

local S = { on = true, players = "Spammer, Bot", words = "gil selling", shouts = false, yells = true, repeats = true }
local F, clock = nil, 0
settings = { get = function(k) return S[k] end }
filters = { chat = function(f) F = f end }
timer = { every = function() end }
text = { strip = function(t) return t end }
os.clock = function() return clock end
load(TEST.source("init.lua"), "@init.lua")()
assert(F("Spammer : hi", 9) == false)
assert(F("[1]<Bot> hi", 14) == false)
assert(F("(Friend) hi", 13) == nil)
assert(F("Friend : cheap GIL SELLING here", 10) == false)
assert(F("Friend : party?", 11) == false)
assert(F("Friend : need a healer", 10) == nil)
clock = 30
assert(F("Friend : need a healer", 10) == false)
clock = 100
assert(F("Friend : need a healer", 10) == nil)
assert(F("The Forest Hare is paralyzed.", 36) == nil)
