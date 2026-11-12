-- Content Trackers: each content's lines become its window's rows; a new zone starts afresh; Odyssey only from
-- Rabao; the pop windows.

local F, T, W, Z = nil, nil, nil, 1
local NAMES = { [1] = "Dynamis - San d'Oria", [2] = "Abyssea - Konschtat", [3] = "Zhayolm Remnants", [4] = "Nyzul Isle",
  [292] = "Reisenjima Henge", [264] = "Yorcia Weald [U]", [261] = "Ceizak Battlegrounds", [262] = "Foret de Hennetiel",
  [247] = "Rabao", [298] = "Walk of Echoes [P1]", [279] = "Walk of Echoes [P2]", [275] = "Outer Ra'Kaznar [U1]" }
local WINS, ON, CMDS, SAID, UPD = {}, {}, {}, {}, {}
settings = { get = function() return true end }
filters = { chat = function(f) F = f end }
timer = { every = function(_, f) T = f end }
text = { strip = function(t) return t end }
commands = { register = function(n, f) CMDS[n] = f end }
chat = { add = function(t) SAID[#SAID + 1] = t end }
storage = { character = {} }
ui = { window = function(id, spec) WINS[id] = spec if id == "content" then W = spec end end, open = function() end,
  close = function(id) WINS[id] = nil if id == "content" then W = nil end end, on = function(id, f) ON[id] = f end,
  is_open = function(id) return WINS[id] ~= nil end, update = function(id, el, f) UPD[#UPD + 1] = { id, el, f } end }
res = { zone = function(id) return NAMES[id] end, item = function(id) return { name = "item" .. id } end,
  key_item = function(id) return { name = "ki" .. id } end }
local bag = { { id = 1452, count = 30 }, { id = 1453, count = 2 } }
local KEYS, BUFFS, POOL, COUNT = { 1546, 9 }, {}, {}, nil
game = { player = function() return { zone = Z, name = "Lnxcsfix" } end, items = function(b) return b == 0 and bag or {} end,
  key_items = function() return KEYS end, buffs = function() return BUFFS end, treasure = function() return POOL end,
  countdown = function() return COUNT end }
load(TEST.source("init.lua"), "@init.lua")()
local function texts()
  local out = {}
  for _, e in ipairs(W.elements) do out[#out + 1] = e.text end
  return table.concat(out, "|")
end
local function el(t)
  for _, e in ipairs(W.elements) do if e.text == t then return e end end
end
T()
F("The sands of the hourglass have begun to fall.\nYou have 60 minutes (Earth time) remaining in Dynamis.")
F("Your stay in Dynamis has been extended by 10 minutes.")
T()
assert(W.title == "Dynamis")
assert(texts() == "69:59|Bronzepiece: 30|Silverpiece: 2|Currency worth 230 singles|Granules: Azure"
  or texts() == "70:00|Bronzepiece: 30|Silverpiece: 2|Currency worth 230 singles|Granules: Azure", texts())
Z = 2
T()
F("Visitant Light Intensity")
F("Pearlescent: 12 / Ebon: 3")
F("Golden: 40 / Silvery: 5")
F("Azure: 100 / Ruby: 0 / Amber: 255")
F("Lnxcsfix's body emits a strong golden light!")
F("Obtained 120 cruor. (Total: 4410)")
F("Exiting in 5 minutes.")
T()
assert(W.elements[1].text == "4:59" or W.elements[1].text == "5:00", W.elements[1].text)
F("Your visitant status will wear off in 30 minutes.")
T()
assert(texts():find("Pearlescent: 12|Ebon: 3|Golden: 40 %+|Silvery: 5|Azure: 100|Ruby: 0|Amber: 255|Cruor: 4410"), texts())
Z = 3
bag = { { id = 5365, count = 1 }, { id = 5376, count = 2 } }
T()
F("Main Weapon/Sub-Weapon restriction removed.")
F("STR Down removed.")
T()
local t = texts()
assert(not t:find("Main Weapon") and not t:find("STR Down") and t:find("Head/Neck|Body"), t)
Z = 4
T()
F("Transfer complete. Welcome to Floor 21.")
F("Objective: Eliminate all enemies.")
F("You obtain 160 tokens!")
F("Floor 21 objective complete. Rune of Transfer activated.")
T()
assert(texts() == "Floor 21 \u{2014} cleared|Eliminate all enemies|Tokens this run: 160", texts())

-- Omen: the floor objective, omens, the bonus objectives; a cleared floor's next objective drops them.
Z = 292
T()
F("Vanquish 5 sweetwater foes.")
F("There are 12 omens from your foes!")
F("Follow the light and carry out your charge before time runs out.\nYou have 120 seconds remaining.")
F("1: Deal 3 critical hits to your foes.")
F("2: Reduce your foe's HP by at least 2000 using a single weapon skill.")
F("3: Restore at least 500 HP 4 times.")
F("1: You have dealt 3 critical hits to your foes.")
F("3: You have restored at least 500 HP 2 times.")
F("2: You have failed to reduce your foe's HP by at least 2000 using a single weapon skill.")
T()
assert(W.title == "Omen")
t = texts()
assert(t:find("Floor 1  \u{00b7}  Omens: 12|Vanquish 5 sweetwater foes|", 1, true), t)
assert(el("1  Critical hits  3/3").colour[2] == 220 and el("2  Weapon skill damage  at least 2000").colour[1] == 235, t)
assert(el("3  Cures  2/4") and el("3  Cures  2/4").colour == nil, t)
F("A spectral light flares up.")
F("Your stay has been extended by 5 minutes.")
F("Vanquish all monsters.")
T()
t = texts()
assert(t == "Floor 2  \u{00b7}  Omens: 12|Vanquish all monsters", t)

-- Delve: a fracture's plasm, kills, stage and airlixirs; in a field zone the plasm opens the window.
Z = 264
T()
F("You receive 150 corpuscles of mweya plasm.")
F("You receive 500 corpuscles, making for a total of 12500.")
F("Your ears pick up a discordant howl.")
F("You find an airlixir +1 on the Yorcia Bloodfly.")
T()
assert(W.title == "Delve")
assert(texts() == "Plasm: 650 (total 12500)|Kills (estimate): 4|3 of 6|Airlixirs: 0  \u{00b7}  +1: 1  \u{00b7}  +2: 0", texts())
Z = 261
T()
assert(W == nil)
F("You receive 100 corpuscles of mweya plasm.")
T()
assert(W.title == "Delve" and texts():find("Plasm: 100", 1, true), texts())

-- A reive: ours joined, momentum, bayld (a quest's is not counted).
Z = 262
T()
F("Lnxcsfix has joined the Colonization Reive!")
F("Reive momentum score: Physical attack.")
F("Momentum bonus: HP recovery!")
F("Momentum Bonus: Fast Cast!")
F("Lnxcsfix obtained 300 bayld!")
F("You have obtained 50 bayld!")
T()
assert(W.title == "Reive", tostring(W and W.title))
t = texts()
assert(t:find("Colonization Reive|", 1, true) and t:find("Bayld: 300", 1, true) and t:find("1  Physical attack", 1, true)
  and t:find("HP recovery \u{00d7}1", 1, true) and t:find("Fast Cast \u{00d7}1", 1, true), t)

-- Odyssey: armed in Rabao, the run, the last run back in Rabao; the moglophone's wait.
KEYS = { 3212 }
Z = 247
T()
assert(W.title == "Odyssey" and texts() == "Moglophone: in hand", texts())
F("Entering Sheol B: Difficult.")
T()
KEYS = {}
Z = 298
T()
assert(W.title == "Odyssey")
F("You receive 20 moogle segments for a total of 1020.")
F("You receive 5 izzat, for a total of 35.")
F("Moogle Mastery is now level IV.")
T()
assert(texts() == "Sheol B (Difficult)|Segments: 20 (total 1020)|Izzat: 5 (total 35)|Moogle Mastery: IV", texts())
Z = 247
T()
t = texts()
assert(t:find("Last run: Sheol B (Difficult)|Segments: 20 (total 1020)|Another moglophone in about ", 1, true), t)
Z = 279
T()
assert(W == nil, "not entered from Rabao: not Odyssey")

-- Sortie: our gallimaufry, coffer reports, temporary items; the game's countdown for the time.
Z = 275
COUNT = { left = 600, length = 1800 }
T()
F("Lnxcsfix received 100 gallimaufry for a total of 900.")
F("Someone received 50 gallimaufry for a total of 60.")
F("#B treasure coffer status report: 3/3 #?: 1/1.")
F("You obtain the temporary item: Ra'Kaznarian Ether!")
T()
assert(W.title == "Sortie")
t = texts()
assert(t:find("^10:00|Gallimaufry: 100 %(total 900%)|#A  no report yet|#B  3/3   #%?: 1/1|"), t)
assert(el("#B  3/3   #?: 1/1").colour[2] == 220 and t:find("Temporary items: Ra'Kaznarian Ether", 1, true), t)

-- Pop lists: an NM's tree with what we hold; the NM list; the Zeni NMs and a ???'s place.
KEYS = { 1530 }
bag = { { id = 3262, count = 1 }, { id = 3291, count = 12 } }
CMDS.pops("alf")
local P = WINS.pops
assert(P.title == "Alfard", P.title)
local tree = P.elements[2].rows
assert(#tree == 5 and tree[1].colour[2] == 220 and tree[2].colour[2] == 220 and tree[3].colour[1] == 235, #tree)
assert(tree[1].cells[1] == "ki1530" and tree[4].cells[1] == "        item3267" and tree[1].cells[2] == "Ningishzida  Forced (I-7/I-8)",
  tree[4].cells[1])
assert(P.elements[3].text == "Every pop held" and P.elements[4].text == "item3291: 12 / 75", P.elements[4].text)
T()
assert(#UPD > 0 and UPD[1][1] == "pops")
CMDS.pops("")
assert(WINS.pops.title == "Pop lists" and #WINS.pops.elements[1].rows == 28)
ON.pops("pops", { kind = "select", id = "nms", index = 2 })
assert(WINS.pops.title == "Amphitrite")
CMDS.pops("nobody")
assert(SAID[#SAID]:find("No pop list", 1, true))
CMDS.znm()
assert(WINS.znm.elements[1].type == "tabs" and #WINS.znm.elements[1].items == 4)
ON.znm("znm", { kind = "select", id = "t1_1", index = 1 })
assert(SAID[#SAID] == "Vulpangue: its ??? is at (-697, -7, -123) in zone 51.", SAID[#SAID])
