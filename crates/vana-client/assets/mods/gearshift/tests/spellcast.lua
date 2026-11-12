-- (Gearshift's XML front ends and regex: spellcast.)

local said, ran, sent, timers = {}, {}, {}, {}
local FILES = {}
local BUFFS = {}
local ITEMS = {
  [11] = { name = "Idle Hat", log_name = "idle hat", slots = 1 << 4, jobs = 1 << 3, level = 1 },
  [12] = { name = "Fast Hat", log_name = "fast hat", slots = 1 << 4, jobs = 1 << 3, level = 1 },
  [13] = { name = "Cast Coat", log_name = "cast coat", slots = 1 << 5, jobs = 1 << 3, level = 1 },
  [14] = { name = "Heal Gloves", log_name = "healing gloves", slots = 1 << 6, jobs = 1 << 3, level = 1 },
}
local PLAYER = { name = "Tester", id = 100, index = 1, main_job = 3, main_level = 75, sub_job = 4, sub_level = 37,
  hp = 900, hp_max = 1000, mp = 400, mp_max = 500, tp = 0, hpp = 90, mpp = 80, status = 0, zone = 230 }
local SPELLS = { [1] = { name = "Cure", kind = 1, skill = 33, element = 6, mp = 8, cast_time = 2, recast = 5, targets = 63 },
  [3] = { name = "Cure III", kind = 1, skill = 33, element = 6, mp = 46, cast_time = 2.5, recast = 6, targets = 63 },
  [144] = { name = "Fire", kind = 2, skill = 36, element = 0, mp = 7, cast_time = 1, recast = 2, targets = 32 },
  [338] = { name = "Utsusemi: Ichi", kind = 4, skill = 39, element = 2, mp = 0, cast_time = 4, recast = 30, targets = 1 } }
game = {
  player = function() return PLAYER end,
  buffs = function() return BUFFS end,
  party = function() return {} end,
  target = function() return { id = 200, index = 50, name = "Goblin", kind = "mob", hpp = 100, distance = 5, status = 1 } end,
  entity = function(id) if id == 100 then return { id = 100, index = 1, name = "Tester", kind = "player", me = true, hpp = 90 } end end,
  entities = function() return {} end,
  entity_by_tag = function() return nil end,
  time = function() return { earth = 0, weekday = 5, hour = 7, minute = 5, moon = 50, moon_phase = 0 } end,
  weather = function() return 4 end,
  equipment = function() return {} end,
  items = function(bag)
    if bag ~= 0 then return {} end
    return { { bag = 0, slot = 1, id = 11 }, { bag = 0, slot = 2, id = 12 }, { bag = 0, slot = 3, id = 13 }, { bag = 0, slot = 4, id = 14 } }
  end,
}
res = {
  job = function(id) return ({ "WAR", "MNK", "WHM", "BLM" })[id] end,
  status = function(id) return ({ [33] = "Haste", [5] = "Blindness" })[id] end,
  spell = function(id) return SPELLS[id] end,
  ability = function(id) if id == 0x200 + 10 then return { name = "Divine Seal", recast_id = 26, targets = 1 } end end,
  item = function(id) return ITEMS[id] end,
  zone = function() return "Southern San d'Oria" end,
  moon = function() return "Full Moon" end,
}
chat = { add = function(t) said[#said + 1] = t end }
clock = function() return 1000 end
timer = { after = function(secs, f) timers[#timers + 1] = { secs, f } end }
commands = { run = function(l) ran[#ran + 1] = l end }
packets = { inject_out = function(d) sent[#sent + 1] = d end, build = function(id, b) return string.pack("<I2I2", id, 0) .. b end }
fs = {
  list = function(dir)
    local t = {}
    for k in pairs(FILES) do
      local n = k:match("^" .. dir:gsub("%p", "%%%0") .. "([^/]+)$")
      if n then t[#t + 1] = n end
    end
    return t
  end,
  read = function(p) return FILES[p] end,
}
local function equips()
  local t = {}
  for _, d in ipairs(sent) do
    local id = string.unpack("<I2", d)
    if id == 0x050 then
      local index, slot = string.unpack("BB", d, 5)
      t[#t + 1] = slot .. ":" .. ITEMS[({ 11, 12, 13, 14 })[index]].name
    elseif id == 0x051 then
      local n = string.unpack("B", d, 5)
      for k = 0, n - 1 do
        local index, slot = string.unpack("BB", d, 9 + k * 4)
        t[#t + 1] = slot .. ":" .. ITEMS[({ 11, 12, 13, 14 })[index]].name
      end
    end
  end
  sent = {}
  table.sort(t)
  return table.concat(t, " ")
end
local function fire_timers()
  local list = timers
  timers = {}
  table.sort(list, function(a, b) return a[1] < b[1] end)
  local order = {}
  for _, x in ipairs(list) do
    x[2]()
    order[#order + 1] = string.format("%.1f", x[1])
  end
  return table.concat(order, " ")
end

local core = require("src.core")
local sc = require("src.spellcast")

-- Matching, variables, Advanced.
assert(sc._match("m*d*", "mad") and not sc._match("m*d*", "master") and sc._match("m???e?", "monkey"))
assert(sc._match("Cure*|Fire", "fire") and sc._match("(regex)^cure (ii|iii)$(/regex)", "Cure III"))
assert(not sc._match("(regex-case)^cure$(/regex)", "Cure"))
assert(sc._key("Cure III") == "cure3" and sc._key("Utsusemi: Ichi") == "utsusemiichi")
sc._uvars({ instra1 = { name = "instra1", value = "A" }, fireobi = { name = "FireObi", value = "Karin Obi" },
  use = { name = "use", value = "$" }, b = { name = "B", value = "bee" } })
assert(sc._expand("$instra11") == "A1", sc._expand("$instra11"))
assert(sc._expand("$%SpellElementObi", { spell = { element = "Fire" } }) == "Karin Obi")
assert(sc._expand("%MoonPCT %Moon %Time %Weather %MainJob/%SubJob") == "50 Full Moon 07.05 Fire WHM/BLM", sc._expand("%MoonPCT %Moon %Time %Weather %MainJob/%SubJob"))
assert(sc._expand("\\$B and $B") == "$B and bee")
local A = sc._advanced
assert(A('"%MainJob"="WHM" AND "%SubJob"="BLM"') and not A('"%MainJob"="WHM" AND %MPP < 50'))
assert(A("(float)%MP/%MPMax*100 > 79") and A("%MP/%MPMax*100 = 0") and A("!(%HPP < 50)"))
assert(A('(bool)strmatch("Southern*","%Area")') and A('(bool)isArea("*d\'Oria")') and A('(bool)regex("%HPP","8[5-9]|9.")'))
assert(not A('(bool)buffactive("Haste")') and A("(regex)%Weather=^fire$(/regex)") and not A("broken ((("))

FILES["spellcast/WHM.xml"] = [==[<?xml version="1.0" ?>
<spellcast xmlns:xi="http://www.w3.org/2001/XInclude">
  <config ShowGearSwaps="false" RequiredVersion="2.30"/>
  <variables clear="true">
    <var name="IdleSet">Standard</var>
  </variables>
  <sets>
    <group name="WHM|WhiteMage" default="yes">
      <set name="Standard"><head>Idle Hat</head></set>
      <set name="FastCast" baseset="Standard"><head>Fast Hat</head><body>Cast Coat</body></set>
      <set name="Cure"><hands>Healing Gloves</hands></set>
    </group>
  </sets>
  <rules>
    <xi:include href="inc.xml" xpointer="//include[@name='Auto']/*"/>
    <if Type="WhiteMagic|BlackMagic">
      <equip when="precast" set="FastCast"/>
    </if>
    <if Spell="Cure*">
      <action type="equip" when="midcast" set="Cure"/>
      <midcastdelay delay="0.4"/>
    </if>
    <elseif Spell="Fire*">
      <addtochat>fire chain</addtochat>
    </elseif>
    <if Spell="Trigger1">
      <var cmd="set IdleSet FastCast"/>
      <addtochat when="precast">idle $IdleSet</addtochat>
    </if>
    <if NotSpellTargetType="SELF" Spell="Fire">
      <cmd when="aftercast">input /p burned %SpellTargetName</cmd>
    </if>
    <equip when="aftercast" set="$IdleSet"/>
  </rules>
</spellcast>]==]
FILES["spellcast/inc.xml"] = [==[<includes><include name="Auto">
  <if spell="autoset"><equip when="idle" set="$IdleSet"/><return/></if>
</include></includes>]==]

assert(sc.find() == "spellcast/WHM.xml", tostring(sc.find()))
assert(sc.load(sc.find()), table.concat(said, " | "))

-- Cure III typed loosely: the line goes out canonical, the precast set (FastCast over its base) at once.
local out = sc.typed("/ma cure3 me")
assert(out == '/ma "Cure III" <me>', tostring(out))
assert(equips() == "4:Fast Hat 5:Cast Coat", "precast")
local when = fire_timers()
assert(when == "0.4 4.0", when)
local got = equips()
assert(got == "4:Idle Hat 6:Heal Gloves", got)

-- Fire at the target: aftercast command.
core.sent = {}
assert(sc.typed('/ma "Fire" <t>') == '/ma "Fire" <t>')
equips()
assert(said[#said] == "fire chain", "the elseif runs when the if before it failed")
fire_timers()
equips()
assert(ran[#ran] == "/p burned Goblin", tostring(ran[#ran]))

-- A trigger: never sent; its var and chat run.
assert(sc.typed("/ma Trigger1") == false)
assert(said[#said] == "idle FastCast", said[#said])

-- The idle autoset uses the variable now.
core.sent = {}
sc.status_change(0)
local idle = equips()
assert(idle == "4:Fast Hat 5:Cast Coat", idle .. " | " .. table.concat(said, " | "))

-- /sc takes over; other lines get their variables.
assert(sc.typed("/sc var set IdleSet Standard") == false and sc._uvars().idleset.value == "Standard")
assert(sc.typed("/p job %MainJob, %HPP%") == "/p job WHM, 90%")
assert(sc.typed("/echo plain") == nil)
