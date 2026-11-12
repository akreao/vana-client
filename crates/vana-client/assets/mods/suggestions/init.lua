-- The battle menu's suggestions (modern UI): this mod's rules (rules.lua) pick, every half second while the battle
-- menu shows its commands, the spells, abilities and weapon skills worth using now, and put them at the menu's top
-- (the battle_rows slot). While the cursor is on one of them the rows hold still and only their families are retaken.
-- Other mods get the picks as the `suggestions` event (mods.emit).

local R = require("rules")
local F = require("facts")

local rows = {}       -- the rules' rows now (with their families)
local sent = nil      -- the slot's value as last set
local at = -math.huge

-- The skillchain rules, from the Skillchain mod (none while it is not there).
local function chain_rules()
  local ok, r = pcall(mods.call, "skillchain", "rules")
  if not ok or type(r) ~= "table" then return nil end
  local names, map, elements = r.names, r.map, r.elements
  local C = {}
  function C.form(resonance, closer)
    for _, a in ipairs(resonance) do
      if a ~= 0 then
        for _, c in ipairs(closer) do
          if c ~= 0 then
            for _, e in ipairs(map) do
              if e[1] == c and e[2] == a then return e[3] end
            end
          end
        end
      end
    end
  end
  function C.level(p)
    if p >= 1 and p <= 8 then return 1 end
    if p >= 9 and p <= 12 then return 2 end
    if p == 13 or p == 14 then return 3 end
    if p == 15 or p == 16 then return 4 end
    return 0
  end
  function C.elements(p) return elements[p] or {} end
  local WORDS = { [0] = "Fire", "Ice", "Wind", "Earth", "Thunder", "Water", "Light", "Dark" }
  function C.chain_text(p)
    local els = {}
    for _, e in ipairs(C.elements(p)) do els[#els + 1] = WORDS[e] end
    return ("%s (%s)"):format(names[p] or "", table.concat(els, ", "))
  end
  return C
end

local function set(value)
  local key = value and json.encode(value) or ""
  if key ~= sent then
    ui.native.slot("battle_rows", value)
    sent = key
  end
end

local function tick()
  local menu = settings.get("on") and ui.native.battle_menu() or nil
  if not menu then
    if #rows > 0 then mods.emit("suggestions", {}) end
    rows = {}
    set(nil)
    return
  end
  if os.clock() - at < 0.5 then return end
  at = os.clock()
  R.chain = R.chain or chain_rules()
  local f = F.gather()
  if menu.row and #rows > 0 then
    R.regroup(f, rows)
  else
    local out = R.suggest(f)
    local before = json.encode(R.rows(rows))
    rows = out
    local now = R.rows(rows)
    if json.encode(now) ~= before then mods.emit("suggestions", now) end
  end
  set(#rows > 0 and R.rows(rows) or nil)
end

if ui.native.battle_menu then
  timer.every(0.1, tick)
  events.on("settings_changed", tick)
  events.on("zone_change", function() R.chain = nil end)
else
  -- An older client runs the rules itself and sends their rows as the suggestions event, while a mod listens.
  local listening
  local function show(r) ui.native.slot("battle_rows", #r > 0 and r or nil) end
  local function apply()
    if settings.get("on") and not listening then
      listening = events.on("suggestions", show)
    elseif not settings.get("on") and listening then
      events.off(listening)
      listening = nil
      ui.native.slot("battle_rows", nil)
    end
  end
  apply()
  events.on("settings_changed", apply)
end
