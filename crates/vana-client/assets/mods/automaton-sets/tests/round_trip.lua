-- Automaton Sets: a saved set changes only the slots that differ: a different attachment off, then the head,
-- the frame and the set's attachments on.

local C, ops, said = {}, {}, {}
local job = { job = 18, head = 8193, frame = 8226, attachments = { 8449, 8450 }, attachment_slots = { 8449, 8450, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 } }
settings = { get = function() return true end }
commands = { register = function(n, f) C[n] = f end }
chat = { add = function(t) said[#said + 1] = t end }
storage = { account = {}, save = function() end }
sleep = function() end
res = { item = function(id) return { name = "item" .. id } end }
actions = { attach = function(slot, id) ops[#ops + 1] = "a" .. slot .. ":" .. id end,
  detach = function(slot) ops[#ops + 1] = "d" .. slot end }
game = { job_data = function() return { main = job } end, entity_by_tag = function() return nil end }
load(TEST.source("init.lua"), "@init.lua")()
C.automaton("save", "Tank")
assert(storage.account.sets.tank.head == 8193)
job = { job = 18, head = 8194, frame = 8227, attachments = { 8451 }, attachment_slots = { 0, 0, 8451, 0, 0, 0, 0, 0, 0, 0, 0, 0 } }
C.automaton("tank")
local s = table.concat(ops, " ")
assert(s == "d5 a1:8193 a2:8226 a3:8449 a4:8450", s)
assert(said[#said]:find("but not"), said[#said])
