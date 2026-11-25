-- The XIFO link's state, kept free of the client so the tests can drive it: a queue of server commands said one
-- at a time, the client's refusals sent again, and the tagged reply lines gathered per request.
--
-- The client says a chat line only when its last one is more than a second old (retail's c2s gate); a refused line
-- prints ">> <line>" and "...A command error occurred." instead of its echo. XI: Forever Offline answers its
-- "!squad panel"-style commands with lines like "squadpanel:<fields>" ending in "squadpanel:end".

local M = {}

M.GAP = 1.25       -- seconds between two lines we say
M.RETRIES = 3      -- a refused line is said again this many times
M.ECHO = 3         -- seconds after a send in which its echo or its refusal may come
M.WAIT = 8         -- seconds a reply's lines are hidden and gathered after its request
-- the chat modes of the server's lines: 0xc8 for s2c 0x017 kinds 6 / 7 ("----== SystemMessage ==----" then the text;
-- printToPlayer's default, the squad commands), 0x79 for kind 0x1d and the other text-only kinds (SYSTEM_3, "basic
-- system messages": the Gambits editor and the hardcore lines). Players cannot send either.
M.SERVER = { [0xc8] = true, [0x79] = true }

function M.new(run)
  return {
    run = run,       -- run(line): says the line
    queue = {},      -- { text, tag, at, tries }, oldest first
    last = -math.huge,
    sent = nil,      -- the last line said: { text, tag, at, tries }
    err_until = 0,   -- a "command error" line before this time belongs to our refusal
    waiting = {},    -- tag -> { till, lines, mark, ends }
    out = {},        -- finished replies: { tag, lines, complete }
    listening = {},  -- marks whose lines are always hidden and handed on, as the server sends them unasked
    heard = {},      -- { mark, text } since the last take
  }
end

-- Whether `text` is a server command this link says: "!" and a word, one line. Any mod may ask the link to send, so
-- it says nothing else (no plain chat, no "/" commands) under the player's name.
function M.command_ok(text)
  return type(text) == "string" and text:match("^![%w_]") ~= nil and not text:find("%c")
end

-- Queues a server command ("!squad panel"). `opts.tag` names the reply it brings ("squadpanel"); its lines are those
-- holding `opts.mark` (default "<tag>:"), and the one whose text after the mark is, or starts with, one of `opts.ends`
-- followed by "|" (default {"end"}) finishes it; that last line is kept unless it is a bare "end". `opts.wait`: seconds
-- the reply may take (default WAIT). `opts.delay` holds the command back. The same text already queued is not queued
-- twice. Commands sent under one tag while its reply is open add to that reply (a save sent in parts).
function M.send(s, text, opts, now)
  opts = opts or {}
  for _, q in ipairs(s.queue) do
    if q.text == text then return false end
  end
  s.queue[#s.queue + 1] = { text = text, tag = opts.tag, at = now + (opts.delay or 0), tries = 0,
    mark = opts.mark or (opts.tag and opts.tag .. ":"), ends = opts.ends or { "end" }, wait = opts.wait or M.WAIT }
  return true
end

local function finish(s, tag, complete)
  local w = s.waiting[tag]
  if not w then return end
  s.waiting[tag] = nil
  s.out[#s.out + 1] = { tag = tag, lines = w.lines, complete = complete }
end

-- Once a frame: gives up on replies that never ended, and says the next line when the gate allows. `typing`: the
-- player's chat line is open (a line of ours would make the player's own line too soon).
function M.pump(s, now, typing)
  for tag, w in pairs(s.waiting) do
    if now > w.till then finish(s, tag, false) end
  end
  local q = s.queue[1]
  if not q or q.at > now or now - s.last < M.GAP or typing then return end
  table.remove(s.queue, 1)
  s.last = now
  q.at = now
  s.sent = q
  if q.tag then
    local w = s.waiting[q.tag]
    if w then
      w.till = math.max(w.till, now + q.wait)
    else
      s.waiting[q.tag] = { till = now + q.wait, lines = {}, mark = q.mark, ends = q.ends }
    end
  end
  s.run("/say " .. q.text)
end

local function ends_with(text, tail)
  return #text >= #tail and text:sub(-#tail) == tail
end

-- A chat line on its way into the log, in chat mode `mode`: false hides it.
function M.filter(s, text, now, mode)
  local q = s.sent
  if q and now - q.at <= M.ECHO then
    if text:sub(1, 3) == ">> " and ends_with(text, q.text) then
      -- refused by the client's gate: said again after the next gap, ahead of the rest
      s.sent = nil
      s.err_until = now + M.ECHO
      if q.tries < M.RETRIES then
        q.tries = q.tries + 1
        q.at = now
        table.insert(s.queue, 1, q)
      end
      return false
    end
    if ends_with(text, " : " .. q.text) then return false end -- the say echo, "Name : !squad panel"
  end
  if now < s.err_until and text:find("A command error occurred", 1, true) then
    s.err_until = 0
    return false
  end
  -- A server line comes as two log lines: this header, then the text. While a reply is due the header is hidden too
  -- (a header whose text is not a reply line loses only the header).
  -- Only the server's lines (their mode) can be a reply: a player's chat is in its channel's mode, its continuation
  -- lines (a line break in the text) too. A reply line also starts with its mark.
  if not M.SERVER[mode] then return end
  if next(s.waiting) and text:find("^%-%-%-%-== SystemMessage ==%-%-%-%-$") then return false end
  -- (only a line that starts with the mark: a player's chat line naming it starts with the player's name)
  for mark in pairs(s.listening) do
    local body = text:gsub("^%s+", "")
    if body:sub(1, #mark) == mark then
      s.heard[#s.heard + 1] = { mark = mark, text = body:sub(#mark + 1) }
      return false
    end
  end
  for tag, w in pairs(s.waiting) do
    if text:sub(1, #w.mark) == w.mark then
      local payload = text:sub(#w.mark + 1)
      local head = payload:match("^([^|]*)")
      local last = false
      for _, e in ipairs(w.ends) do
        if payload == e or head == e then last = true end
      end
      if not last or payload ~= "end" then w.lines[#w.lines + 1] = payload end
      if last then
        finish(s, tag, true)
      else
        w.till = math.max(w.till, now + 2)
      end
      return false
    end
  end
end

-- Lines holding `mark` ("[HC-ICONS]") are hidden whenever they come, and handed on with the text after the mark.
function M.listen(s, mark)
  s.listening[mark] = true
end

-- The replies finished since the last call.
function M.take(s)
  local out, heard = s.out, s.heard
  s.out, s.heard = {}, {}
  return out, heard
end

return M
