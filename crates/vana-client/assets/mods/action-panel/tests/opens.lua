-- Action Panel: //actions shows its window (a HUD one: the game keeps its keys), listing the castable spells by
-- school and the abilities by tab with the costs the game's lists show; a row runs its macro line; the list
-- follows a pet called while it is open.

C, E, ran, said, tick = {}, {}, {}, {}, nil
on, pet = true, nil
settings = { get = function() return on end }
commands = { register = function(n, f) C[n] = f end, run = function(l) ran[#ran + 1] = l end }
events = { on = function(n, f) E[n] = f end }
chat = { add = function(t) said[#said + 1] = t end }
timer = { every = function(_, f) tick = f end }
local spells = {
  [1] = { name = "Cure", kind = 1, levels = { [3] = 1 }, mp = 8, targets = 0x1F },
  [100] = { name = "Protect", kind = 1, levels = { [3] = 7 }, mp = 9, targets = 1 },
  [144] = { name = "Fire", kind = 2, levels = { [4] = 13 }, mp = 7, targets = 0x20 },
  [145] = { name = "Fire II", kind = 2, levels = { [4] = 38 }, mp = 26, targets = 0x20 },
  [146] = { name = nil, kind = 2, levels = { [4] = 1 }, mp = 1, targets = 0x20 },
}
local function ab(name, kind, t) t.name, t.kind = name, kind; t.tp = t.tp or 255; t.cost = t.cost or 0
  t.targets = t.targets or 1; t.recast_id = t.recast_id or 900; t.sub = t.sub or 0; return t end
local abilities = {
  [0x20] = ab("Fast Blade", 3, { targets = 0x20 }),
  [0x210] = ab("Divine Seal", 1, { recast_id = 26 }),
  [0x220] = ab("Curing Waltz", 11, { tp = 200, targets = 0x1F }),
  -- Not listed: a trait, and a row that only opens a sub-list (its rolls are listed).
  [0x230] = ab("Resist Sleep", 4, {}),
  [0x240] = ab("Phantom Roll", 1, { recast_id = 193, sub = 8 }),
  [0x250] = ab("Fighter's Roll", 8, { recast_id = 193 }),
  -- Pet commands, only with a pet: a blood pact costs MP, a Ready move charges.
  [0x400] = ab("Fight", 2, { targets = 0x20 }),
  [0x410] = ab("Foot Kick", 0x12, { cost = 1, targets = 0x20 }),
  [0x420] = ab("Searing Light", 6, { cost = 107, targets = 0x20 }),
}
game = {
  player = function() return { main_job = 3, main_level = 75, sub_job = 4, sub_level = 37 } end,
  info = function() return { pet_index = pet } end,
  job_data = function() return {} end,
  spells = function() return { 1, 100, 144, 145, 146 } end,
  abilities = function() return { 0x20, 0x210, 0x220, 0x230, 0x240, 0x250, 0x400, 0x410, 0x420 } end,
  recasts = function() return { spells = { [1] = 3.2 }, abilities = { [26] = 300 }, totals = {} } end,
}
res = { spell = function(id) return spells[id] end, ability = function(id) return abilities[id] end }
local on_window = ui.on
ui.on = function(id, f) H = f; on_window(id, f) end
load(TEST.source("init.lua"), "@init.lua")()
C.actions()
assert(ui.is_open("actions"), "//actions opened no window")
tick()
-- White magic (the first school): Cure and Protect by name; a pick runs its line.
H("actions", { kind = "select", id = "list", index = 1 })
H("actions", { kind = "select", id = "list", index = 2 })
-- Black magic: the nameless spell (names still loading) is left out.
H("actions", { kind = "set", id = "school", index = 2 })
H("actions", { kind = "select", id = "list", index = 1 })
-- Weapon skills.
H("actions", { kind = "set", id = "tabs", index = 3 })
H("actions", { kind = "select", id = "list", index = 1 })
assert(table.concat(ran, "|") == '/ma "Cure" <st>|/ma "Protect" <me>|/ma "Fire" <st>|/ws "Fast Blade" <st>',
  table.concat(ran, "|"))
C.actions()
assert(not ui.is_open("actions"), "a second //actions closes it")
on = false
C.actions()
assert(not ui.is_open("actions"), "off: //actions does nothing")
on = true
C.actions()
on = false
E.settings_changed("on", false)
assert(not ui.is_open("actions"), "switched off: the window closes")
on = true
C.actions()
H("actions", { kind = "press", id = "close", index = 1 })
assert(not ui.is_open("actions"), "Close closes it")
C.actions()
H("actions", { kind = "set", id = "tabs", index = 2 })

-- The window as drawn: a HUD one, open, without the keys; its list's rows.
local function rows()
  local w = TEST.window("actions")
  assert(w.open and w.hud and not w.focus, "a HUD window, open, without the keys")
  local out = {}
  for _, r in ipairs(w.rows) do out[#out + 1] = table.concat(r, "|") end
  return table.concat(out, " / ")
end
-- The Abilities page: the waltz's TP, Divine Seal's recast, Fighter's Roll.
local got = rows()
assert(got == "Curing Waltz|200 TP| / Divine Seal||5:00 / Fighter's Roll||", got)
-- The Pet page: nothing without a pet; the commands once one is called.
H("actions", { kind = "set", id = "tabs", index = 4 })
assert(rows() == "", rows())
pet = 1234; tick()
got = rows()
assert(got == "Fight|| / Foot Kick|1 charge| / Searing Light|107 MP|", got)
