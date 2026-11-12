-- Casket Solver: hint lines narrow 10..99; the answer comes when one is left.

local F, T, said = nil, nil, {}
settings = { get = function() return true end }
filters = { chat = function(f) F = f end }
timer = { every = function(_, f) T = f end }
chat = { add = function(t) said[#said + 1] = t end }
text = { strip = function(t) return t end }
load(TEST.source("init.lua"), "@init.lua")()
F("The monster was concealing a treasure chest!")
F("You have a hunch that the combination is greater than 40 and less than 60.")
T()
assert(said[#said] == "Casket Solver: 19 left. Try 50.", said[#said])
F("You have a hunch that the first digit is odd.")
F("You have a hunch that the second digit is 2, 7, or 9.")
T()
assert(said[#said] == "Casket Solver: 3 left: 52, 57, 59. Try 57.", said[#said])
F("You have a hunch that the lock's combination is greater than 57.")
T()
assert(said[#said] == "Casket Solver: the combination is 59.", said[#said])
