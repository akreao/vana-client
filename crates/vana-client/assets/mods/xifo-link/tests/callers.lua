-- XIFO Link's send serves only the XIFO mods (mods.caller()): Squad, Gambits and HC Icons; any other mod is refused.

local provided, caller = {}, nil
mods = { provide = function(name, f) provided[name] = f end, caller = function() return caller end,
  emit = function() end }
commands = { run = function() end }
filters = { chat = function() end }
events = { on = function() end }
chat = { line = function() return { open = false } end }
clock = function() return 0 end
load(TEST.source("init.lua"), "@init.lua")()

for _, id in ipairs({ "xifo-squad", "xifo-gambits", "xifo-hc-icons" }) do
  caller = id
  assert(pcall(provided.send, "!hcicons " .. id), id .. " refused")
end
caller = "some-other-mod"
local ok, err = pcall(provided.send, "!squad dismiss all")
assert(not ok and tostring(err):find("only for the XIFO mods", 1, true), tostring(err))
caller = nil -- (called from outside a mods.call)
assert(not pcall(provided.send, "!squad dismiss all"))

-- listen too: a heard line counts as the server answering
caller = "xifo-hc-icons"
assert(pcall(provided.listen, "[HC-ICONS]"))
caller = "some-other-mod"
assert(not pcall(provided.listen, "[FAKE]"))
