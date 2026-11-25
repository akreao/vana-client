-- XIFO Squad: a window for the squad mates of an XI: Forever Offline server. It shows each mate's jobs, role, mode
-- and switches, and calls, dismisses and changes them with the server's own "!squad" commands (the ones a player
-- could type). The profile window shows one mate's stats, gear, jobs and actions. Everything goes through the XIFO
-- Link mod, which says the commands and hides the server's answers.

local P = require("src.parse")

local LINK = "xifo-link"
local ROLE_LABELS = { "Auto", "Tank", "Damage", "Healer", "Support" }
local MODE_LABELS = { "Balanced", "Aggressive", "Conserve" }
local SW_LABELS = { "Melee", "Ranged", "Abilities", "Magic" }
local DIM = { 160, 160, 160 }

local st = {
  panel = nil,        -- the last "!squad panel" answer (src/parse.lua)
  sel = 1,            -- the picked mate's row
  asked = -math.huge, -- when the panel was last asked for
  sheet_slot = nil,   -- the mate the profile window shows
  sheet = nil,
  sheet_asked = -math.huge,
  pullers = {},       -- the puller list's "!squad pull" words, by row
}

local function cap(word) return word and (word:sub(1, 1):upper() .. word:sub(2)) or "" end

-- The server names things as its database does ("onion_staff", "cure_ii"): items by their id from the game's files,
-- spells and abilities by name, else the database name tidied.
local function tidy(name)
  return (name:gsub("_", " "):gsub("^%l", string.upper):gsub(" %l", string.upper))
end

local function item_name(id, fallback)
  local ok, it = pcall(res.item, id)
  return ok and it and it.name and it.name ~= "" and it.name or tidy(fallback)
end

local action_names
local function action_name(name)
  local function key(n) return (n:lower():gsub("[^%w]", "")) end
  if not action_names then
    action_names = {}
    for _, f in ipairs({ res.spell, res.ability }) do
      for id = 1, 1023 do
        local ok, r = pcall(f, id)
        if ok and r and r.name and r.name ~= "" then action_names[key(r.name)] = action_names[key(r.name)] or r.name end
      end
    end
  end
  return action_names[key(name)] or tidy(name)
end

local function index_of(list, value)
  for i, v in ipairs(list) do
    if v == value then return i end
  end
end

local function status(text) ui.update("squad", "status", { text = text }) end

-- Hands a command to XIFO Link; tag names the reply lines it should gather.
local function send(text, tag, delay)
  local ok, err = pcall(mods.call, LINK, "send", text, { tag = tag, delay = delay })
  if not ok then
    status("XIFO Link is not running: switch it on in the Mods list.")
    log.warn(tostring(err))
  end
  return ok
end

local function ask(delay)
  if send("!squad panel", "squadpanel", delay) then st.asked = clock() + (delay or 0) end
end

local function ask_sheet(delay)
  if st.sheet_slot and send("!squad sheet " .. st.sheet_slot, "squadsheet", delay) then
    st.sheet_asked = clock() + (delay or 0)
  end
end

local function pending()
  local ok, n = pcall(mods.call, LINK, "pending")
  return ok and n or 0
end

-- the squad window ------------------------------------------------------------------------------------------------

ui.window("squad", {
  title = "Squad",
  width = 480,
  animate = true,
  help = "Your squad mates on this server. A new role, mode, switch or job counts from a mate's next call: Apply now "
    .. "dismisses the squad and calls it again.",
  elements = {
    { type = "buttons", id = "all", items = { "Call all", "Dismiss all", "Apply now", "Refresh" },
      tip = "Apply now: dismiss everyone and call them all again, so their new settings count." },
    { type = "text", id = "status", text = "Asking the server...", dim = true },
    { type = "list", id = "mates", height = 6, rows = {},
      columns = { { title = "#", width = 24 }, { title = "Name" }, { title = "Jobs", width = 120 },
        { title = "Role", width = 66 }, { title = "Mode", width = 82 }, { title = "", width = 48 } } },
    { type = "separator" },
    { type = "text", id = "sel", text = "No squad mate picked." },
    { type = "buttons", id = "mate", items = { "Call", "Dismiss", "Profile" } },
    { type = "choice", id = "role", label = "Role", items = ROLE_LABELS, at = 1,
      tip = "What the mate does first: Auto follows its job." },
    { type = "choice", id = "mode", label = "Mode", items = MODE_LABELS, at = 1,
      tip = "Aggressive spends MP and TP freely; Conserve holds them back." },
    { type = "buttons", id = "sw", label = "Allowed actions (click to switch)", items = { "", "", "", "" },
      tip = "What kinds of action the mate may use at all, whatever its role says." },
    { type = "dropdown", id = "job", label = "Main job", items = P.JOBS, at = 1,
      tip = "A job your character has not unlocked is refused by the server." },
    { type = "dropdown", id = "sub", label = "Support job", items = { "None" }, at = 1 },
    { type = "section", id = "campsec", label = "Camp", open = false },
    { type = "text", id = "camp", text = "No camp set." },
    { type = "buttons", id = "campbtn", items = { "Camp here", "Start", "Pause", "Lift" },
      tip = "Camp here: the squad holds this spot and fights what comes near. Start: the puller starts pulling." },
    { type = "dropdown", id = "puller", label = "Puller", items = { "Me" }, at = 1 },
  },
})

local open_sheet -- the profile window, below

local function mate() return st.panel and st.panel.mates[st.sel] end

local function jobs_text(m)
  local t = ("%s %d"):format(m.job, m.level)
  if m.sub then t = t .. ("/%s %d"):format(m.sub, m.sub_level) end
  return t
end

local function job_items(m, none)
  local items = none and { "None" } or {}
  for _, job in ipairs(P.JOBS) do
    local lv = m and m.levels[job]
    items[#items + 1] = lv and ("%s %d"):format(job, lv) or job
  end
  return items
end

local function show_mate()
  local m = mate()
  if not m then
    ui.update("squad", "sel", { text = "No squad mate picked." })
    return
  end
  ui.update("squad", "sel", { text = ("#%d %s: %s, %s"):format(m.slot, m.name, jobs_text(m),
    m.out and "out now" or "away") })
  ui.update("squad", "role", { at = index_of(P.ROLES, m.role) or 1 })
  ui.update("squad", "mode", { at = index_of(P.MODES, m.mode) or 1 })
  local sw = {}
  for i, label in ipairs(SW_LABELS) do
    sw[i] = label .. (P.has_switch(m.switches, i) and ": on" or ": off")
  end
  ui.update("squad", "sw", { items = sw })
  ui.update("squad", "job", { items = job_items(m), at = index_of(P.JOBS, m.job) or 1 })
  ui.update("squad", "sub", { items = job_items(m, true), at = m.sub and (index_of(P.JOBS, m.sub) or 0) + 1 or 1 })
end

local function show_camp()
  local p = st.panel
  local c = p.camp
  local items, words, at = { "Me" }, { "me" }, 1
  for _, m in ipairs(p.mates) do
    items[#items + 1] = m.name
    words[#words + 1] = tostring(m.slot)
    if c.on and c.puller == m.slot then at = #items end
  end
  for _, name in ipairs(p.bots) do
    items[#items + 1] = name .. " (bot)"
    words[#words + 1] = name
    if p.bot_puller == name then at = #items end
  end
  st.pullers = words
  ui.update("squad", "puller", { items = items, at = at })
  local text = "No camp set."
  if c.on then
    text = ("Holding camp, %s pulls; %d pull%s so far%s."):format(items[at] == "Me" and "you" or items[at],
      c.pulls, c.pulls == 1 and "" or "s", c.paused and "; pulls paused" or "")
  end
  ui.update("squad", "camp", { text = text })
  ui.update("squad", "campbtn", { items = { "Camp here", "Start", c.paused and "Resume" or "Pause", "Lift" } })
end

local function show_panel()
  local p = st.panel
  if not p then return end
  local rows, out = {}, 0
  for _, m in ipairs(p.mates) do
    if m.out then out = out + 1 end
    local note = m.out and "Out" or (m.cooldown > 0 and (m.cooldown .. " s") or "")
    rows[#rows + 1] = { tostring(m.slot), m.name, jobs_text(m), cap(m.role), cap(m.mode), note,
      colour = not m.out and DIM or nil }
  end
  if st.sel > #rows then st.sel = math.max(1, #rows) end
  ui.update("squad", "mates", { rows = rows, at = st.sel })
  if #rows == 0 then
    status("You have no squad mates yet. Make one with: !squad new <name> <job>")
  else
    status(("%d squad mate%s, %d out."):format(#rows, #rows == 1 and "" or "s", out))
  end
  show_mate()
  show_camp()
end

local function change(kind, value)
  local m = mate()
  if not m then return end
  send(("!squad %s %d %s"):format(kind, m.slot, value))
  ask(1)
end

ui.on("squad", function(_, e)
  local m = mate()
  if e.kind == "press" and e.id == "all" then
    if e.index == 1 then
      send("!squad call all")
      ask(4)
    elseif e.index == 2 then
      send("!squad dismiss")
      ask(1.5)
    elseif e.index == 3 then
      send("!squad dismiss")
      send("!squad call all", nil, 2.5)
      ask(6)
    else
      ask()
    end
  elseif e.kind == "select" and e.id == "mates" then
    st.sel = e.index
    show_mate()
  elseif e.kind == "press" and e.id == "mate" and m then
    if e.index == 1 then
      send("!squad call " .. m.slot)
      ask(4)
    elseif e.index == 2 then
      send("!squad dismiss " .. m.slot)
      ask(1.5)
    else
      open_sheet(m.slot)
    end
  elseif e.kind == "set" and e.id == "role" and m then
    m.role = P.ROLES[e.index]
    change("role", m.role)
    show_panel()
  elseif e.kind == "set" and e.id == "mode" and m then
    m.mode = P.MODES[e.index]
    change("mode", m.mode)
    show_panel()
  elseif e.kind == "press" and e.id == "sw" and m then
    local bit = 1 << (e.index - 1)
    local on = not P.has_switch(m.switches, e.index)
    m.switches = on and (m.switches | bit) or (m.switches & ~bit)
    send(("!squad sw %d %s %s"):format(m.slot, P.SWITCHES[e.index], on and "on" or "off"))
    show_mate()
  elseif e.kind == "set" and e.id == "job" and m then
    change("job", P.JOBS[e.index])
  elseif e.kind == "set" and e.id == "sub" and m then
    change("sub", e.index == 1 and "none" or P.JOBS[e.index - 1])
  elseif e.kind == "press" and e.id == "campbtn" and st.panel then
    local paused = st.panel.camp.paused
    send(({ "!squad camp", "!squad camp start", paused and "!squad camp resume" or "!squad camp pause",
      "!squad camp off" })[e.index])
    ask(1.5)
  elseif e.kind == "set" and e.id == "puller" and st.pullers[e.index] then
    send("!squad pull " .. st.pullers[e.index])
    ask(1.5)
  end
end)

-- the profile window ----------------------------------------------------------------------------------------------

ui.window("squad_sheet", {
  title = "Squad Mate",
  width = 400,
  animate = true,
  elements = {
    { type = "text", id = "hdr", text = "" },
    { type = "text", id = "sub", text = "", dim = true },
    { type = "gauge", id = "hp", label = "HP", kind = "hp" },
    { type = "gauge", id = "mp", label = "MP", kind = "mp" },
    { type = "gauge", id = "tp", label = "TP", kind = "tp" },
    { type = "gauge", id = "exp", label = "EXP", kind = "experience" },
    { type = "text", id = "note", text = "", dim = true },
    { type = "tabs", id = "tabs", items = { "Gear", "Stats", "Jobs", "Actions" } },
    { type = "list", id = "gear", page = 1, height = 10, rows = {},
      columns = { { title = "Slot", width = 80 }, { title = "Item" } } },
    { type = "list", id = "stats", page = 2, height = 10, rows = {},
      columns = { { title = "" }, { title = "", width = 90, align = "right" } } },
    { type = "list", id = "jobs", page = 3, height = 10, rows = {},
      columns = { { title = "Job" }, { title = "Level", width = 70, align = "right" } } },
    { type = "list", id = "acts", page = 4, height = 10, rows = {},
      columns = { { title = "" }, { title = "", width = 90, align = "right" } } },
    { type = "buttons", id = "sbtn", items = { "Refresh", "Close" } },
  },
})

local function vital(id, v, label)
  if v and v.max > 0 then
    ui.update("squad_sheet", id, { value = v.now / v.max, text = ("%d / %d"):format(v.now, v.max) })
  else
    ui.update("squad_sheet", id, { value = 0, text = label or "-" })
  end
end

local function show_sheet()
  local s = st.sheet
  if not s or not s.name then return end
  local t = ("%s: %s %d"):format(s.name, s.job, s.level)
  if s.sub then t = t .. (" / %s %d"):format(s.sub, s.sub_level) end
  ui.update("squad_sheet", "hdr", { text = t })
  ui.update("squad_sheet", "sub", { text = ("%s, role %s, mode %s"):format(s.race or "?", s.role, s.mode) })
  local away = next(s.vitals) == nil and "not called" or nil
  vital("hp", s.vitals.HP, away)
  vital("mp", s.vitals.MP, away)
  vital("tp", s.vitals.TP, away)
  if s.exp and s.exp.need > 0 then
    ui.update("squad_sheet", "exp", { value = s.exp.now / s.exp.need, text = ("%d / %d"):format(s.exp.now, s.exp.need) })
  else
    ui.update("squad_sheet", "exp", { value = s.exp and 1 or 0, text = s.exp and "Level cap" or "-" })
  end
  ui.update("squad_sheet", "note", { text = table.concat(s.notes, " ") })

  local gear = {}
  for _, g in ipairs(s.gear) do
    gear[#gear + 1] = { cap(g.slot_name), item_name(g.id, g.item), icon = g.id > 0 and { kind = "item", id = g.id } or nil }
  end
  for _, g in ipairs(s.look) do
    gear[#gear + 1] = { cap(g.slot_name) .. " (look)", item_name(g.id, g.item), icon = g.id > 0 and { kind = "item", id = g.id } or nil,
      colour = DIM }
  end
  if #gear == 0 then gear[1] = { "", "Nothing worn." } end
  ui.update("squad_sheet", "gear", { rows = gear })

  local stats = {}
  for _, x in ipairs(s.stats) do stats[#stats + 1] = { x.name, tostring(x.value) } end
  for _, x in ipairs(s.combat) do stats[#stats + 1] = { x.name, tostring(x.value) } end
  for _, x in ipairs(s.haste) do stats[#stats + 1] = { "Haste (" .. x.kind .. ")", ("%g%%"):format(x.value / 100) } end
  if #stats == 0 then stats[1] = { "Call this mate to see the numbers.", "" } end
  ui.update("squad_sheet", "stats", { rows = stats })

  local jobs = {}
  for _, j in ipairs(s.jobs) do
    jobs[#jobs + 1] = { j.job, tostring(j.level), colour = j.job ~= s.job and j.job ~= s.sub and DIM or nil }
  end
  ui.update("squad_sheet", "jobs", { rows = jobs })

  local acts = {}
  for _, a in ipairs(s.spells) do acts[#acts + 1] = { action_name(a.name), a.mp .. " MP" } end
  for _, a in ipairs(s.abilities) do acts[#acts + 1] = { action_name(a.name), "Lv. " .. a.level } end
  if #acts == 0 then acts[1] = { "Call this mate to see its spells and abilities.", "" } end
  ui.update("squad_sheet", "acts", { rows = acts })
end

open_sheet = function(slot)
  if st.sheet_slot ~= slot then
    st.sheet = nil
    ui.update("squad_sheet", "hdr", { text = "Asking the server..." })
    ui.update("squad_sheet", "sub", { text = "" })
  end
  st.sheet_slot = slot
  ui.open("squad_sheet")
  ask_sheet()
end

ui.on("squad_sheet", function(_, e)
  if e.kind == "press" and e.id == "sbtn" then
    if e.index == 1 then ask_sheet() else ui.close("squad_sheet") end
  end
end)

-- the server's answers, through XIFO Link ------------------------------------------------------------------------

events.on("xifo_link_reply", function(tag, lines, complete)
  if tag == "squadpanel" then
    if #lines == 0 and not complete then
      status("No answer from the server: this needs an XI: Forever Offline server.")
      return
    end
    st.panel = P.panel(lines)
    show_panel()
  elseif tag == "squadsheet" then
    if #lines == 0 then
      ui.update("squad_sheet", "hdr", { text = "No answer from the server." })
      return
    end
    st.sheet = P.sheet(lines)
    show_sheet()
  end
end)

-- Open windows are asked again now and then: the panel every 15 seconds, a called mate's sheet every 6.
timer.every(1, function()
  local now = clock()
  if ui.is_open("squad") and now - st.asked > 15 and pending() == 0 then ask() end
  if ui.is_open("squad_sheet") and st.sheet_slot and pending() == 0 then
    local live = st.sheet and next(st.sheet.vitals) ~= nil
    if now - st.sheet_asked > (live and 6 or 30) then ask_sheet() end
  end
end)

commands.register("squad", function(what, slot)
  what = what and what:lower()
  if what == "refresh" then
    ui.open("squad")
    ask()
  elseif what == "sheet" or what == "profile" then
    local n = tonumber(slot) or st.sheet_slot or (mate() and mate().slot)
    if n then open_sheet(n) else print("XIFO Squad: which mate? //squad sheet <slot>") end
  elseif ui.is_open("squad") then
    ui.close("squad")
  else
    ui.open("squad")
    ask()
  end
end)
