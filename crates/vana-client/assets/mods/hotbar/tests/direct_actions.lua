-- Hotbars: a slot of one action goes straight to the game's action (actions.cast / ability / use_item / equip, the
-- system commands that are actions), lines run as typed; a pet command and a job ability of one name stay apart;
-- trusts are their own type; a group takes more than 12; a text area's lines; an icon picked for a macro is saved
-- and shown; the one tooltip of a macro.

local spells = {
  [1] = { name = "Cure", kind = 1, mp = 8, cast_time = 2, recast = 5, targets = 0x1f, description = "Restores HP." },
  [896] = { name = "Shantotto", kind = 8, mp = 0, targets = 1 },
}
local abilities = {
  [547] = { name = "Provoke", kind = 1, targets = 0x20, recast = 30 },
  [600] = { name = "Fight", kind = 1, targets = 0x20 },
  [1030] = { name = "Fight", kind = 2, targets = 0x20 },
}
local items = { [4112] = { name = "Potion", kind = "usable" }, [13180] = { name = "Ring", slots = 0x6000 } }
res = setmetatable({ spell = function(id) return spells[id] end, ability = function(id) return abilities[id] end,
  item = function(id) return items[id] end, job = function(id) return ({ [1] = "WAR" })[id] end },
  { __index = function() return function() end end })
game = setmetatable({ player = function() return { main_job = 1, sub_job = 0, main_level = 99 } end,
  spells = function() return { 1, 896 } end,
  find_items = function(name)
    for id, r in pairs(items) do if r.name:lower() == name:lower() then return { { id = id } } end end
    return {}
  end }, { __index = function() return function() return {} end end })
chat = { add = function() end }

local DID = {}
local function did(s) DID[#DID + 1] = s end
actions.cast = function(id, t) did(("cast %d %s"):format(id, t or "-")) end
actions.ability = function(id, t) did(("ability %d %s"):format(id, t or "-")) end
actions.use_item = function(id, t) did(("item %d %s"):format(id, t or "-")) end
actions.equip = function(slot, id) did(("equip %s %d"):format(slot, id)) end
actions.attack = function(t) did("attack " .. (t or "-")) end
actions.heal = function() did("heal") end
commands.run = function(line) did("run " .. line) end

local data = require("src.data")
data.init({ read = function() return nil end, write = function() end, decode = json.decode, encode = json.encode })
local runner = require("src.runner")
local function slot(entry)
  local id = data.save_entry("WAR", nil, entry)
  DID = {}
  runner.run(data.macro_of("WAR", id))
  return table.concat(DID, "; ")
end
local function one(kind, name, target) return { name = name, action = { kind = kind, name = name, target = target } } end
assert(slot(one("ma", "Cure", "stpc")) == "cast 1 <stpc>")
assert(slot(one("trust", "Shantotto", "me")) == "cast 896 <me>")
assert(slot(one("ja", "Provoke", "t")) == "ability 547 <t>")
-- A job ability and a pet command of one name: each its own.
assert(slot(one("ja", "Fight", "t")) == "ability 600 <t>")
assert(slot(one("pet", "Fight", "t")) == "ability 1030 <t>")
assert(slot(one("item", "Potion", "me")) == "item 4112 <me>")
assert(slot(one("equip", "Ring", "ring1")) == "equip ring1 13180")
assert(slot({ name = "Attack", action = { kind = "cmd", name = "/attack", target = "t" } }) == "attack <t>")
assert(slot({ name = "Heal", action = { kind = "cmd", name = "/heal", target = "" } }) == "heal")
-- No action of the client's for it: its line.
assert(slot({ name = "Sit", action = { kind = "cmd", name = "/sit", target = "" } }) == "run /sit")
-- A macro's lines run as typed; so does an older entry's action with lines after it.
local wait = sleep
sleep = function() end
assert(slot({ name = "Talk", lines = { "/p one", "/p two" } }) == "run /p one; run /p two")
assert(slot({ name = "Old", action = { kind = "ma", name = "Cure", target = "t" }, lines = { "/p cured" } })
  == 'run /ma "Cure" <t>; run /p cured')
sleep = wait

-- Trusts are their own type: not among the spells.
local builder = require("src.builder")
local function found(kind)
  local out = {}
  for _, e in ipairs(builder.find(kind, "")) do out[#out + 1] = e.name end
  return table.concat(out, ",")
end
assert(found("ma") == "Cure" and found("trust") == "Shantotto", found("ma") .. " / " .. found("trust"))
assert(builder.TYPE_LABELS[builder.type_index("trust")] == "Trust")

-- A group holds more than 12.
local members = {}
for k = 1, 20 do members[k] = { name = "M" .. k, lines = { "/p " .. k } } end
local gid = data.save_entry("WAR", nil, { name = "Many", group = members })
assert(#data.get_entry("WAR", gid).group == 20, "a group of 20 lost members")

-- A text area's lines (one a line), and the one-line field's (| between them).
assert(table.concat(data.split_lines("/p a | b\n /p c \n\n", "\n"), ",") == "/p a | b,/p c")
assert(table.concat(data.split_lines("/p a | /p b"), ",") == "/p a,/p b")
assert(data.join_lines({ "/p a", "/p b" }, "\n") == "/p a\n/p b")

-- An icon picked for a macro is saved and shown (its recast still its action's).
local actions_ = require("src.actions")
local iid = data.save_entry("WAR", nil, { name = "Cure", icon = { kind = "item", id = 4112 },
  action = { kind = "ma", name = "Cure", target = "t" } })
local m = data.macro_of("WAR", iid)
assert(data.get_entry("WAR", iid).icon.id == 4112, "the icon was not saved")
local look, of = actions_.look(m), actions_.of(m)
assert(look.kind == "item" and look.id == 4112 and of.kind == "spell" and of.id == 1, "look / of")
assert(not data.get_entry("WAR", data.save_entry("WAR", nil, { name = "Bad", icon = { kind = "nope", id = 1 },
  lines = { "/p x" } })).icon, "a bad icon was kept")

-- The one tooltip: the name beside the icon shown, the action's numbers, its description.
local tooltip = require("src.tooltip")
local t = tooltip.macro(m)
assert(t.title == "Cure" and t.icon.kind == "item" and t.text == "MP 8    Cast 2s    Recast 5s\n\nRestores HP.", t.text)
t = tooltip.macro(data.macro_of("WAR", data.save_entry("WAR", nil, { name = "Talk", lines = { "/p one", "/p two" } })))
assert(t.text == "/p one\n/p two", t.text)
t = tooltip.action({ kind = "ability", id = 547 }, "Provoke")
assert(t.text == "Recast 30s" and t.icon.id == 547, t.text)
