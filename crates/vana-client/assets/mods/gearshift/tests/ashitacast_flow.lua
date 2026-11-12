-- (Gearshift's XML front ends and regex: ashitacast flow.)

local said, ran, sent = {}, {}, {}
local FILES = {}
local BUFFS = {}
local ITEMS = {
  [11] = { name = "Idle Hat", slots = 1 << 4, jobs = 1 << 1, level = 1 },
  [12] = { name = "Acc Hat", slots = 1 << 4, jobs = 1 << 1, level = 1 },
  [13] = { name = "Ear A", slots = (1 << 11) | (1 << 12), jobs = 1 << 1, level = 1 },
  [14] = { name = "Body B", slots = 1 << 5, jobs = 1 << 1, level = 1 },
}
local PLAYER = { name = "Tester", id = 100, index = 1, main_job = 1, main_level = 75, sub_job = 13, sub_level = 37,
  hp = 900, hp_max = 1000, mp = 50, mp_max = 100, tp = 0, hpp = 90, mpp = 50, status = 0, zone = 230, pos = { x = 0, y = 0, z = 0 } }
game = {
  player = function() return PLAYER end,
  buffs = function() return BUFFS end,
  party = function() return {} end,
  target = function() return nil end,
  entity = function() return nil end,
  entities = function() return {} end,
  entity_by_tag = function() return nil end,
  time = function() return { earth = 0, weekday = 0, hour = 7, minute = 5 } end,
  weather = function() return 0 end,
  equipment = function() return {} end,
  items = function(bag)
    if bag ~= 0 then return {} end
    return { { bag = 0, slot = 1, id = 11 }, { bag = 0, slot = 2, id = 12 }, { bag = 0, slot = 3, id = 13 }, { bag = 0, slot = 4, id = 14 } }
  end,
}
res = {
  job = function(id) return ({ "WAR", "MNK", "WHM", [13] = "NIN" })[id] end,
  status = function(id) return ({ [33] = "Haste" })[id] end,
  spell = function() return nil end,
  ability = function() return nil end,
  item = function(id) return ITEMS[id] end,
  zone = function() return "Somewhere" end,
}
chat = { add = function(t) said[#said + 1] = t end }
clock = function() return 1000 end
timer = { after = function(_, f) f() end }
commands = { run = function(l) ran[#ran + 1] = l end }
packets = { inject_out = function(d) sent[#sent + 1] = d end, build = function(id, b) return string.pack("<I2", id) .. b end }
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
local function ids()
  local t = {}
  for _, d in ipairs(sent) do
    local id = string.unpack("<I2", d)
    if id == 0x050 then
      local index, slot = string.unpack("BB", d, 3)
      t[#t + 1] = string.format("%d:%d", slot, index)
    else
      t[#t + 1] = string.format("%03X", id)
    end
  end
  sent = {}
  return table.concat(t, " ")
end

local core = require("src.core")
local ac = require("src.acxml")

FILES["ashitacast/Tester_WAR.xml"] = [==[<?xml version="1.0"?>
<!-- a comment with <tags> in it -->
<ashitacast>
  <sets>
    <set name="Idle"><head>Idle Hat</head></set>
    <set name="Acc" baseset="Idle"><head>Acc Hat</head><body>Body B</body></set>
  </sets>
  <load><setvar name="mode" value="Idle"/><echo>loaded $p_name</echo></load>
  <commands>
    <command input="/mode"><setvar name="mode" value="$arg1"/><echo>mode $u_mode</echo></command>
    <command input="/modex"><echo>never: the shorter input wins</echo></command>
  </commands>
  <default>
    <if u_mode="Acc"><equip set="Acc"/></if>
    <else><equip set="Idle"/></else>
  </default>
</ashitacast>]==]

assert(ac.find() == "ashitacast/Tester_WAR.xml", tostring(ac.find()))
assert(ac.load(ac.find()), table.concat(said, " | "))
assert(said[#said] == "loaded Tester", said[#said])
ac.tick()
assert(ids() == "4:1", "idle hat")
-- /modex is taken by /mode (a prefix, the shorter input); the line is consumed.
assert(ac.typed("/modex Acc") == false and said[#said] == "mode Acc", said[#said])
assert(ac.typed("/other") == nil)
core.sent = {}
ac.tick()
assert(ids() == "4:2 5:4", "acc set over its baseset")

-- A v3 file through the import rules: idlegear, buffactive, lear, variables.
ac.unload()
core.sent = {}
FILES["ashitacast/Tester_WAR.xml"] = [==[<ashitacast>
  <variables><var name="x">7</var></variables>
  <sets><set name="Ears" lock="true"><lear>Ear A</lear></set></sets>
  <idlegear>
    <if buffactive="Haste"><equip set="Ears"/></if>
    <elseif advanced="%p_hpp &lt; 50"><equip><head>Acc Hat</head></equip></elseif>
    <doidlegear/>
  </idlegear>
</ashitacast>]==]
assert(ac.load(ac.find()), table.concat(said, " | "))
assert(ac._uvars().x == "7")
ac.tick()
assert(ids() == "", "no haste: nothing")
BUFFS = { { id = 33 } }
ac.tick()
assert(ids() == "11:3", "ear1 from lear under haste")
