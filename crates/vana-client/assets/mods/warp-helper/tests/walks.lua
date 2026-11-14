-- Warp Helper: the Home Point and Survival Guide walks pick the rows the menus showed live, through a second page.

local C, said, picks, menus, at = {}, {}, {}, {}, 0
settings = { get = function(k) return k == "fare" and "Gil" or true end }
commands = { register = function(n, f) C[n] = f end }
chat = { add = function(t) said[#said + 1] = t end }
events = { on = function() end }
sleep = function() end
game = {
  event = function()
    if at == 0 or at > #menus then return nil end
    return { menu = { question = menus[at][1], options = menus[at][2], cursor = 1 } }
  end,
  entities = function() return { { id = 7, name = "Home Point #1", distance = 3, pos = { x = 0, z = 0 } },
    { id = 8, name = "Survival Guide", distance = 40, pos = { x = 0, z = 0 } } } end,
}
actions = {
  talk = function() at = 1 end,
  choose = function(i) picks[#picks + 1] = i; at = at + 1 end,
}
load(TEST.source("init.lua"), "@init.lua")()
menus = {
  { "What will you do?", { "Travel to another home point.", "Set this as your home point.", "Other settings.", "On second thought, never mind." } },
  { "Teleport to which region?", { "On second thought, none.", "Select from favorites.", "Current region.", "San d'Oria.", "Bastok.", "Windurst.", "Jeuno.", "Next page." } },
  { "Teleport where?", { "Never mind.", "Ru'Lude Gardens.", "Upper Jeuno.", "Lower Jeuno.", "Port Jeuno." } },
  { "Teleport where?(Lower Jeuno)", { "Nowhere.", "Home Point #1 (E).", "Home Point #2 (M)." } },
  { "Teleport? (Current gil: 97531)", { "Add to favorites.", "Yes, please.", "No, thank you." } },
}
C.warp("hp", "lower", "jeuno", "2")
assert(table.concat(picks, " ") == "1 7 4 3 2", table.concat(picks, " "))
C.warp("jeuno")
assert(said[#said] == "Warp Helper: which one: Upper Jeuno, Lower Jeuno, Port Jeuno?", said[#said])
picks, at = {}, 0
game.entities = function() return { { id = 8, name = "Survival Guide", distance = 2, pos = { x = 0, z = 0 } } } end
menus = {
  { "What chapter will you read?", { "None.", "\u{201C}Teleportation Assistance.\u{201D}", "\u{201C}Other mysteries.\u{201D}" } },
  { "Travel to which region?", { "On second thought, none.", "Select from amongst favorites.", "Bastok.", "Ronfaure.", "Next page." } },
  { "Travel to which region?", { "On second thought, none.", "Select from amongst favorites.", "Movalpolos.", "Mamool Ja Savage.", "Previous page." } },
  { "Where will you travel?", { "Nowhere.", "Wajaom Woodlands.", "Mamook.", "Aydeewa Subterrane." } },
  { "Travel to Wajaom Woodlands?", { "Add to favorites.", "Travel using 1000 gil. (97531 gil)", "Travel using 50 tabs. (0 total)", "Not just yet." } },
}
C.warp("wajaom")
assert(table.concat(picks, " ") == "2 5 4 2 2", table.concat(picks, " "))
