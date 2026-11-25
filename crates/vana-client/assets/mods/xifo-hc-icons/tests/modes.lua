-- XIFO HC Icons: the server's lines read into names and modes.

local M = require("src.modes")
local k = {}
M.read(k, " !Nymh=1,Fayd=4")
assert(k.Nymh == 1 and k.Fayd == 4)
M.read(k, " Salia=2")
assert(k.Salia == 2 and k.Nymh == 1, "a later arrival is added")
M.read(k, " !Bea=3")
assert(k.Bea == 3 and k.Nymh == nil, "a full list replaces")
M.read(k, " Odd=9")
assert(k.Odd == nil, "an unknown mode is left out")
M.read(k, " !")
assert(next(k) == nil, "nobody hardcore here")
