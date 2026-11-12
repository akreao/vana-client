-- Calculator: precedence, brackets, powers, k / m, commas in the answer; bad input is an error line, never run.

local C, said = {}, {}
settings = { get = function() return true end }
commands = { register = function(n, f) C[n] = f end }
chat = { add = function(t) said[#said + 1] = t end }
ui = { window = function() end, open = function() end, on = function() end, update = function() end }
load(TEST.source("init.lua"), "@init.lua")()
local function calc(s) C.calc(s) return said[#said] end
assert(calc("12 * 99 + 450") == "12 * 99 + 450 = 1,638", said[#said])
assert(calc("(2+3)*4") == "(2+3)*4 = 20")
assert(calc("2^10") == "2^10 = 1,024")
assert(calc("1.5k / 4") == "1.5k / 4 = 375")
assert(calc("2m - 1") == "2m - 1 = 1,999,999")
assert(calc("10 / 4") == "10 / 4 = 2.5")
assert(calc("os.exit()"):find("^Calculator: "))
assert(calc("1/0"):find("no answer"))
