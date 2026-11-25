-- XIFO Gambits' window against a scripted server: hello, the squad, a mate's sets and choices, opening a set,
-- editing a rule, saving it in parts, assigning it.

local sent, windows, handlers, ev, cmd, open, prompts = {}, {}, {}, {}, {}, {}, {}
local timers, delays, queued = {}, {}, 0 -- timer.after's callbacks and delays; XIFO Link's waiting commands
mods = { call = function(id, name, text, opts)
  assert(id == "xifo-link")
  if name == "pending" then return queued end
  assert(name == "send")
  sent[#sent + 1] = { text = text, tag = opts.tag, mark = opts.mark, ends = opts.ends }
  return true
end }
events = { on = function(name, f) ev[name] = f end }
timer = { after = function(s, f) timers[#timers + 1] = f; delays[#timers] = s end }
commands = { register = function(n, f) cmd[n] = f end }
ui = {
  get = function(id, el) return { text = windows[id][el].text } end,
  window = function(id, spec)
    windows[id] = {}
    for _, e in ipairs(spec.elements) do if e.id then windows[id][e.id] = e end end
  end,
  update = function(id, el, fields)
    for k, v in pairs(fields) do
      if k == "items" then for i, x in ipairs(v) do assert(type(x) == "string", "item " .. i) end end
      windows[id][el][k] = v
    end
  end,
  on = function(id, f) handlers[id] = f end,
  open = function(id) open[id] = true end,
  close = function(id) open[id] = nil end,
  is_open = function(id) return open[id] == true end,
  prompt = function(text, buttons, f) prompts[#prompts + 1] = { text = text, f = f } end,
}
load(TEST.source("init.lua"), "@init.lua")()
local w, rw = windows.gambits, windows.gambits_rule
local H = function(e) handlers.gambits("gambits", e) end
local HR = function(e) handlers.gambits_rule("gambits_rule", e) end

-- the server answers the last request
local function answer(lines, complete)
  local s = sent[#sent]
  ev.xifo_link_reply(s.tag, lines, complete ~= false)
end
local function last() return sent[#sent].text end

cmd.gambits()
assert(open.gambits and last():match("^!gsq hello %d+$"))
assert(sent[#sent].mark == "GSQ|" .. last():match("(%d+)$") .. "|")
answer({ "hello|9|32|120|64" })
assert(last():match("^!gsq mates %d+$"))
answer({ "mate|1|40|Fayd|WHM|30|BLM|4|1|15", "mate|2|41|Salia|WAR|28|-|0|0|-", "end|2" })
assert(w.mate.items[1] == "Fayd  WHM 30/BLM 15  (out)", w.mate.items[1])

-- the set list and the catalog were both asked for; answer each by its own tag
local list_req, cat_req = sent[#sent - 1], sent[#sent]
assert(list_req.text:match("^!gsq list %d+ WHM$") and cat_req.text:match("^!gsq catm %d+ 1 jm$"), cat_req.text)
ev.xifo_link_reply(list_req.tag, { "set|4|WHM|2|o|3|w4|Fayd heals", "set|1|WHM|5|m|1|whm1|Default WHM", "end|2" }, true)
assert(#w.sets.rows == 2 and w.sets.rows[1][4] == "In use" and w.sets.rows[2][4] == "Copy only")
local G = require("src.gsq")
local cat = { "cat|1|t|SELF,PARTY,TANK", "cat|2|c|ALWAYS:n,HPP_LT:r:0:100,NOT_STATUS:e", "cat|3|f|CURE,PROTECT",
  "cat|4|j|DIVINE_SEAL:15", "cat|5|s|CURE:1,PROTECT:7", "cat|6|u|CURE_II:11" }
local h = 7
for _, line in ipairs(cat) do
  local kind, list = line:match("^cat|%d+|(%a)|(.*)$")
  for e in list:gmatch("[^,]+") do h = G.check(kind .. ":" .. e .. "\n", h) end
end
cat[#cat + 1] = "end|6|3.3.2.1.2.1|" .. h
ev.xifo_link_reply(cat_req.tag, cat, true)

-- open set 4
H({ kind = "select", id = "sets", index = 1 })
H({ kind = "press", id = "setbtn", index = 1 })
assert(last():match("^!gsq get %d+ 4$"))
answer({ "hdr|4|WHM|o|3|w4|Fayd heals", "rule|1|1/1|1,PARTY,HPP_LT,50,,,f,CURE,", "rule|2|1/1|1,SELF,ALWAYS,,,,r,,",
  "end|2" })
assert(#w.rules.rows == 2 and w.rules.rows[1][3] == "Party member" and w.rules.rows[1][4] == "HP below 50%")
assert(rw.target.items[rw.target.at] == "Party member" and rw.cond.items[rw.cond.at] == "HP below %")
assert(rw.action.items[rw.action.at] == "Cure")

-- edit rule 1: Tank, and a second condition; a value out of range shows the rule in red and stops the save
HR({ kind = "set", id = "target", index = 3 })
HR({ kind = "set", id = "cond2", index = 4 })
HR({ kind = "change", id = "arg2", text = "protect" })
assert(w.rules.rows[1][3] == "Tank" and w.rules.rows[1][4] == "HP below 50% and Missing effect Protect")
assert(w.editing.text:find("not saved", 1, true))
HR({ kind = "change", id = "arg", text = "250" })
local n = #sent
H({ kind = "press", id = "save", index = 1 })
assert(#sent == n and w.status.text:find("0%-100"), w.status.text)
HR({ kind = "change", id = "arg", text = "45" })

-- change rule 2 to a spell: the first offered is picked; the spells learned later are listed, marked
H({ kind = "select", id = "rules", index = 2 })
assert(open.gambits_rule and rw.which.text == "Rule 2 of Fayd heals")
HR({ kind = "set", id = "kind", index = 2 })
assert(rw.action.items[1] == "Cure (Lv 1)" and rw.action.items[3] == "Cure II (Lv 11, not learned yet)", rw.action.items[3])

-- save: parts under one tag, the server's ok
H({ kind = "press", id = "save", index = 1 })
local parts = {}
for i = n + 1, #sent do parts[#parts + 1] = sent[i] end
assert(#parts >= 1 and parts[1].text:match("^!gsq x %d+ 1 %d+ 4,3,WHM,2,Fayd_heals;1,TANK,HPP_LT,45,NOT_STATUS,PROTECT"))
for _, p in ipairs(parts) do assert(p.tag == parts[1].tag) end
ev.xifo_link_reply(parts[1].tag, { "ack|1|2", "ok|4|4|w4" }, true)
assert(w.status.text == "Saved as Rule Set 4." and not w.editing.text:find("not saved", 1, true))
-- the set list again, and the roster (an edited set in use shows as waiting for the next call)
assert(sent[#sent - 1].text:match("^!gsq list %d+ WHM$") and last():match("^!gsq mates %d+$"), last())

-- assign another set to the picked mate
H({ kind = "select", id = "sets", index = 2 })
H({ kind = "press", id = "use", index = 1 })
assert(last() == "!squad ruleset 1 1")
H({ kind = "press", id = "use", index = 2 })
assert(last() == "!squad ruleset 1 off")

-- a set that is not yours saves as a new one
H({ kind = "press", id = "setbtn", index = 1 })
answer({ "hdr|1|WHM|m|1|whm1|Default WHM", "rule|1|1/1|1,PARTY,HPP_LT,50,,,f,CURE,", "end|1" })
assert(w.editing.text:find("Save makes a copy", 1, true))
H({ kind = "press", id = "save", index = 1 })
assert(last():match("^!gsq x %d+ 1 1 0,%-,WHM,1,Default_WHM;"), last())

-- switching mates with changes asks first
H({ kind = "change", id = "name", text = "Mine" })
H({ kind = "set", id = "mate", index = 2 })
assert(#prompts == 1)
prompts[1].f(1)
assert(last():match("^!gsq catm %d+ 2 jm$"))

-- include / exclude roles: only for a role target (the new mate's catalog answered first)
local w2 = windows.gambits
ev.xifo_link_reply(sent[#sent].tag, cat, true)
H({ kind = "select", id = "sets", index = 1 })
H({ kind = "press", id = "setbtn", index = 1 })
answer({ "hdr|4|WHM|o|4|w4|Fayd heals", "rule|1|1/1|1,MELEE-TANK,HPP_LT,50,,,f,CURE,", "end|1" })
H({ kind = "select", id = "rules", index = 1 })
assert(rw.rf.items[1] == "Tank -" and rw.rf.items[2] == "Mage" and rw.rf.items[3] == "Ranged", rw.rf.items[1])
HR({ kind = "press", id = "rf", index = 2 })
assert(w2.rules.rows[1][3] == "Melee + Mage, not Tank", w2.rules.rows[1][3])
HR({ kind = "press", id = "rf", index = 1 })
assert(w2.rules.rows[1][3] == "Melee + Mage", "Tank: left out -> not set")
HR({ kind = "press", id = "rf", index = 1 })
HR({ kind = "press", id = "rf", index = 1 })
assert(w2.rules.rows[1][3] == "Melee + Mage, not Tank", "not set -> counts too -> left out")
HR({ kind = "set", id = "target", index = 2 }) -- (Melee is not in this catalog: listed first, marked)
assert(#rw.rf.items == 1 and w2.rules.rows[1][3] == "Self", tostring(w2.rules.rows[1][3]))

-- weapon skills of the picked mate (Salia, slot 2, after the mate switch above)
H({ kind = "set", id = "tabs", index = 2 })
assert(last():match("^!gsq wsg %d+ 2$"))
answer({ "ws|2|WAR|c|1|1|1500|33|0|-|1|0|1|1", "end|1" })
assert(w2.ws_ctl.at == 2 and w2.ws_mode.at == 1 and w2.ws_tp.at == 2)
assert(w2.ws_info.text:find("cannot use the chosen", 1, true))
assert(last():match("^!gsq wsl %d+ 2$"))
answer({ "wsl|16:Sturmwind,17:Raging_Axe", "end|1|2" })
assert(w2.ws_ws.items[2] == "Sturmwind" and w2.ws_ws.items[4] == "Weapon skill 33 (not usable now)", w2.ws_ws.items[4])
H({ kind = "set", id = "ws_ws", index = 3 })
H({ kind = "set", id = "ws_tp", index = 4 })
H({ kind = "press", id = "ws_btn", index = 1 })
assert(last():match("^!gsq wss %d+ 2 1 1 3000 17$"), last())

-- party roles
H({ kind = "set", id = "tabs", index = 3 })
assert(last():match("^!gsq prl %d+$"))
answer({ "pr|0|Nymh|RDM|5|0|C|1|0", "pr|1|Fayd|WHM|30|1|C|1|1", "pr|2|Salia|WAR|28|0|M|0|-", "pm|Bea|PLD|p|TM",
  "end|4" })
assert(w2.pr_list.rows[1][1] == "Nymh (you)" and w2.pr_list.rows[1][3] == "Auto (Mage)")
assert(w2.pr_list.rows[2][3] == "Tank" and w2.pr_list.rows[3][4] == "not called" and w2.pr_list.rows[4][4] == "Tank, Melee")
H({ kind = "select", id = "pr_list", index = 3 })
H({ kind = "set", id = "pr_role", index = 2 })
assert(last():match("^!gsq prs %d+ 2 1$"))
H({ kind = "select", id = "pr_list", index = 4 })
local n2 = #sent
H({ kind = "set", id = "pr_role", index = 3 })
assert(#sent == n2, "another player's role is not ours to set")

-- share codes: the open set out to the clipboard, a code in the box opened as a new set
local copied
client = { copy = function(t) copied = t end, paste = function(f) f("") end }
H({ kind = "set", id = "tabs", index = 1 })
H({ kind = "select", id = "sets", index = 1 })
local asked = #prompts
H({ kind = "press", id = "setbtn", index = 1 })
assert(#prompts == asked + 1, "the edited set is not thrown away without asking")
prompts[#prompts].f(1)
answer({ "hdr|4|WHM|o|4|w4|Fayd heals", "rule|1|1/1|1,PARTY,HPP_LT,50,,,f,CURE,", "end|1" })
H({ kind = "press", id = "share", index = 1 })
assert(copied and copied:sub(1, 3) == "GS1" and w2.code.text == copied)
local S = require("src.share")
assert(S.decode(copied).rules[1].name == "CURE")
w2.code.text = "  " .. copied .. "  "
H({ kind = "press", id = "share", index = 2 })
assert(w2.editing.text:find("New Rule Set", 1, true) and w2.name.text == "Fayd heals", w2.editing.text)
assert(w2.status.text:find("Imported Fayd heals", 1, true), w2.status.text)
-- a code that does not read changes nothing: no question, the unsaved import stays unsaved
w2.code.text = "GS1nope"
H({ kind = "press", id = "share", index = 2 })
assert(#prompts == asked + 1 and w2.status.text:find("Not imported", 1, true))
assert(w2.editing.text:find("not saved", 1, true))
-- a good code over unsaved work asks first; the imported rules lose their old ids
local S2 = require("src.share")
local code = assert(S2.encode({ job = "WHM", name = "Ids", rules = { { on = true, target = "SELF", cond = "ALWAYS",
  arg = "", kind = "r", id = "w9" } } }))
w2.code.text = code
H({ kind = "press", id = "share", index = 2 })
assert(#prompts == asked + 2)
prompts[#prompts].f(1)
assert(w2.name.text == "Ids")
H({ kind = "press", id = "share", index = 1 })
assert(S2.decode(copied).rules[1].id == nil)
-- the clipboard comes first (a long code is cut in the box)
client.paste = function(f) f(code) end
w2.code.text = "GS1cut"
H({ kind = "press", id = "share", index = 2 })
prompts[#prompts].f(1)
assert(w2.status.text:find("Imported Ids", 1, true), w2.status.text)
-- a job no mate has: kept, and said
local brd = assert(S2.encode({ job = "BRD", name = "Songs", rules = { { on = true, target = "SELF", cond = "ALWAYS",
  arg = "", kind = "r" } } }))
client.paste = function(f) f(brd) end
H({ kind = "press", id = "share", index = 2 })
prompts[#prompts].f(1)
assert(w2.status.text:find("No squad mate is a BRD", 1, true), w2.status.text)

-- a cut code on the clipboard does not hide a good one in the box
client.paste = function(f) f(code:sub(1, -4)) end
w2.code.text = brd
H({ kind = "press", id = "share", index = 2 })
prompts[#prompts].f(1)
assert(w2.status.text:find("Imported Songs", 1, true), w2.status.text)
-- neither reads: the box's reason is given
w2.code.text = "GS1zz"
H({ kind = "press", id = "share", index = 2 })
assert(w2.status.text:find("Not imported", 1, true))

-- a save's answer leaves the edits made while it was being sent unsaved
H({ kind = "set", id = "tabs", index = 1 })
H({ kind = "press", id = "save", index = 1 })
local save_tag = sent[#sent].tag
H({ kind = "change", id = "name", text = "Later" })
ev.xifo_link_reply(save_tag, { "ok|4|5|w4" }, true)
assert(w2.editing.text:find("not saved", 1, true) and w2.status.text:find("not saved yet", 1, true), w2.status.text)

-- "Keep editing" leaves the mate dropdown on the mate being edited
local cur = w2.mate.at
local other = cur == 1 and 2 or 1
w2.mate.at = other -- (the dropdown shows the pick before its event comes)
H({ kind = "set", id = "mate", index = other })
prompts[#prompts].f(2)
assert(w2.mate.at == cur)

-- after a mate switch, the last mate's weapon skill setting cannot be saved for the new one (Fayd is slot 1, Salia 2)
H({ kind = "set", id = "mate", index = other })
prompts[#prompts].f(1)
assert(w2.mate.at == other)
H({ kind = "set", id = "tabs", index = 2 })
assert(last():match("^!gsq wsg %d+ " .. other .. "$"), last())
H({ kind = "press", id = "ws_btn", index = 1 })
assert(not last():match("^!gsq wss"), "saved the other mate's setting")

-- an Open whose reply was cut short opens nothing
H({ kind = "set", id = "tabs", index = 1 })
H({ kind = "select", id = "sets", index = 1 })
H({ kind = "press", id = "setbtn", index = 1 })
answer({ "hdr|4|WHM|o|5|w4|Fayd heals", "rule|1|1/1|1,PARTY,HPP_LT,50,,,f,CURE," }, false)
assert(w2.editing.text == "No Rule Set open.", w2.editing.text)

-- opened with no squad mates, then reopened once there is one: its sets and choices load
sent, windows, open = {}, {}, {}
load(TEST.source("init.lua"), "@init.lua")()
cmd.gambits()
answer({ "hello|9|32|120|64" })
answer({ "end|0" })
cmd.gambits()
cmd.gambits()
assert(last():match("^!gsq mates %d+$"))
answer({ "mate|1|40|Fayd|WHM|30|-|0|0|-", "end|1" })
assert(sent[#sent - 1].text:match("^!gsq list %d+ WHM$") and last():match("^!gsq catm %d+ 1 jm$"), last())

-- a roster refresh that loses the picked mate on the Weapon skills page asks for the new pick's setting
H({ kind = "set", id = "tabs", index = 2 })
answer({ "ws|1|WHM|d|0|3|1000|0|0|-|1|1|1|1", "end|1" })
H({ kind = "press", id = "use", index = 3 }) -- Apply now; then the refresh, here by reopening
cmd.gambits()
cmd.gambits()
answer({ "mate|2|41|Salia|WAR|28|-|0|0|-", "end|1" })
assert(last():match("^!gsq wsg %d+ 2$"), last())

-- with no squad mates, an imported set still saves (under its own job)
sent, windows, open = {}, {}, {}
load(TEST.source("init.lua"), "@init.lua")()
local w3 = windows.gambits
cmd.gambits()
answer({ "hello|9|32|120|64" })
answer({ "end|0" })
local S3 = require("src.share")
w3.code.text = assert(S3.encode({ job = "WHM", name = "Starter", rules = { { on = true, target = "SELF",
  cond = "ALWAYS", arg = "", kind = "r" } } }))
client = { copy = function() end, paste = function(f) f("") end }
H({ kind = "press", id = "share", index = 2 })
assert(w3.status.text:find("Imported Starter", 1, true), w3.status.text)
H({ kind = "press", id = "save", index = 1 })
assert(last():match("^!gsq x %d+ 1 1 0,%-,WHM,1,Starter;"), last())
answer({ "ok|7|1|w7" })
assert(w3.status.text:find("Saved as Rule Set 7", 1, true), w3.status.text)
H({ kind = "press", id = "save", index = 2 }) -- Check, with no mate to check it on
assert(w3.status.text:find("squad mate", 1, true), w3.status.text)
-- a long code cut in the box, with no clipboard permission: the status says how to read it whole
client = { copy = function() end, paste = function() error("needs the clipboard permission") end }
w3.code.text = "GS1" .. string.rep("A", 1021) -- as the box cuts it: 1024 characters
H({ kind = "press", id = "share", index = 2 })
assert(w3.status.text:find("/lua grant xifo-gambits clipboard", 1, true), w3.status.text)

-- an imported set stays open while the first mates arrive: a mate of its job is picked, or the editor says whose
-- choices it shows
sent, windows, open = {}, {}, {}
load(TEST.source("init.lua"), "@init.lua")()
local w4 = windows.gambits
cmd.gambits()
answer({ "hello|9|32|120|64" })
answer({ "end|0" })
w4.code.text = assert(require("src.share").encode({ job = "WHM", name = "Heals", rules = { { on = true,
  target = "SELF", cond = "ALWAYS", arg = "", kind = "r" } } }))
client = { copy = function() end, paste = function(f) f("") end }
H({ kind = "press", id = "share", index = 2 })
cmd.gambits()
cmd.gambits()
answer({ "mate|1|41|Salia|WAR|28|-|0|0|-", "mate|2|40|Fayd|WHM|30|-|0|0|-", "end|2" })
assert(w4.mate.at == 2 and w4.editing.text:find("for WHM", 1, true) and not w4.editing.text:find("choices from", 1, true),
  w4.editing.text)
-- the picked mate's catalog fills the open rule's dropdowns
for i = #sent, 1, -1 do
  if sent[i].text:match("^!gsq catm %d+ 2 jm$") then
    ev.xifo_link_reply(sent[i].tag, cat, true)
    break
  end
end
H({ kind = "select", id = "rules", index = 1 })
local rw4 = windows.gambits_rule
assert(rw4.target.items[rw4.target.at] == "Self" and #rw4.target.items == 3, rw4.target.items[rw4.target.at])
cmd.gambits()
cmd.gambits()
answer({ "mate|1|41|Salia|WAR|28|-|0|0|-", "end|1" })
assert(w4.editing.text:find("choices from Salia, a WAR", 1, true), w4.editing.text)
-- not while the kept mate's weapon skill setting has unsaved changes
H({ kind = "set", id = "tabs", index = 2 })
answer({ "ws|1|WAR|d|0|3|1000|0|0|-|0|1|1|1", "end|1" })
H({ kind = "set", id = "ws_tp", index = 4 })
cmd.gambits()
cmd.gambits()
answer({ "mate|1|41|Salia|WAR|28|-|0|0|-", "mate|3|42|Bea|WHM|20|-|0|0|-", "end|2" })
assert(w4.mate.at == 1, "the edited weapon skill setting's mate was switched away")
H({ kind = "press", id = "ws_btn", index = 1 })
assert(last():match("^!gsq wss %d+ 1 0 3 3000 0$"), last())
answer({ "ws|1|WAR|c|0|3|3000|0|0|-|0|1|1|1", "end|1" })
-- the same mate changed jobs (Squad) while an edit was unsaved: it counts as a new pick, its old job's setting gone
H({ kind = "set", id = "ws_tp", index = 2 })
cmd.gambits()
cmd.gambits()
answer({ "mate|1|41|Salia|RDM|28|-|0|0|-", "mate|3|42|Bea|WHM|20|-|0|0|-", "end|2" })
assert(w4.mate.at == 2 and last():match("^!gsq wsg %d+ 3$"), last())
H({ kind = "press", id = "ws_btn", index = 1 })
assert(not last():match("^!gsq wss"), "the old job's setting was saved")
H({ kind = "set", id = "tabs", index = 1 })
-- a mate of the set's job arriving later is picked over the one of another job, the unsaved set kept
cmd.gambits()
cmd.gambits()
answer({ "mate|1|41|Salia|WAR|28|-|0|0|-", "mate|3|42|Bea|WHM|20|-|0|0|-", "end|2" })
assert(w4.mate.at == 2 and w4.name.text == "Heals" and not w4.editing.text:find("choices from", 1, true),
  w4.editing.text)

-- with no set open, a mate that changed jobs stays picked (its new job's setting loads)
H({ kind = "press", id = "save", index = 3 }) -- Undo changes: a new set is dropped
assert(w4.editing.text == "No Rule Set open.", w4.editing.text)
H({ kind = "set", id = "tabs", index = 2 })
cmd.gambits()
cmd.gambits()
answer({ "mate|1|41|Salia|RDM|28|-|0|0|-", "mate|3|42|Bea|BLM|20|-|0|0|-", "end|2" })
assert(w4.mate.at == 2 and last():match("^!gsq wsg %d+ 3$"), last())

-- changed jobs while the window stayed open (in XIFO Squad): the setting's read names the new job, the mate follows
-- it, and Save saves
answer({ "ws|3|SMN|d|0|3|1000|0|0|-|0|1|1|1", "end|1" })
assert(last():match("^!gsq mates %d+$"), "the roster is read again: " .. last())
answer({ "mate|1|41|Salia|RDM|28|-|0|0|-", "mate|3|42|Bea|SMN|22|WHM|0|0|11", "end|2" })
assert(w4.mate.at == 2 and w4.mate.items[2]:find("SMN 22/WHM 11", 1, true), w4.mate.items[2])
assert(last():match("^!gsq wsg %d+ 3$"), last())
answer({ "ws|3|SMN|d|0|3|1000|0|0|-|0|1|1|1", "end|1" })
H({ kind = "set", id = "ws_tp", index = 4 })
H({ kind = "press", id = "ws_btn", index = 1 })
assert(last():match("^!gsq wss %d+ 3 0 3 3000 0$"), last())

-- back to the job it had, after the roster agreed once: read again too (the guard cleared), and the mate stays
-- picked even with an open set another mate's job fits
answer({ "ws|3|SMN|c|0|3|3000|0|0|-|0|1|1|1", "end|1" })
w4.code.text = assert(require("src.share").encode({ job = "WAR", name = "Axe", rules = { { on = true,
  target = "SELF", cond = "ALWAYS", arg = "", kind = "r" } } }))
H({ kind = "press", id = "share", index = 2 })
assert(w4.mate.at == 2 and w4.status.text:find("No squad mate is a WAR", 1, true), w4.status.text)
H({ kind = "press", id = "ws_btn", index = 3 }) -- Refresh: the server now says BLM again
answer({ "ws|3|BLM|d|0|3|1000|0|0|-|0|1|1|1", "end|1" })
assert(last():match("^!gsq mates %d+$"), last())
answer({ "mate|1|41|Salia|WAR|28|-|0|0|-", "mate|3|42|Bea|BLM|20|-|0|0|-", "end|2" })
assert(w4.mate.at == 2 and last():match("^!gsq wsg %d+ 3$"), "the mate that changed jobs stays picked: " .. last())
-- the same mismatch again after the roster agreed in between: read again (the guard was cleared by the agreement)
answer({ "ws|3|BLM|d|0|3|1000|0|0|-|0|1|1|1", "end|1" }) -- agrees with the roster
cmd.gambits()
cmd.gambits()
answer({ "mate|1|41|Salia|RDM|28|-|0|0|-", "mate|3|42|Bea|SMN|20|-|0|0|-", "end|2" }) -- the roster says SMN now
assert(last():match("^!gsq wsg %d+ 3$"), last())
answer({ "ws|3|BLM|d|0|3|1000|0|0|-|0|1|1|1", "end|1" }) -- but the server's setting is BLM's again
assert(last():match("^!gsq mates %d+$"), "the repeated mismatch is read again: " .. last())

-- a version 10 server: capability words, each mate's live Rule Set state, Apply this mate
sent, windows, open = {}, {}, {}
load(TEST.source("init.lua"), "@init.lua")()
local w5 = windows.gambits
cmd.gambits()
answer({ "hello|10|32|120|64|live,reapply1" })
answer({ "mate|1|40|Fayd|WHM|30|BLM|4|1|15|pending|edited|4|0|0", "mate|2|41|Salia|WAR|28|-|0|0|-|off|-|0|0|0",
  "end|2" })
assert(w5.mate.items[1] == "Fayd  WHM 30/BLM 15  (out, *)", w5.mate.items[1])
assert(w5.use.items[4] == "Apply this mate", tostring(w5.use.items[4]))
local list5 = sent[#sent - 1]
ev.xifo_link_reply(list5.tag, { "set|4|WHM|2|o|3|w5|Fayd heals", "end|1" }, true)
assert(w5.live.text:find("Runs Rule Set 4; its set was edited", 1, true), w5.live.text)
timers, delays, queued = {}, {}, 1 -- the command still waits in XIFO Link
handlers.gambits("gambits", { kind = "press", id = "use", index = 4 })
assert(last() == "!squad reapply 1", last())
-- while the link still holds it, checked again every half second; once said, the 6 s recall wait; then the roster
assert(#timers == 1 and delays[1] == 0.5, tostring(delays[1]))
timers[1]()
assert(#timers == 2 and delays[2] == 0.5, "still queued")
queued = 0
timers[2]()
assert(#timers == 3 and delays[3] == 6, tostring(delays[3]))
assert(last() == "!squad reapply 1", "no roster read before the wait")
timers[3]()
assert(last():match("^!gsq mates %d+$"), last())
answer({ "mate|1|40|Fayd|WHM|30|BLM|4|1|15|set|-|4|0|0", "mate|2|41|Salia|WAR|28|-|0|0|-|off|-|0|0|0", "end|2" })
assert(w5.mate.items[1] == "Fayd  WHM 30/BLM 15  (out)" and w5.live.text == "Runs Rule Set 4.", w5.live.text)
-- an away mate: no Apply this mate
handlers.gambits("gambits", { kind = "set", id = "mate", index = 2 })
assert(w5.use.items[4] == nil)

-- deleting the set a mate runs: the roster is read again and the live line says what the mate runs now
handlers.gambits("gambits", { kind = "set", id = "mate", index = 1 })
for i = #sent, 1, -1 do
  if sent[i].text:match("^!gsq list %d+ WHM$") then
    ev.xifo_link_reply(sent[i].tag, { "set|4|WHM|2|o|3|w5|Fayd heals", "end|1" }, true)
    break
  end
end
handlers.gambits("gambits", { kind = "select", id = "sets", index = 1 })
handlers.gambits("gambits", { kind = "press", id = "setbtn", index = 4 })
prompts[#prompts].f(1)
assert(last():match("^!gsq del %d+ 4 3$"), last())
answer({ "ok|4|3|w5" })
assert(last():match("^!gsq mates %d+$"), "the roster after a delete: " .. last())
answer({ "mate|1|40|Fayd|WHM|30|BLM|0|1|15|fallback|-|0|1|0", "mate|2|41|Salia|WAR|28|-|0|0|-|off|-|0|0|0", "end|2" })
assert(w5.live.text == "Runs its job rules: the chosen set is gone.", w5.live.text)

-- a role set on the Party roles page: once the server takes it, the roles and the roster are read again
handlers.gambits("gambits", { kind = "set", id = "tabs", index = 3 })
answer({ "pr|0|Nymh|RDM|5|0|C|1|0", "pr|1|Fayd|WHM|30|0|C|1|0", "end|2" })
handlers.gambits("gambits", { kind = "select", id = "pr_list", index = 2 })
handlers.gambits("gambits", { kind = "set", id = "pr_role", index = 2 })
assert(last():match("^!gsq prs %d+ 1 1$"), last())
answer({ "pr|1|Fayd|WHM|30|1|C|1|1", "end|1" })
assert(last():match("^!gsq mates %d+$") and sent[#sent - 1].text:match("^!gsq prl %d+$"), last())
answer({ "mate|1|40|Fayd|WHM|30|BLM|0|1|15|pending|role|0|0|0", "mate|2|41|Salia|WAR|28|-|0|0|-|off|-|0|0|0", "end|2" })
assert(w5.live.text:find("its role changed", 1, true), w5.live.text)
