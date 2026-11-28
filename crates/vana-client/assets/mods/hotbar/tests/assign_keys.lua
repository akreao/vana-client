-- Hotbars' Assign Keys: the bars' right-click menu anywhere (a slot's line only on a slot); Assign Keys takes the
-- bars' keys and clicks away, a hovered or clicked slot takes the next key pressed, a clash (another slot, the game)
-- asks first; Escape ends it and binds the keys again.

timer.after = function(_, f) f() end
timer.every = function() end
SAID = {}
chat = { add = function(t) SAID[#SAID + 1] = t end }
storage = { character = { hb_shown = 2 }, account = {}, save = function() end }

local OVER = { on = true, visible = true, display = "Window", bars = 2, slots = 12, size = 40, gap = 4,
  show_keys = false, pad_size = 36, pad_autohide = false }
local real_get = settings.get
settings.get = function(k) if OVER[k] ~= nil then return OVER[k] end return real_get(k) end

res = setmetatable({
  spell = function(id) if id == 1 then return { name = "Cure", kind = 1 } end end,
  job = function(id) return ({ [5] = "RDM" })[id] end,
}, { __index = function() return function() end end })
game = setmetatable({
  player = function() return { main_job = 5, sub_job = 0 } end,
  recasts = function() return { spells = {}, abilities = {}, totals = { spells = {}, abilities = {} } } end,
}, { __index = function() return function() return {} end end })

local OPEN = {}
WINDOWS = {}
ui.window = function(id, spec) WINDOWS[id] = spec end
local HANDLERS = {}
ui.on = function(id, f) HANDLERS[id] = f end
ui.update = function() end
ui.close = function(id) OPEN[id] = nil end
ui.open = function(id) OPEN[id] = true end
ui.is_open = function(id) return OPEN[id] end

local data = require("src.data")
data.init({ read = function() return nil end, write = function() end, decode = json.decode, encode = json.encode })
local lib_id = data.save_entry("RDM", nil, { name = "Cure", action = { kind = "ma", name = "Cure", target = "t" },
  lines = { '/ma "Cure" <t>' } })
local grp_id = data.save_entry("RDM", nil, { name = "Sets", group = {
  { name = "A", lines = { '/echo a' } },
  { name = "B", lines = { '/echo b' } },
} })
data.assign("RDM", nil, 1, 1, lib_id)
data.assign("RDM", nil, 1, 3, grp_id)


-- The keys bound, the key filter, the prompt asked (answered by ANSWER).
local BOUND = {}
binds.set = function(c) BOUND[c] = true end
binds.unset = function(c) BOUND[c] = nil end
local KEYF
input.on_key = function(f) KEYF = f; return 7 end
input.off = function(id) if id == 7 then KEYF = nil end end
local ASKED, ANSWER
ui.prompt = function(text, _, h) ASKED = text; h(ANSWER) end
binds.users = function(c) return c == "f1" and { { kind = "game", name = "targeting yourself" } } or {} end

local bar = require("src.bar")
local MANAGER = 0
bar.hooks = { changed = function() end, open_manager = function() MANAGER = MANAGER + 1 end }
bar.rebuild()
bar.rebind()
assert(BOUND["1"] and BOUND["ctrl+1"], "the bars' keys are bound")
local H = HANDLERS.bars
local function menu_rows()
  local m = WINDOWS.slotmenu
  for _, e in ipairs(m.elements) do
    if e.id == "acts" then
      local out = {}
      for k, r in ipairs(e.rows) do out[k] = r[1] end
      return table.concat(out, ", "), m
    end
  end
end
local function has_head(m)
  for _, e in ipairs(m.elements) do if e.id == "head" then return true end end
  return false
end
local function pick(label)
  for k, r in ipairs(WINDOWS.slotmenu.elements[#WINDOWS.slotmenu.elements].rows) do
    if r[1] == label then HANDLERS.slotmenu(nil, { kind = "select", id = "acts", index = k }); return end
  end
  error("no " .. label .. " in the menu")
end

-- Off the slots (the window's heading: its context event) the menu has no slot line.
H(nil, { kind = "context", x = 5, y = 5 })
local rows, m = menu_rows()
assert(rows == "Open the manager, Assign Keys" and not has_head(m), rows)
-- On a slot: the slot's line, and its Clear.
H(nil, { kind = "click", id = "c", x = 20, y = 20, button = "right" })
rows, m = menu_rows()
assert(has_head(m) and rows:find("Assign Keys", 1, true), rows)
-- Off the slots on the canvas too.
H(nil, { kind = "click", id = "c", x = 1, y = 1, button = "right" })
rows, m = menu_rows()
assert(not has_head(m), "a right-click off the slots showed a slot")

-- Assign Keys: the keys come off, the heading says so.
pick("Assign Keys")
assert(bar.assigning() and KEYF and not BOUND["1"], "assigning: the bars' keys are still bound")
assert(WINDOWS.bars.heading:find("Assigning keys", 1, true), WINDOWS.bars.heading)
-- A click chooses a slot and runs nothing; the next key is its.
local ran = 0
local runner = require("src.runner")
local run = runner.run_checked
runner.run_checked = function() ran = ran + 1 end
bar.fire(1, 2)
assert(ran == 0, "a click ran the slot while assigning")
assert(KEYF({ key = "q", down = true, ctrl = true }) == true, "the key was not taken")
local keys = require("src.keys")
assert(keys.combo(1, 2) == "ctrl+q", keys.combo(1, 2))
-- Modifiers alone wait; a clash asks: Cancel keeps the old key, Use It takes it.
assert(KEYF({ key = "lctrl", down = true, ctrl = true }) == false)
ANSWER = 2
KEYF({ key = "f1", down = true })
assert(ASKED and ASKED:find("F1 is also targeting yourself", 1, true), tostring(ASKED))
assert(keys.combo(1, 2) == "ctrl+q", "Cancel changed the key")
ANSWER = 1
KEYF({ key = "f1", down = true })
assert(keys.combo(1, 2) == "f1")
-- Another slot's key asks too.
ASKED, ANSWER = nil, 2
KEYF({ key = "1", down = true })
assert(ASKED and ASKED:find("bar 1, slot 1", 1, true), tostring(ASKED))
-- Use It moves the key: the other slot gives it up (a shared key binds the first slot only).
ANSWER = 1
KEYF({ key = "1", down = true })
assert(keys.combo(1, 2) == "1" and keys.combo(1, 1) == "", ("1:1 %q, 1:2 %q"):format(keys.combo(1, 1), keys.combo(1, 2)))
-- Use the bar's key on slot 1 takes "1" back the same way (asks, then slot 2 gives it up).
H(nil, { kind = "click", id = "c", x = 20, y = 64, button = "right" })
assert(WINDOWS.slotmenu.title == "Bar 1, Slot 1", WINDOWS.slotmenu.title)
ASKED, ANSWER = nil, 1
pick("Use the bar's key")
assert(ASKED and ASKED:find("bar 1, slot 2", 1, true), tostring(ASKED))
assert(keys.get_slot(1, 1) == nil and keys.combo(1, 1) == "1" and keys.combo(1, 2) == "",
  ("1:1 %q, 1:2 %q"):format(keys.combo(1, 1), keys.combo(1, 2)))
-- Escape ends it: the keys are bound again, clicks run.
assert(KEYF({ key = "escape", down = true }) == true)
assert(not bar.assigning() and KEYF == nil and BOUND["1"], "Escape did not end assigning")
bar.fire(1, 1)
assert(ran == 1, "a click after assigning did not run the slot")
assert(#require("src.keys").conflicts() == 0, "a moved key left a conflict")
runner.run_checked = run
