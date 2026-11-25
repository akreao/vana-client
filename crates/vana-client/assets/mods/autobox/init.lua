-- Autobox: ordered rules for your own character, one sheet per main job. Each rule is a target, one or two
-- conditions and an action; every 0.2 s the first rule that holds and that the game would take now is done (see
-- src/core.lua). It waits while you move, cast, talk to an NPC or zone, and it only does what the magic and ability
-- menus would let you do. //autobox opens the sheet, //autobox on | off switches it.

local core = require("src.core")
local presets = require("src.presets")

local W, R = "autobox", "autobox_rule"
local s = core.new()
local view = { at = 1, editing = nil, draft = nil }

local function say(t) chat.add("Autobox: " .. t) end

local function saved() return storage.character end

local function job()
  local p = game.player()
  return p and p.main_job and res.job(p.main_job) or nil
end

-- The main job's sheet; a job seen the first time starts from its preset.
local function rules()
  local j = job()
  if not j then return {} end
  local all = saved().rules or {}
  saved().rules = all
  if not all[j] then
    all[j] = presets.for_job(j)
    storage.save()
  end
  return all[j]
end

local function on() return saved().on == true end

-- Name lookups, built once the name tables have loaded.
local names = nil
local function lookups()
  if names then return names end
  local spells, abilities, statuses = {}, {}, {}
  for id = 1, 1023 do
    local r = res.spell(id)
    if r and r.name and r.name ~= "" and not spells[r.name:lower()] then spells[r.name:lower()] = id end
  end
  for id = 1, 1535 do
    local r = res.ability(id)
    if r and r.name and r.name ~= "" and (r.sub or 0) == 0 and not abilities[r.name:lower()] then
      abilities[r.name:lower()] = id
    end
  end
  for id = 0, 1023 do
    local n = res.status(id)
    if n and n ~= "" then
      local k = n:lower()
      statuses[k] = statuses[k] or {}
      table.insert(statuses[k], id)
    end
  end
  if next(spells) == nil then return nil end
  names = { spells = spells, abilities = abilities, statuses = statuses }
  return names
end

local function env()
  local n = lookups()
  if not n then return nil end
  local me = game.player()
  local buffs = {}
  for _, b in ipairs(game.buffs()) do buffs[b.id] = true end
  local known = {}
  for _, id in ipairs(game.spells()) do known[id] = true end
  local party, ours = game.party(), { [me.id] = true }
  for _, m in ipairs(party) do ours[m.id] = true end
  local info = game.info()
  local pet = info.pet_index and game.entity(info.pet_index) or nil
  return {
    now = clock(), on = on(), me = me, buffs = buffs, known = known, party = party, ours = ours,
    target = game.target(), pet = pet, entity = game.entity, rules = rules(),
    casting = game.cast() ~= nil, event = info.event,
    usable = function(kind, id) return (game.usable(kind, id)) end,
    spell = res.spell, ability = res.ability,
    spell_id = function(name) return n.spells[name:lower()] end,
    ability_id = function(name) return n.abilities[name:lower()] end,
    status_ids = function(name) return n.statuses[name:lower()] end,
  }
end

local function do_action(a)
  if a.kind == "spell" then
    actions.cast(a.id, a.tag)
  elseif a.kind == "ability" then
    actions.ability(a.id, a.tag)
  elseif a.kind == "ranged" then
    actions.ranged(a.tag)
  elseif a.kind == "attack" then
    actions.attack(a.tag)
  end
end

-- The sheet window ------------------------------------------------------------------------------------------------

local function rows()
  local out = {}
  for i, r in ipairs(rules()) do
    local when = core.condition_text(r.when)
    if r.when2 then when = when .. ", " .. core.condition_text(r.when2) end
    out[#out + 1] = { tostring(i), r.on == false and "off" or "on", core.TARGET_LABELS[r.target] or "?", when,
      core.action_text(r.action), colour = r.on == false and { 150, 150, 150, 255 } or nil }
  end
  return out
end

local function status_text()
  if not on() then return "Switched off." end
  local last = s.last and ("Last: %s on %s (rule %d)."):format(s.last.text, s.last.who, s.last.rule) or ""
  return (s.holding and ("Holding: " .. s.holding .. ". ") or "Running. ") .. last
end

local function refresh()
  if not ui.is_open(W) then return end
  ui.update(W, "job", { text = ("Rules for %s, tried from the top; the first that holds is done."):format(job() or "?") })
  ui.update(W, "switch", { marked = on() and 1 or 2 })
  ui.update(W, "rules", { rows = rows(), at = math.max(1, math.min(view.at, #rules())) })
  ui.update(W, "status", { text = status_text() })
end

ui.window(W, {
  title = "Autobox",
  width = 640,
  animate = true,
  help = "Rules for your own character, one sheet per main job. Each tick the first rule whose target is there, whose "
    .. "conditions hold and whose action the game would take now is done. It waits while you move, cast, talk to an "
    .. "NPC or zone.",
  elements = {
    { type = "buttons", id = "switch", label = "Autobox", items = { "On", "Off" }, marked = 2 },
    { type = "text", id = "job", text = "" },
    { type = "list", id = "rules", height = 10, rows = {},
      columns = { { title = "#", width = 26 }, { title = "", width = 34 }, { title = "Target", width = 120 },
        { title = "When" }, { title = "Do", width = 160 } } },
    { type = "buttons", id = "ops", items = { "Add", "Edit", "On / Off", "Delete", "Up", "Down" },
      tip = "Edit (or a click on a rule) opens it in its own window." },
    { type = "buttons", id = "sheet", items = { "Start over from the preset" },
      tip = "Replaces this job's rules with the starting sheet." },
    { type = "text", id = "status", text = "", dim = true },
  },
})

-- The rule window -------------------------------------------------------------------------------------------------

local COND_ITEMS = {}
for i, c in ipairs(core.CONDITIONS) do COND_ITEMS[i] = c.label end
local COND2_ITEMS = { "Nothing more" }
for i = 2, #core.CONDITIONS do COND2_ITEMS[#COND2_ITEMS + 1] = core.CONDITIONS[i].label end
local TARGET_ITEMS, ACTION_ITEMS = {}, {}
for i, t in ipairs(core.TARGETS) do TARGET_ITEMS[i] = core.TARGET_LABELS[t] end
for i, a in ipairs(core.ACTIONS) do ACTION_ITEMS[i] = core.ACTION_LABELS[a] end

local function index_of(list, v)
  for i, x in ipairs(list) do if x == v then return i end end
  return 1
end

-- What the action dropdown offers for a kind: the names you have now, sorted, and the rule's own name kept.
local function what_items(kind, current)
  local list, seen = {}, {}
  local function add(n) if n and n ~= "" and not seen[n] then seen[n] = true; list[#list + 1] = n end end
  if kind == "spell" then
    for _, id in ipairs(game.spells()) do
      local r = res.spell(id)
      if r and r.name and r.kind ~= 8 then add(r.name) end
    end
  elseif kind == "ability" or kind == "ws" then
    for _, id in ipairs(game.abilities()) do
      local r = res.ability(id)
      if r and r.name and (r.sub or 0) == 0 and ((kind == "ws") == (id < 512)) and r.kind ~= 4 then add(r.name) end
    end
  end
  table.sort(list)
  if kind == "ws" then table.insert(list, 1, "Best") end
  if current and current ~= "" and not seen[current] then table.insert(list, kind == "ws" and 2 or 1, current) end
  if #list == 0 then list[1] = "-" end
  return list
end

ui.window(R, {
  title = "Rule",
  width = 440,
  animate = true,
  elements = {
    { type = "text", id = "which", text = "" },
    { type = "choice", id = "on", label = "Rule", items = { "On", "Off" }, at = 1 },
    { type = "dropdown", id = "target", label = "Target", items = TARGET_ITEMS, at = 1,
      tip = "Party member: everyone in your party within casting range, the lowest HP first. Battle target: your target "
        .. "when it is a monster you or your party claimed (or any while you are engaged)." },
    { type = "dropdown", id = "cond", label = "When", items = COND_ITEMS, at = 1 },
    { type = "input", id = "arg", label = "Value", text = "", max = 24,
      tip = "A number for HP, MP and TP; a status effect's name (Protect, poison...) for Lacks / Has status. "
        .. "A monster's status effects are not known, so a status condition on the battle target never holds." },
    { type = "dropdown", id = "cond2", label = "And when", items = COND2_ITEMS, at = 1 },
    { type = "input", id = "arg2", label = "Value", text = "", max = 24 },
    { type = "dropdown", id = "kind", label = "Do", items = ACTION_ITEMS, at = 1 },
    { type = "dropdown", id = "what", label = "What", items = { "-" }, at = 1 },
    { type = "choice", id = "best", label = "Spell tier", items = { "This spell", "Best tier I can cast" }, at = 1,
      tip = "Best tier: Cure picks Cure IV, III, II or Cure, the highest you know and can cast now." },
    { type = "text", id = "problem", text = "", colour = { 235, 120, 110 } },
    { type = "buttons", id = "done", items = { "Save", "Cancel" } },
  },
})

local function draft_what()
  local d = view.draft
  local a = d.action
  if a.kind == "spell" or a.kind == "ability" or a.kind == "ws" then
    local items = what_items(a.kind, a.best and a.kind == "ws" and "Best" or a.name)
    local at = index_of(items, a.kind == "ws" and a.best and "Best" or a.name)
    if not a.name or a.name == "" then a.name = items[at] ~= "-" and items[at] ~= "Best" and items[at] or a.name end
    if a.kind == "ws" and items[at] == "Best" then a.best = true end
    return items, at
  end
  return { "-" }, 1
end

local function show_rule()
  local d = view.draft
  local items, at = draft_what()
  ui.update(R, "which", { text = view.editing and ("Rule %d of %s"):format(view.editing, job() or "?") or "New rule" })
  ui.update(R, "on", { at = d.on == false and 2 or 1 })
  ui.update(R, "target", { at = index_of(core.TARGETS, d.target) })
  ui.update(R, "cond", { at = core.CONDITION_BY_KIND[d.when.kind].index })
  ui.update(R, "arg", { text = d.when.value and tostring(d.when.value) or "" })
  ui.update(R, "cond2", { at = d.when2 and core.CONDITION_BY_KIND[d.when2.kind].index or 1 })
  ui.update(R, "arg2", { text = d.when2 and d.when2.value and tostring(d.when2.value) or "" })
  ui.update(R, "kind", { at = index_of(core.ACTIONS, d.action.kind) })
  ui.update(R, "what", { items = items, at = at })
  ui.update(R, "best", { at = d.action.kind == "spell" and d.action.best and 2 or 1 })
  ui.update(R, "problem", { text = "" })
end

local function copy(t)
  if type(t) ~= "table" then return t end
  local out = {}
  for k, v in pairs(t) do out[k] = copy(v) end
  return out
end

local function open_rule(i)
  view.editing = i
  view.draft = i and copy(rules()[i]) or
    { on = true, target = "me", when = { kind = "always" }, action = { kind = "spell", name = "", best = false } }
  view.draft.when = view.draft.when or { kind = "always" }
  view.draft.action = view.draft.action or { kind = "spell" }
  ui.open(R)
  show_rule()
end

local function value_of(kind, text)
  local d = core.CONDITION_BY_KIND[kind]
  if d and d.value == "number" then return tonumber(text) end
  if d and d.value == "status" then return text ~= "" and text or nil end
  return nil
end

ui.on(R, function(_, e)
  local d = view.draft
  if not d then return end
  if e.kind == "set" and e.id == "on" then
    d.on = e.index == 1
  elseif e.kind == "set" and e.id == "target" then
    d.target = core.TARGETS[e.index]
  elseif e.kind == "set" and e.id == "cond" then
    d.when = { kind = core.CONDITIONS[e.index].kind, value = value_of(core.CONDITIONS[e.index].kind, ui.get(R, "arg").text or "") }
  elseif e.kind == "set" and e.id == "cond2" then
    d.when2 = e.index > 1 and { kind = core.CONDITIONS[e.index].kind,
      value = value_of(core.CONDITIONS[e.index].kind, ui.get(R, "arg2").text or "") } or nil
  elseif e.kind == "change" and e.id == "arg" then
    d.when.value = value_of(d.when.kind, e.text)
  elseif e.kind == "change" and e.id == "arg2" and d.when2 then
    d.when2.value = value_of(d.when2.kind, e.text)
  elseif e.kind == "set" and e.id == "kind" then
    d.action = { kind = core.ACTIONS[e.index], best = false }
    local items, at = draft_what()
    ui.update(R, "what", { items = items, at = at })
  elseif e.kind == "set" and e.id == "what" then
    local items = what_items(d.action.kind, d.action.name)
    local pick = items[e.index]
    if d.action.kind == "ws" then
      d.action.best = pick == "Best"
      d.action.name = pick ~= "Best" and pick or nil
    elseif pick ~= "-" then
      d.action.name = pick
    end
  elseif e.kind == "set" and e.id == "best" then
    d.action.best = e.index == 2
  elseif e.kind == "press" and e.id == "done" then
    if e.index == 2 then
      ui.close(R)
      view.draft = nil
      return
    end
    if d.action.kind ~= "spell" and d.action.kind ~= "ws" then d.action.best = nil end
    if d.action.kind == "ranged" or d.action.kind == "attack" then d.action.name = nil end
    local ok, why = core.check(d)
    if not ok then
      ui.update(R, "problem", { text = "Not saved: " .. why .. "." })
      return
    end
    local list = rules()
    if view.editing and list[view.editing] then list[view.editing] = d else list[#list + 1] = d; view.at = #list end
    storage.save()
    ui.close(R)
    view.draft = nil
    s.rule_rest = {}
    refresh()
  elseif e.kind == "close" then
    view.draft = nil
  end
end)

ui.on(W, function(_, e)
  local list = rules()
  if e.kind == "set" and e.id == "switch" then
    saved().on = e.index == 1
    storage.save()
    refresh()
  elseif e.kind == "select" and e.id == "rules" then
    view.at = e.index
    open_rule(e.index)
  elseif e.kind == "press" and e.id == "ops" then
    view.at = (ui.get(W, "rules") or {}).at or view.at
    local i = view.at
    if e.index == 1 then
      open_rule(nil)
    elseif not list[i] then
      return
    elseif e.index == 2 then
      open_rule(i)
    elseif e.index == 3 then
      list[i].on = list[i].on == false
    elseif e.index == 4 then
      table.remove(list, i)
      view.at = math.max(1, i - 1)
    elseif e.index == 5 and i > 1 then
      list[i], list[i - 1] = list[i - 1], list[i]
      view.at = i - 1
    elseif e.index == 6 and i < #list then
      list[i], list[i + 1] = list[i + 1], list[i]
      view.at = i + 1
    end
    s.rule_rest = {}
    storage.save()
    refresh()
  elseif e.kind == "press" and e.id == "sheet" then
    local j = job()
    if not j then return end
    saved().rules = saved().rules or {}
    saved().rules[j] = presets.for_job(j)
    storage.save()
    view.at = 1
    s.rule_rest = {}
    refresh()
    say(("%s's rules are the starting sheet again."):format(j))
  end
end)

-- The engine ------------------------------------------------------------------------------------------------------

local shown = nil
timer.every(0.2, function()
  if not settings.get("on") then return end
  local p = game.player()
  if not p or not p.main_job then return end
  local e = env()
  if not e then return end
  local a = core.tick(s, e)
  if a then do_action(a) end
  local text = status_text()
  if text ~= shown and ui.is_open(W) then
    shown = text
    ui.update(W, "status", { text = text })
  end
end)

events.on("zone_change", function() core.zoned(s, clock()) end)
events.on("job_change", function()
  s.rule_rest = {}
  refresh()
end)
events.on("settings_changed", function(key, value)
  if key == "on" and not value then
    ui.close(W)
    ui.close(R)
  end
end)

commands.register("autobox", function(what)
  if not settings.get("on") then return end
  what = (what or ""):lower()
  if what == "" then
    if ui.is_open(W) then ui.close(W) else ui.open(W); refresh() end
  elseif what == "on" or what == "off" then
    saved().on = what == "on"
    storage.save()
    refresh()
    say(what == "on" and ("on (%s rules)."):format(job() or "?") or "off.")
  else
    for _, line in ipairs(commands.usage("autobox")) do chat.add(line) end
  end
end)
