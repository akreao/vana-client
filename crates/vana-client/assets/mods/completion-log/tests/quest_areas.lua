-- Completion Log: the Quests tab lists only the areas the game's quest menu shows (game.quest_areas), and leaves out
-- the quest files' unused entries with Japanese working names unless we have taken one; on a server without Wings of
-- the Goddess, the later updates' quests in the old areas too.

C, said = {}, {}
settings = { get = function() return true end }
commands = { register = function(n, f) C[n] = f end }
chat = { add = function(t) said[#said + 1] = t end }
game = {
  player = function() return { main_job = 4 } end,
  jobs = function() return { levels = { [4] = 75 } } end,
  spells = function() return {} end,
  quest_areas = function() return { 0, 1, 2, 3, 4, 5, 6 } end,
  quests = function()
    return {
      [0] = {
        { number = 1, name = "A Squire's Test", current = false, completed = true },
        { number = 111, name = "SDクエスト111", current = false, completed = false },
        { number = 115, name = "SDクエスト115", current = true, completed = false },
      },
      [6] = { { number = 1, name = "Making Amends", current = true, completed = false } },
      [7] = { { number = 1, name = "Lost in Translocation", current = false, completed = false } },
    }
  end,
}
res = { job = function(id) return ({ [4] = "BLM" })[id] or "JOB" .. id end, spell = function() return nil end }
local on_window, window, update = ui.on, ui.window, ui.update
ui.on = function(id, f) H = f; on_window(id, f) end
-- What the mod drew: the window's elements by id, then each update's fields over them.
local drawn = {}
ui.window = function(id, spec)
  for _, e in ipairs(spec.elements) do if e.id then drawn[e.id] = e end end
  return window(id, spec)
end
ui.update = function(id, el, fields)
  for k, v in pairs(fields) do drawn[el][k] = v end
  return update(id, el, fields)
end
load(TEST.source("init.lua"), "@init.lua")()

C.left()
assert(ui.is_open("completion"), "//left opened no window")
H("completion", { kind = "set", id = "tabs", index = 2 })
local areas = table.concat(drawn.area.items, "|")
assert(areas == "San d'Oria|Bastok|Windurst|Jeuno|Other Areas|Outlands|Aht Urhgan", areas)
local function names()
  local out = {}
  for _, r in ipairs(drawn.quests.rows) do out[#out + 1] = r[1] end
  return table.concat(out, "|")
end
-- San d'Oria: the unused entry is left out, the one under way stays.
assert(names() == "A Squire's Test|SDクエスト115", names())
H("completion", { kind = "set", id = "area", index = 7 })
assert(names() == "Making Amends", names())

C.left("quests")
local text = table.concat(said, "\n")
assert(text:find("San d'Oria: 1 of 2 done, 1 under way.", 1, true), text)
assert(not text:find("Crystal War", 1, true), text)

-- A server that stops at ToAU (no Crystal War area): the later updates' quests in the old areas are left out unless
-- taken; with Wings of the Goddess they are listed.
local areas_on = { 0, 1, 2, 3, 4, 5, 6 }
game.quest_areas = function() return areas_on end
game.quests = function()
  return {
    [0] = {
      { number = 1, name = "A Squire's Test", current = false, completed = true },
      { number = 119, name = "Trust: San d'Oria", current = false, completed = false },
    },
    [4] = {
      { number = 34, name = "Monstrosity", current = false, completed = false },
      { number = 84, name = "VW Op. #026: Tavnazian Terrors", current = false, completed = false },
      { number = 110, name = "Records of Eminence", current = false, completed = true },
      { number = 111, name = "Unity Concord", current = false, completed = false },
    },
    [5] = { { number = 104, name = "Skyward Ho, Voidwatcher!", current = false, completed = false } },
  }
end
H("completion", { kind = "set", id = "area", index = 1 })
assert(names() == "A Squire's Test", names())
H("completion", { kind = "set", id = "area", index = 5 })
assert(names() == "Records of Eminence", names())
H("completion", { kind = "set", id = "area", index = 6 })
assert(names() == "", names())
areas_on = { 0, 1, 2, 3, 4, 5, 6, 7 }
H("completion", { kind = "set", id = "area", index = 1 })
assert(names() == "A Squire's Test|Trust: San d'Oria", names())
