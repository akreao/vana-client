-- XIFO Squad's window: a panel answer fills it, and its controls send the server's "!squad" commands through XIFO Link.

local sent, windows, handlers, ev, cmd, open = {}, {}, {}, {}, {}, {}
local T = 0
clock = function() return T end
log = { warn = function() end }
mods = { call = function(id, name, text, opts)
  assert(id == "xifo-link")
  if name == "pending" then return 0 end
  sent[#sent + 1] = { text = text, tag = opts.tag, delay = opts.delay }
  return true
end }
events = { on = function(name, f) ev[name] = f end }
timer = { every = function() end }
res = {
  item = function(id) return id == 16640 and { name = "Bronze Axe" } or nil end,
  spell = function(id) return id == 4 and { name = "Cure IV" } or nil end,
  ability = function() return nil end,
}
commands = { register = function(n, f) cmd[n] = f end }
ui = {
  window = function(id, spec)
    windows[id] = {}
    for _, e in ipairs(spec.elements) do if e.id then windows[id][e.id] = e end end
  end,
  update = function(id, el, fields) for k, v in pairs(fields) do windows[id][el][k] = v end end,
  on = function(id, f) handlers[id] = f end,
  open = function(id) open[id] = true end,
  close = function(id) open[id] = nil end,
  is_open = function(id) return open[id] == true end,
}
load(TEST.source("init.lua"), "@init.lua")()

local function last() return sent[#sent].text end

cmd.squad()
assert(open.squad and last() == "!squad panel" and sent[#sent].tag == "squadpanel")

ev.xifo_link_reply("squadpanel", { "1|Fayd|WHM|30|BLM|15|healer|balanced|1|15", "jl|1|BLM:15,WHM:30",
  "2|Salia|WAR|28|-|0|tank|aggressive|0|5", "camp|0|0|0|0", "campbots|Bea|-" }, true)
local w = windows.squad
assert(#w.mates.rows == 2 and w.mates.rows[1][3] == "WHM 30/BLM 15" and w.mates.rows[2].colour)
assert(w.status.text == "2 squad mates, 1 out.", w.status.text)
assert(w.role.at == 4 and w.mode.at == 1 and w.sw.items[1] == "Melee: on")
assert(w.job.items[3] == "WHM 30" and w.job.at == 3 and w.sub.at == 5)
assert(#w.puller.items == 4 and w.puller.items[4] == "Bea (bot)")

local H = handlers.squad
H("squad", { kind = "select", id = "mates", index = 2 })
assert(w.sel.text:find("Salia", 1, true) and w.role.at == 2 and w.sub.at == 1)
assert(w.sw.items[2] == "Ranged: off")
H("squad", { kind = "set", id = "mode", index = 3 })
assert(sent[#sent - 1].text == "!squad mode 2 conserve" and last() == "!squad panel")
H("squad", { kind = "press", id = "sw", index = 2 })
assert(last() == "!squad sw 2 ranged on" and w.sw.items[2] == "Ranged: on")
H("squad", { kind = "set", id = "sub", index = 4 })
assert(sent[#sent - 1].text == "!squad sub 2 WHM")
H("squad", { kind = "set", id = "job", index = 1 })
assert(sent[#sent - 1].text == "!squad job 2 WAR")
H("squad", { kind = "press", id = "mate", index = 1 })
assert(sent[#sent - 1].text == "!squad call 2")
H("squad", { kind = "press", id = "all", index = 3 })
assert(sent[#sent - 2].text == "!squad dismiss" and sent[#sent - 1].text == "!squad call all")
assert(sent[#sent - 1].delay == 2.5)
H("squad", { kind = "set", id = "puller", index = 4 })
assert(sent[#sent - 1].text == "!squad pull Bea")
H("squad", { kind = "set", id = "puller", index = 2 })
assert(sent[#sent - 1].text == "!squad pull 1")

-- the profile
H("squad", { kind = "press", id = "mate", index = 3 })
assert(open.squad_sheet and last() == "!squad sheet 2" and sent[#sent].tag == "squadsheet")
ev.xifo_link_reply("squadsheet", { "hdr|Salia|WAR|28|-|0|tank|aggressive|Mithra|5", "job|WAR|28",
  "exp|WAR|28|100|4000", "note|Not called right now.", "eq|0|main|bronze_axe|16640", "eq|4|head|iron_mask|12345",
  "spell|cure_iv|88", "ability|provoke|5" }, true)
local s = windows.squad_sheet
assert(s.hdr.text == "Salia: WAR 28", s.hdr.text)
assert(s.hp.text == "not called" and s.exp.text == "100 / 4000")
assert(s.gear.rows[1][2] == "Bronze Axe" and s.gear.rows[1].icon.id == 16640)
assert(s.gear.rows[2][2] == "Iron Mask", "no record: the database name tidied")
assert(s.acts.rows[1][1] == "Cure IV" and s.acts.rows[2][1] == "Provoke")
assert(s.stats.rows[1][1]:find("Call this mate", 1, true))

-- no server answer
ev.xifo_link_reply("squadpanel", {}, false)
assert(w.status.text:find("XI: Forever Offline", 1, true))

-- Servers from v0.3.11: the picked mate's Rules line, a mark on a mate whose next call changes its rules, and a
-- profile answer for another mate (picked again meanwhile) dropped.
ev.xifo_link_reply("squadpanel", { "1|Fayd|WHM|30|BLM|15|healer|balanced|1|15", "rs|1|pending|choice|0|-|4|Heals|0|0|WHM",
  "2|Salia|WAR|28|-|0|tank|aggressive|0|5", "rs|2|off|-|0|-|4|Heals|0|0|-" }, true)
H("squad", { kind = "select", id = "mates", index = 1 })
assert(w.mates.rows[1][6] == "Out*" and w.mates.rows[2][6] == "", w.mates.rows[1][6])
assert(w.mates.rows[1].tip == w.rules.text and w.mates.rows[2].tip == "Rules: set 4 Heals at its next call")
-- (no "cap|reapply1": no Apply button for the mate, so the hint names Apply now)
assert(w.rules.text == "Rules: its job rules now, set 4 Heals from its next call (another set was chosen; Apply now)",
  w.rules.text)
H("squad", { kind = "select", id = "mates", index = 2 })
assert(w.rules.text == "Rules: set 4 Heals at its next call", w.rules.text)
H("squad", { kind = "press", id = "mate", index = 3 })
assert(last() == "!squad sheet 2")
ev.xifo_link_reply("squadsheet", { "slot|1", "hdr|Fayd|WHM|30|BLM|15|healer|balanced|Elvaan|15" }, true)
assert(s.hdr.text ~= "Fayd: WHM 30 / BLM 15", "another mate's answer is dropped")
ev.xifo_link_reply("squadsheet", { "slot|2", "hdr|Salia|WAR|28|-|0|tank|aggressive|Mithra|5" }, true)
assert(s.hdr.text == "Salia: WAR 28", s.hdr.text)

-- A server that can re-call one mate (cap|reapply1): an out mate gets Apply, which sends !squad reapply <slot>; an
-- away mate, or a server without the word, does not.
ev.xifo_link_reply("squadpanel", { "1|Fayd|WHM|30|BLM|15|healer|balanced|1|15", "rs|1|pending|choice|0|-|4|Heals|0|0|WHM",
  "2|Salia|WAR|28|-|0|tank|aggressive|0|5", "cap|reapply1" }, true)
H("squad", { kind = "select", id = "mates", index = 1 })
assert(w.rules.text:find("another set was chosen; Apply)", 1, true), "the mate's own Apply: " .. w.rules.text)
assert(table.concat(w.mate.items, ",") == "Call,Dismiss,Apply,Profile", table.concat(w.mate.items, ","))
H("squad", { kind = "press", id = "mate", index = 3 })
assert(sent[#sent - 1].text == "!squad reapply 1" and last() == "!squad panel" and sent[#sent].delay == 6)
H("squad", { kind = "press", id = "mate", index = 4 })
assert(last() == "!squad sheet 1")
H("squad", { kind = "select", id = "mates", index = 2 })
assert(table.concat(w.mate.items, ",") == "Call,Dismiss,Profile")
ev.xifo_link_reply("squadpanel", { "1|Fayd|WHM|30|BLM|15|healer|balanced|1|15" }, true)
H("squad", { kind = "select", id = "mates", index = 1 })
assert(table.concat(w.mate.items, ",") == "Call,Dismiss,Profile", "no reapply1: no Apply")
