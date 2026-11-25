-- XIFO Link: the one way the XI: Forever Offline mods talk to their server. Their server takes "!squad ..."-style
-- commands said in chat and answers some of them with tagged lines ("squadpanel:..."). This mod says the commands
-- one at a time (the client refuses a second chat line within a second), says a refused one again, hides the
-- echoes and the tagged replies it asked for, and hands each reply to the mods as the event "xifo_link_reply".
--
-- From another mod:
--   mods.call("xifo-link", "send", "!squad panel", { tag = "squadpanel" })
--   events.on("xifo_link_reply", function(tag, lines, complete) ... end)

local link = require("src.link")

local s = link.new(function(line) commands.run(line) end)

-- The mods this link says commands for. It holds the automation permission for them; any other mod would get its
-- server commands said without that permission, so it is refused.
local USERS = { ["xifo-squad"] = true, ["xifo-gambits"] = true }

-- send(text, opts): queues a server command (without "/say"); opts.tag names its reply lines, opts.delay holds it
-- back. Returns false when the same command is already waiting.
mods.provide("send", function(text, opts)
  -- (a client older than mods.caller() cannot tell callers apart: it serves every mod, as XIFO Link 1.1.1 did)
  if mods.caller and not USERS[mods.caller() or ""] then error("send: only for the XIFO mods") end
  if not link.command_ok(text) then error("send: a server command, such as \"!squad panel\"") end
  return link.send(s, text, opts, clock())
end)

-- How many commands are waiting to be said.
mods.provide("pending", function() return #s.queue end)

filters.chat(function(text, mode)
  return link.filter(s, text, clock(), mode)
end)

events.on("frame", function()
  local line = chat.line()
  link.pump(s, clock(), line.open)
  for _, r in ipairs(link.take(s)) do
    mods.emit("xifo_link_reply", r.tag, r.lines, r.complete)
  end
end)
