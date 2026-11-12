-- Training Regime's page labels, credit attribution, and its block on the real zone 100 dialogue.

TEST.grant("packets.read")
local M = load(TEST.source("init.lua"), "@init.lua")()
local label = M.label
assert(label("\x0a\x00 \x7f\x92\x00[Siege Bat/Siege Bats].") == "Siege Bat")
assert(label("\x0a\x01 member\x7f\x92\x01[/s] of the worm family.") == "Worm family")
assert(label("\x0a\x02 undead-type creature\x7f\x92\x02[/s].") == "Undead-type")
-- Credits: by counts, then names; a family slot takes the monster no name fits; nothing fits a full slot.
local function attribute(slots, k, n, name)
  local t = {}
  for i, s in ipairs(slots) do t[i] = { label = s[1], needed = s[2], killed = s[3], known = s[4] } end
  return M.attribute(t, k, n, name)
end
local s = { { "Siege Bat", 4, 2, true }, { "Borer Beetle", 2, 0, true } }
assert(attribute(s, 3, 4, nil) == 1 and attribute(s, 1, 2, nil) == 2)
assert(attribute({ { "Goblin family", 4, 1, true }, { "Gigas family", 4, 1, true } }, 2, 4, "Goblin Mugger") == 1)
s = { { "Crab family", 6, 2, true }, { "Hill Lizard", 6, 2, true } }
assert(attribute(s, 3, 6, "Snipper") == 1 and attribute(s, 3, 6, "Hill Lizard") == 2)
-- Counts unknown (a book's page, no kills yet): the name decides.
assert(attribute({ { "Siege Bat", 0, 0, false }, { "Borer Beetle", 0, 0, false } }, 3, 4, "Borer Beetle") == 2)
assert(M.matches("Bat family", "Seeker Bats") and M.matches("Siege Bat", "siege bat")
  and not M.matches("Siege Bat", "Siege Bats"))

-- The real block of zone 100, through stand-ins for res.zone_text / zone_text_find.
local lines, n = TEST.zone_text(100)
if not lines then return end -- the game's files are not installed here
res.zone_text = function(_, id) return lines[id] end
res.zone_text_find = function(_, p)
  for i = 0, n - 1 do
    if lines[i] and lines[i]:sub(1, #p) == p then return i end
  end
end
local r = res.zone_text_find(100, "New training regime registered!")
local l, area = M.page(100, r, 2)
assert(#l == 1 and l[1] == "Forest Hare", table.concat(l, ", "))
assert(area:find("Ronfaure", 1, true), area)
l, area = M.page(100, r, 704)
assert(table.concat(l, ", ") == "Siege Bat, Borer Beetle" and area == "Garlaige Citadel", table.concat(l, ", ") .. " / " .. area)
