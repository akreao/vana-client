-- XIFO Gambits: edit your squad mates' Rule Sets on an XI: Forever Offline server. A Rule Set is an ordered list of
-- rules for one main job ("Party member, HP below 50% -> best Cure"); a mate that uses one follows it instead of its
-- built-in job rules. The server keeps the sets and checks every save; this window loads, edits, saves, checks and
-- assigns them with the server's "!gsq" and "!squad ruleset" commands, through the XIFO Link mod.

local G = require("src.gsq")
local Share = require("src.share")

local LINK = "xifo-link"
local DIM, BAD = { 160, 160, 160 }, { 235, 120, 110 }
local W = "gambits"

local st = {
  version = nil,   -- the server's editor version (hello)
  max_rules = 32,
  mates = {},      -- from "mates"
  mate = 1,        -- the picked mate's row
  sets = {},       -- the picked mate's job's sets ("list")
  set_row = 1,
  cat = nil,       -- the picked mate's catalog ("catm")
  draft = nil,     -- the open set: { num, job, name, flags, rev, rules }
  rule = 1,        -- the picked rule
  dirty = false,
  -- the lists behind the dropdowns, by row: names as the server writes them
  targets = {}, conds = {}, actions = {},
}

local function index_of(list, v)
  for i, x in ipairs(list) do
    if x == v then return i end
  end
end

local function status(text, bad)
  ui.update(W, "status", { text = text, colour = bad and BAD or false })
end

-- requests -------------------------------------------------------------------------------------------------------

local pending = {}          -- reply tag -> handler(lines, complete)
local next_id = (os.time() % 9000) * 100

local function say(text, delay)
  local ok, err = pcall(mods.call, LINK, "send", text, { delay = delay })
  if not ok then status("XIFO Link is not running: switch it on in the Mods list.", true) end
  return ok, err
end

-- Sends one !gsq request and calls handler(lines, complete) with its reply lines ("kind|fields...").
local function request(build, handler, ends, wait)
  next_id = next_id + 1
  local id = next_id
  local tag = "gsq" .. id
  pending[tag] = handler
  local ok = pcall(mods.call, LINK, "send", build(id),
    { tag = tag, mark = "GSQ|" .. id .. "|", ends = ends or { "end", "err" }, wait = wait })
  if not ok then
    pending[tag] = nil
    status("XIFO Link is not running: switch it on in the Mods list.", true)
  end
  return id
end

events.on("xifo_link_reply", function(tag, lines, complete)
  local h = pending[tag]
  if h then
    pending[tag] = nil
    h(lines, complete)
  end
end)

local function failed(err, what)
  status(("%s: %s"):format(what, err and err.text or "no answer from the server"), true)
end

-- the window -----------------------------------------------------------------------------------------------------

ui.window(W, {
  title = "Gambits",
  width = 600,
  animate = true,
  help = "Rule Sets for your squad mates. Rules are tried from the top; the first whose conditions hold is done. A set "
    .. "counts from the mate's next call: Apply now calls the squad again.",
  elements = {
    { type = "dropdown", id = "mate", label = "Squad mate", items = { "Asking the server..." }, at = 1 },
    { type = "text", id = "status", text = "", dim = true },
    { type = "tabs", id = "tabs", items = { "Rule Sets", "Weapon skills", "Party roles" }, at = 1 },
    { page = 1, type = "list", id = "sets", height = 5, rows = {},
      columns = { { title = "#", width = 30 }, { title = "Rule Set" }, { title = "Rules", width = 50, align = "right" },
        { title = "", width = 110 } } },
    { page = 1, type = "buttons", id = "setbtn", items = { "Open", "New", "Copy", "Delete" } },
    { page = 1, type = "buttons", id = "use", items = { "Use this set", "Use job rules", "Apply now" },
      tip = "Use this set: the mate follows the picked Rule Set from its next call. Use job rules: back to its built-in "
        .. "rules. Apply now: dismiss and call the squad, so the change counts now." },
    { page = 1, type = "separator" },
    { page = 1, type = "text", id = "editing", text = "No Rule Set open." },
    { page = 1, type = "input", id = "name", label = "Name", text = "", max = 24 },
    { page = 1, type = "list", id = "rules", height = 8, rows = {},
      columns = { { title = "#", width = 26 }, { title = "", width = 34 }, { title = "Target", width = 130 },
        { title = "When" }, { title = "Do", width = 150 } } },
    { page = 1, type = "buttons", id = "ruleops", items = { "Add", "Edit", "Copy", "Delete", "Up", "Down" },
      tip = "Edit (or a click on a rule) opens the rule in its own window." },
    { page = 1, type = "buttons", id = "save", items = { "Save", "Check", "Undo changes" },
      tip = "Check: the server tries the saved set at the called mate's level and lists the rules it would skip." },
    { page = 1, type = "buttons", id = "share", label = "Share codes (the same GS1 codes as XIFO's Gambits add-on)",
      items = { "Copy code", "Import code" },
      tip = "Copy code: the open Rule Set as a code on the clipboard. Import code: reads the box below, or the clipboard "
        .. "when the box is empty, and opens the set as a new one; nothing is saved until you Save." },
    { page = 1, type = "input", id = "code", label = "Share code", text = "", max = 1024 },
    { page = 2, type = "text", id = "ws_info", text = "Pick a squad mate." },
    { page = 2, type = "choice", id = "ws_ctl", label = "Weapon skills", items = { "Squad default", "Custom" }, at = 1,
      tip = "Squad default: the squad picks skillchain roles and weapon skills itself. Custom: the settings below." },
    { page = 2, type = "choice", id = "ws_mode", label = "Skillchains", items = { "Open", "Close", "Open + Close",
      "Independent" }, at = 3, tip = "Open: starts skillchains. Close: only closes them. Independent: at its TP." },
    { page = 2, type = "choice", id = "ws_tp", label = "At TP", items = { "1000", "1500", "2000", "3000" }, at = 1,
      tip = "Not used by Close." },
    { page = 2, type = "dropdown", id = "ws_ws", label = "Weapon skill", items = { "Best" }, at = 1,
      tip = "Best: the best closer when a skillchain is open, else its strongest. The list is the called mate's." },
    { page = 2, type = "buttons", id = "ws_btn", items = { "Save", "Put on now", "Refresh" },
      tip = "Save keeps the setting for the mate's main job (from its next call). Put on now: the called mate takes it at once." },
    { page = 3, type = "text", id = "pr_info",
      text = "Who the Rule Sets' Tank, Melee, Mage and Ranged targets mean. Auto follows the main job." },
    { page = 3, type = "list", id = "pr_list", height = 8, rows = {},
      columns = { { title = "" }, { title = "Job", width = 70 }, { title = "Role", width = 150 },
        { title = "Counts as", width = 130 } } },
    { page = 3, type = "choice", id = "pr_role", label = "Role", items = { "Auto", "Tank", "Melee", "Mage", "Ranged" },
      at = 1, tip = "Not the Squad window's Role: this one only decides who a Rule Set's role targets mean." },
    { page = 3, type = "buttons", id = "pr_btn", items = { "Refresh" } },
  },
})

-- The picked rule, in its own window.
local R = "gambits_rule"
ui.window(R, {
  title = "Rule",
  width = 420,
  animate = true,
  elements = {
    { type = "text", id = "which", text = "" },
    { type = "choice", id = "on", label = "Rule", items = { "On", "Off" }, at = 1 },
    { type = "dropdown", id = "target", label = "Target", items = { "-" }, at = 1 },
    { type = "buttons", id = "rf", label = "Other roles", items = { "-" },
      tip = "For the targets Tank, Melee, Mage and Ranged: a click turns a role between counting too (+), left out (-) and "
        .. "not set." },
    { type = "dropdown", id = "cond", label = "When", items = { "-" }, at = 1 },
    { type = "input", id = "arg", label = "Value", text = "", max = 24,
      tip = "A number for HP, MP, TP and the like; a status effect's name (Haste, Protect...) for Has / Missing effect." },
    { type = "dropdown", id = "cond2", label = "And when", items = { "Nothing more" }, at = 1 },
    { type = "input", id = "arg2", label = "Value", text = "", max = 24 },
    { type = "dropdown", id = "kind", label = "Do", items = { "-" }, at = 1 },
    { type = "dropdown", id = "action", label = "What", items = { "-" }, at = 1,
      tip = "What the mate can use. Entries marked with a later level start working at that level." },
    { type = "input", id = "level", label = "Only from level (blank: always)", text = "", max = 2 },
    { type = "text", id = "problem", text = "", colour = { 235, 120, 110 } },
    { type = "buttons", id = "done", items = { "Done" } },
  },
})

local function mate() return st.mates[st.mate] end
local function rule() return st.draft and st.draft.rules[st.rule] end

local function target_text(r)
  local t = G.target_label(r.target)
  if r.include and #r.include > 0 then
    local x = {}
    for i, v in ipairs(r.include) do x[i] = G.target_label(v) end
    t = t .. " + " .. table.concat(x, ", ")
  end
  if r.exclude and #r.exclude > 0 then
    local x = {}
    for i, v in ipairs(r.exclude) do x[i] = G.target_label(v) end
    t = t .. ", not " .. table.concat(x, ", ")
  end
  return t
end

local function show_sets()
  local m = mate()
  local rows = {}
  for _, s in ipairs(st.sets) do
    local note = {}
    if m and m.set == s.num then note[#note + 1] = "In use" end
    if not G.editable(s.flags) then note[#note + 1] = s.flags:find("m", 1, true) and "Copy only" or "Read-only" end
    rows[#rows + 1] = { tostring(s.num), s.name, tostring(s.count), table.concat(note, ", "),
      colour = (m and m.set == s.num) and { 150, 220, 150 } or nil }
  end
  if #rows == 0 then rows[1] = { "", "No Rule Sets for this job yet: New makes one.", "", "", colour = DIM } end
  if st.set_row > #st.sets then st.set_row = math.max(1, #st.sets) end
  ui.update(W, "sets", { rows = rows, at = st.set_row })
end

-- The dropdowns' lists for the picked rule (from the mate's catalog; a value it does not offer is kept, marked).
-- `label(v)` names a value the list leaves out; `keep_first`: the list's first row ("Nothing more") stays first.
local function fill_dropdown(id, names, labels, current, label, keep_first)
  local at = index_of(names, current)
  if not at and current and current ~= "" then
    local pos = keep_first and 2 or 1
    table.insert(names, pos, current)
    table.insert(labels, pos, label(current) .. " (not usable now)")
    at = pos
  end
  if #labels == 0 then names, labels = { "" }, { "-" } end
  ui.update(R, id, { items = labels, at = at or 1 })
  return names
end

local function action_lists(kind)
  local cat = st.cat or {}
  local names, labels = {}, {}
  local function add(n, l) names[#names + 1] = n; labels[#labels + 1] = l end
  if kind == "f" then
    for _, n in ipairs(cat.f or {}) do add(n, G.tidy(n)) end
  elseif kind == "s" or kind == "j" then
    for _, e in ipairs(cat[kind] or {}) do
      add(e.name, ("%s (%sLv %d)"):format(G.tidy(e.name), e.sub and "sub " or "", e.level))
    end
    if kind == "s" then
      for _, e in ipairs(cat.u or {}) do add(e.name, ("%s (Lv %d, not learned yet)"):format(G.tidy(e.name), e.level)) end
    end
  end
  return names, labels
end

local function show_rule()
  local r = rule()
  local cat = st.cat or {}
  if not r or r.raw then
    if r then ui.close(R) end
    return
  end
  ui.update(R, "which", { text = ("Rule %d of %s"):format(st.rule, st.draft.name or "") })
  ui.update(R, "on", { at = r.on and 1 or 2 })
  local tn, tl = {}, {}
  for _, t in ipairs(cat.t or {}) do tn[#tn + 1] = t; tl[#tl + 1] = G.target_label(t) end
  st.targets = fill_dropdown("target", tn, tl, r.target, G.target_label)
  local role_target = index_of(G.ROLE_TARGETS, r.target) and (st.version or 0) >= 9
  local rf = {}
  if role_target then
    for _, role in ipairs(G.ROLE_TARGETS) do
      if role ~= r.target then
        local mark = index_of(r.include or {}, role) and " +" or (index_of(r.exclude or {}, role) and " -" or "")
        rf[#rf + 1] = G.target_label(role) .. mark
      end
    end
  else
    rf[1] = "Only for the targets Tank, Melee, Mage and Ranged"
  end
  ui.update(R, "rf", { items = rf })
  local cn, cl = {}, {}
  for _, c in ipairs(cat.c or {}) do cn[#cn + 1] = c.name; cl[#cl + 1] = G.cond_label(c.name) end
  st.conds = fill_dropdown("cond", cn, cl, r.cond, G.cond_label)
  local cn2, cl2 = { "" }, { "Nothing more" }
  for _, c in ipairs(cat.c or {}) do cn2[#cn2 + 1] = c.name; cl2[#cl2 + 1] = G.cond_label(c.name) end
  st.conds2 = fill_dropdown("cond2", cn2, cl2, r.cond2 or "", G.cond_label, true)
  ui.update(R, "arg", { text = r.arg or "" })
  ui.update(R, "arg2", { text = r.cond2 and (r.arg2 or "") or "" })
  local kl = {}
  for i, k in ipairs(G.KINDS) do kl[i] = G.KIND_LABELS[k] end
  ui.update(R, "kind", { items = kl, at = index_of(G.KINDS, r.kind) or 1 })
  local an, al = action_lists(r.kind)
  if G.NAMED[r.kind] then
    st.actions = fill_dropdown("action", an, al, r.name or "", G.tidy)
  else
    st.actions = {}
    ui.update(R, "action", { items = { "-" }, at = 1 })
  end
  ui.update(R, "level", { text = r.level or "" })
  local p = G.rule_problem(r, st.cat)
  ui.update(R, "problem", { text = p and ("This rule " .. p .. ".") or "" })
end

local function show_rules()
  local d = st.draft
  if not d then
    ui.close(R)
    ui.update(W, "editing", { text = "No Rule Set open." })
    ui.update(W, "rules", { rows = {} })
    ui.update(W, "name", { text = "" })
    return
  end
  local what = d.num and d.num > 0 and ("Rule Set %d"):format(d.num) or "New Rule Set"
  local note = G.editable(d.flags or "o") and "" or " (not yours: Save makes a copy)"
  ui.update(W, "editing", { text = ("%s for %s%s%s"):format(what, d.job, note, st.dirty and ", not saved" or "") })
  ui.update(W, "name", { text = d.name or "" })
  local rows = {}
  for i, r in ipairs(d.rules) do
    local when, what2 = G.describe(r)
    local problem = G.rule_problem(r, st.cat)
    rows[i] = { tostring(i), r.raw and "?" or (r.on and "On" or "Off"), r.raw and "" or target_text(r), when, what2,
      colour = r.raw and BAD or (problem and BAD) or (not r.on and DIM) or nil }
  end
  if st.rule > #rows then st.rule = math.max(1, #rows) end
  ui.update(W, "rules", { rows = rows, at = st.rule })
  show_rule()
end

local function changed()
  st.dirty = true
  show_rules()
end

-- loading --------------------------------------------------------------------------------------------------------

local load_sets

local function load_catalog()
  local m = mate()
  if not m then return end
  st.cat = nil
  local slot = m.slot
  request(function(id) return G.catm(id, slot, st.version) end, function(lines)
    local cat, err = G.parse_catalog(lines)
    if mate() and mate().slot == slot then
      if cat then st.cat = cat else failed(err, "The mate's choices") end
      show_rules()
    end
  end, nil, 20)
end

load_sets = function(then_open)
  local m = mate()
  if not m then return end
  local job = m.job
  request(function(id) return G.list(id, job) end, function(lines)
    local sets, err = G.parse_list(lines)
    if not sets then return failed(err, "The Rule Sets") end
    st.sets = sets
    if then_open then st.set_row = index_of((function()
      local n = {}
      for i, s in ipairs(sets) do n[i] = s.num end
      return n
    end)(), then_open) or st.set_row end
    show_sets()
    status(("%s, %s %d%s. %s"):format(m.name, m.job, m.level, m.sub and ("/%s %d"):format(m.sub, m.sub_level) or "",
      m.set > 0 and ("Uses Rule Set %d."):format(m.set) or "Uses its job rules."))
  end)
end

local function show_mates()
  local items = {}
  for i, m in ipairs(st.mates) do
    items[i] = ("%s  %s %d%s%s"):format(m.name, m.job, m.level, m.sub and ("/%s %d"):format(m.sub, m.sub_level) or "",
      m.out and "  (out)" or "")
  end
  if #items == 0 then items[1] = "No squad mates" end
  ui.update(W, "mate", { items = items, at = st.mate })
end

local function load_mates(keep)
  request(G.mates, function(lines)
    local mates, err = G.parse_mates(lines)
    if not mates then return failed(err, "The squad") end
    local slot = mate() and mate().slot
    st.mates = mates
    st.mate = (keep and slot and index_of((function()
      local n = {}
      for i, m in ipairs(mates) do n[i] = m.slot end
      return n
    end)(), slot)) or 1
    show_mates()
    if #mates == 0 then return status("You have no squad mates yet (!squad new <name> <job>).") end
    show_sets()
    if not keep then
      load_sets()
      load_catalog()
    end
  end)
end

local function start()
  status("Asking the server...")
  request(G.hello, function(lines, complete)
    local h, err = G.parse_hello(lines)
    if not h then
      return status(complete and ("The server refused: " .. (err and err.text or "?"))
        or "No answer: this needs an XI: Forever Offline server with the Gambits editor.", true)
    end
    if h.version < 4 then return status("This server's Gambits editor is too old (version 4 or later needed).", true) end
    st.version, st.max_rules = h.version, h.max_rules
    load_mates()
  end, { "hello", "err" })
end

-- weapon skills and party roles -----------------------------------------------------------------------------------

st.tab = 1
st.ws = nil        -- the picked mate's weapon skill setting as the server has it ("wsg")
st.ws_edit = nil   -- { control, mode, tp, ws } as the player is setting it
st.ws_list = nil   -- the called mate's weapon skills ("wsl")
st.ws_ids = {}     -- the weapon skill dropdown's ids by row (0 = Best)
st.roles = nil     -- "prl"
st.pr_row = 1

local function ws_name(id)
  for _, w in ipairs(st.ws_list or {}) do
    if w.id == id then return G.tidy(w.name) end
  end
  return ("Weapon skill %d"):format(id)
end

local function show_ws()
  local w, m, e = st.ws, mate(), st.ws_edit
  if not w or not e or not m then return end
  local notes = {}
  notes[#notes + 1] = ({ custom = "Uses the Custom setting.", default = "Uses the squad default.",
    legacy = "No Gambits setting: the !squad ws setting applies." })[w.source] or ""
  if not w.engine then notes[#notes + 1] = "This map server cannot run Custom settings." end
  if w.usable == false then notes[#notes + 1] = m.name .. " cannot use the chosen weapon skill now and uses Best." end
  if w.applied == false then notes[#notes + 1] = "The called mate still runs the old setting: Put on now." end
  if not w.out then notes[#notes + 1] = "Call " .. m.name .. " to pick from its weapon skills." end
  ui.update(W, "ws_info", { text = ("%s as %s. %s"):format(m.name, w.job, table.concat(notes, " ")) })
  ui.update(W, "ws_ctl", { at = e.control + 1 })
  ui.update(W, "ws_mode", { at = e.mode })
  ui.update(W, "ws_tp", { at = index_of(G.WS_TPS, e.tp) or 1 })
  local ids, items = { 0 }, { "Best" }
  for _, x in ipairs(st.ws_list or {}) do ids[#ids + 1] = x.id; items[#items + 1] = G.tidy(x.name) end
  if not index_of(ids, e.ws) then
    ids[#ids + 1] = e.ws
    items[#items + 1] = ws_name(e.ws) .. (w.out and " (not usable now)" or "")
  end
  st.ws_ids = ids
  ui.update(W, "ws_ws", { items = items, at = index_of(ids, e.ws) or 1 })
end

local function took_ws(lines)
  local w, err = G.parse_ws(lines)
  if not w then return failed(err, "Weapon skills") end
  st.ws = w
  st.ws_edit = { control = w.control, mode = w.mode, tp = w.tp, ws = w.ws }
  show_ws()
end

local function load_ws()
  local m = mate()
  if not m then return end
  if (st.version or 0) < 5 then
    return ui.update(W, "ws_info", { text = "This server's Gambits editor has no weapon skill settings (version 5)." })
  end
  local slot = m.slot
  st.ws_list = nil
  request(function(id) return G.wsg(id, slot) end, function(lines)
    if not mate() or mate().slot ~= slot then return end
    took_ws(lines)
    if st.ws and st.ws.out then
      request(function(id) return G.wsl(id, slot) end, function(l2)
        local list = G.parse_wsl(l2)
        if list and mate() and mate().slot == slot then
          st.ws_list = list
          show_ws()
        end
      end)
    end
  end)
end

local function role_text(set, auto)
  if set == 0 then return "Auto (" .. (auto ~= "" and G.letters(auto) or "none") .. ")" end
  return G.ROLE_LABELS[set] or "?"
end

local function show_roles()
  local r = st.roles
  if not r then return end
  local rows = {}
  for _, x in ipairs(r.own) do
    rows[#rows + 1] = { x.who == 0 and (x.name .. " (you)") or x.name, ("%s %d"):format(x.job, x.level),
      role_text(x.set, x.auto),
      x.live and (x.live == 0 and G.letters(x.auto) or G.ROLE_LABELS[x.live]) or "not called",
      colour = (x.who ~= 0 and not x.out) and DIM or nil }
  end
  for _, x in ipairs(r.others) do
    rows[#rows + 1] = { x.name .. (x.kind == "t" and " (trust)" or ""), x.job, "their own", G.letters(x.targets),
      colour = DIM }
  end
  if st.pr_row > #rows then st.pr_row = 1 end
  ui.update(W, "pr_list", { rows = rows, at = st.pr_row })
  local own = r.own[st.pr_row]
  ui.update(W, "pr_role", { at = own and own.set + 1 or 1 })
end

local function load_roles()
  if (st.version or 0) < 7 then
    return ui.update(W, "pr_info", { text = "This server's Gambits editor has no party roles (version 7)." })
  end
  request(G.prl, function(lines)
    local r, err = G.parse_roles(lines)
    if not r then return failed(err, "Party roles") end
    st.roles = r
    show_roles()
  end)
end

local function load_page()
  if st.tab == 2 then load_ws() elseif st.tab == 3 then load_roles() end
end

-- opening, editing, saving ----------------------------------------------------------------------------------------

local function copy_rules(rules)
  local out = {}
  for i, r in ipairs(rules) do
    local c = {}
    for k, v in pairs(r) do c[k] = v end
    c.id = nil -- the server numbers a copy's rules again
    out[i] = c
  end
  return out
end

local function open_set(num)
  request(function(id) return G.get(id, num) end, function(lines)
    local set, err = G.parse_set(lines)
    if not set then return failed(err, "Opening the Rule Set") end
    st.draft, st.rule, st.dirty = set, 1, false
    show_rules()
  end, nil, 15)
end

-- Asks before throwing away unsaved changes.
local function unless_dirty(go)
  if not st.dirty then return go() end
  ui.prompt("Throw away the changes to the open Rule Set?", { "Throw away", "Keep editing" }, function(i)
    if i == 1 then
      st.dirty = false
      go()
    end
  end)
end

local function new_rule()
  local cat = st.cat or {}
  local cond = (G.find_cond(cat, "HPP_LT") and "HPP_LT") or (cat.c and cat.c[1] and cat.c[1].name) or "ALWAYS"
  local c = G.find_cond(cat, cond)
  return { on = true, target = index_of(cat.t or {}, "SELF") and "SELF" or (cat.t and cat.t[1]) or "SELF", cond = cond,
    arg = c and c.kind == "r" and tostring(math.min(c.hi, math.max(c.lo, 50))) or "", kind = "f",
    name = cat.f and cat.f[1] or "", include = {}, exclude = {} }
end

local function save()
  local d, m = st.draft, mate()
  if not d or not m then return end
  for i, r in ipairs(d.rules) do
    local p = G.rule_problem(r, st.cat)
    if p then return status(("Rule %d %s."):format(i, p), true) end
  end
  local set = d
  if not G.editable(d.flags or "o") then
    set = { num = 0, job = d.job, name = d.name, rules = d.rules }
  end
  next_id = next_id + 1
  local token = next_id
  local lines, why = G.save_lines(set, token)
  if not lines then return status(why, true) end
  local tag = "save" .. token
  pending[tag] = function(replies, complete)
    local saved, err = G.parse_saved(replies)
    if not saved then
      return status(complete and ("Not saved: " .. (err and err.text or "?"))
        or "Not saved: the server did not answer every part. Try Save again.", true)
    end
    d.num, d.rev, d.flags = saved.num, saved.rev, "o"
    st.dirty = false
    status(("Saved as Rule Set %d."):format(saved.num))
    show_rules()
    load_sets(saved.num)
  end
  for _, line in ipairs(lines) do
    local ok = pcall(mods.call, LINK, "send", line,
      { tag = tag, mark = "GSQ|" .. token .. "|", ends = { "ok", "err", "aborted" }, wait = 15 })
    if not ok then
      pending[tag] = nil
      return status("XIFO Link is not running: switch it on in the Mods list.", true)
    end
  end
  status(("Saving (%d line%s)..."):format(#lines, #lines == 1 and "" or "s"))
end

local function check()
  local d, m = st.draft, mate()
  if not d or not d.num or d.num == 0 then return status("Save the Rule Set first: Check tries the saved one.", true) end
  if not m.out then return status("Call " .. m.name .. " first: Check uses the called mate's level.", true) end
  request(function(id) return G.chk(id, m.slot, d.num) end, function(lines)
    local res, err = G.parse_check(lines)
    if not res then return failed(err, "Check") end
    if #res.problems == 0 then
      return status(("Every rule works for %s at level %d."):format(m.name, res.level or m.level))
    end
    local parts = {}
    for _, p in ipairs(res.problems) do parts[#parts + 1] = ("rule %s: %s"):format(p.rule, p.why) end
    status(("At level %d, %s skips %s."):format(res.level or m.level, m.name, table.concat(parts, "; ")), true)
  end)
end

local function arg_text(cond, text)
  local c = G.find_cond(st.cat, cond)
  text = (text or ""):gsub("^%s+", ""):gsub("%s+$", "")
  if c and c.kind ~= "r" then text = text:upper():gsub("%s+", "_") end
  if c and c.kind == "n" then text = "" end
  return text
end

ui.on(W, function(_, e)
  local m, d, r = mate(), st.draft, rule()
  if e.kind == "close" then return ui.close(R) end
  if e.kind == "set" and e.id == "mate" then
    if not st.mates[e.index] then return end
    unless_dirty(function()
      st.mate, st.draft, st.set_row = e.index, nil, 1
      show_rules()
      load_sets()
      load_catalog()
      load_page()
    end)
  elseif e.kind == "set" and e.id == "tabs" then
    st.tab = e.index
    load_page()
  elseif e.kind == "set" and st.ws_edit and (e.id == "ws_ctl" or e.id == "ws_mode" or e.id == "ws_tp" or e.id == "ws_ws") then
    local x = st.ws_edit
    if e.id == "ws_ctl" then x.control = e.index - 1
    elseif e.id == "ws_mode" then x.mode = e.index
    elseif e.id == "ws_tp" then x.tp = G.WS_TPS[e.index] or x.tp
    else x.ws = st.ws_ids[e.index] or x.ws end
  elseif e.kind == "press" and e.id == "ws_btn" and m then
    local slot = m.slot
    if e.index == 1 and st.ws_edit then
      local x = st.ws_edit
      request(function(id) return G.wss(id, slot, x) end, function(lines)
        took_ws(lines)
        if st.ws then status("Weapon skill setting saved for " .. m.name .. ".") end
      end)
    elseif e.index == 2 then
      request(function(id) return G.wsa(id, slot) end, took_ws)
    else
      load_ws()
    end
  elseif e.kind == "select" and e.id == "pr_list" then
    st.pr_row = e.index
    show_roles()
  elseif e.kind == "set" and e.id == "pr_role" then
    local own = st.roles and st.roles.own[st.pr_row]
    if not own then
      status("Only you and your squad mates' roles are set here; others set their own.", true)
      return show_roles()
    end
    request(function(id) return G.prs(id, own.who, e.index - 1) end, function(lines)
      local r, err = G.parse_roles(lines)
      if not r then return failed(err, "Party role") end
      load_roles()
    end)
  elseif e.kind == "press" and e.id == "pr_btn" then
    load_roles()
  elseif e.kind == "select" and e.id == "sets" then
    st.set_row = e.index
  elseif e.kind == "press" and e.id == "setbtn" and m then
    local s = st.sets[st.set_row]
    if e.index == 1 and s then
      unless_dirty(function() open_set(s.num) end)
    elseif e.index == 2 then
      unless_dirty(function()
        st.draft = { num = 0, job = m.job, name = "New " .. m.job, flags = "o", rules = { new_rule() } }
        st.rule, st.dirty = 1, true
        show_rules()
      end)
    elseif e.index == 3 and d then
      st.draft = { num = 0, job = d.job, name = ("Copy of " .. (d.name or "")):sub(1, 24), flags = "o",
        rules = copy_rules(d.rules) }
      st.dirty = true
      show_rules()
    elseif e.index == 4 and s then
      if not G.editable(s.flags) then return status("Only your own Rule Sets can be deleted.", true) end
      ui.prompt(("Delete Rule Set %d, %s?"):format(s.num, s.name), { "Delete", "Keep" }, function(i)
        if i ~= 1 then return end
        request(function(id) return G.del(id, s.num, s.rev) end, function(lines)
          local done, err = G.parse_saved(lines)
          if not done then return failed(err, "Delete") end
          if d and d.num == s.num then st.draft, st.dirty = nil, false end
          show_rules()
          load_sets()
        end, { "ok", "err" })
      end)
    end
  elseif e.kind == "press" and e.id == "use" and m then
    local s = st.sets[st.set_row]
    if e.index == 1 then
      if not s then return status("Pick a Rule Set in the list first.", true) end
      say(("!squad ruleset %d %d"):format(m.slot, s.num))
    elseif e.index == 2 then
      say(("!squad ruleset %d off"):format(m.slot))
    else
      say("!squad reapply")
    end
    timer.after(2, function() load_mates(true) end)
  elseif e.kind == "press" and e.id == "share" then
    if e.index == 1 then
      if not d then return status("Open a Rule Set first.", true) end
      local code, why = Share.encode(d)
      if not code then return status("No code: " .. why, true) end
      client.copy(code)
      ui.update(W, "code", { text = #code <= 1024 and code or "" })
      status(("The code of %s (%d rules) is on the clipboard."):format(d.name or "the set", #d.rules))
    else
      local function import(text)
        local set, why = Share.decode(text)
        if not set then return status("Not imported: " .. why, true) end
        local mjob = m and m.job
        st.draft = { num = 0, job = set.job, name = set.name, flags = "o", rules = set.rules }
        st.rule, st.dirty = 1, true
        show_rules()
        status(("Imported %s (%s, %d rules) as a new Rule Set; Save keeps it.%s"):format(set.name, set.job, #set.rules,
          mjob and mjob ~= set.job and (" It is for " .. set.job .. ", not " .. m.name .. "'s " .. mjob .. ".") or ""))
      end
      local typed = ui.get(W, "code")
      typed = typed and typed.text or ""
      if typed:match("%S") then
        unless_dirty(function() import(typed) end)
      else
        local ok = pcall(client.paste, function(text) unless_dirty(function() import(text) end) end)
        if not ok then status("Paste the code into the box, then Import code.", true) end
      end
    end
  elseif e.kind == "change" and e.id == "name" and d then
    d.name = e.text
    changed()
  elseif e.kind == "select" and e.id == "rules" then
    st.rule = e.index
    if rule() and not rule().raw then ui.open(R) end
    show_rule()
  elseif e.kind == "press" and e.id == "ruleops" and d then
    if e.index == 1 or e.index == 3 then
      if #d.rules >= st.max_rules then return status(("A Rule Set holds at most %d rules."):format(st.max_rules), true) end
      local add = e.index == 1 and new_rule() or (r and copy_rules({ r })[1])
      if not add then return end
      table.insert(d.rules, math.min(st.rule + 1, #d.rules + 1), add)
      st.rule = math.min(st.rule + 1, #d.rules)
      if e.index == 1 then ui.open(R) end
    elseif e.index == 2 then
      if r and not r.raw then ui.open(R) end
      return show_rule()
    elseif e.index == 4 and r then
      table.remove(d.rules, st.rule)
      if #d.rules == 0 then ui.close(R) end
    elseif e.index == 5 and st.rule > 1 then
      d.rules[st.rule], d.rules[st.rule - 1] = d.rules[st.rule - 1], d.rules[st.rule]
      st.rule = st.rule - 1
    elseif e.index == 6 and st.rule < #d.rules then
      d.rules[st.rule], d.rules[st.rule + 1] = d.rules[st.rule + 1], d.rules[st.rule]
      st.rule = st.rule + 1
    else
      return
    end
    changed()
  elseif e.kind == "press" and e.id == "save" then
    if e.index == 1 then
      save()
    elseif e.index == 2 then
      check()
    elseif d then
      if d.num and d.num > 0 then
        st.dirty = false
        open_set(d.num)
      else
        st.draft, st.dirty = nil, false
        show_rules()
      end
    end
  end
end)

ui.on(R, function(_, e)
  local r = rule()
  if e.kind == "press" and e.id == "done" then return ui.close(R) end
  if not r or r.raw then return end
  if e.kind == "set" and e.id == "on" then
    r.on = e.index == 1
  elseif e.kind == "set" and e.id == "target" and st.targets[e.index] then
    r.target = st.targets[e.index]
    local function without(list)
      local out = {}
      for _, v in ipairs(list or {}) do
        if v ~= r.target and index_of(G.ROLE_TARGETS, r.target) then out[#out + 1] = v end
      end
      return out
    end
    r.include, r.exclude = without(r.include), without(r.exclude)
  elseif e.kind == "press" and e.id == "rf" then
    if not index_of(G.ROLE_TARGETS, r.target) or (st.version or 0) < 9 then return end
    local others = {}
    for _, v in ipairs(G.ROLE_TARGETS) do if v ~= r.target then others[#others + 1] = v end end
    local role = others[e.index]
    if not role then return end
    local inc, exc = {}, {}
    for _, v in ipairs(r.include or {}) do if v ~= role then inc[#inc + 1] = v end end
    for _, v in ipairs(r.exclude or {}) do if v ~= role then exc[#exc + 1] = v end end
    -- not set -> counts too -> left out -> not set
    if not index_of(r.include or {}, role) and not index_of(r.exclude or {}, role) then
      inc[#inc + 1] = role
    elseif index_of(r.include or {}, role) then
      exc[#exc + 1] = role
    end
    r.include, r.exclude = inc, exc
  elseif e.kind == "set" and e.id == "cond" and st.conds[e.index] then
    r.cond = st.conds[e.index]
    r.arg = arg_text(r.cond, r.arg)
  elseif e.kind == "set" and e.id == "cond2" and st.conds2[e.index] then
    r.cond2 = st.conds2[e.index] ~= "" and st.conds2[e.index] or nil
    r.arg2 = r.cond2 and arg_text(r.cond2, r.arg2) or ""
  elseif e.kind == "change" and e.id == "arg" then
    r.arg = arg_text(r.cond, e.text)
  elseif e.kind == "change" and e.id == "arg2" and r.cond2 then
    r.arg2 = arg_text(r.cond2, e.text)
  elseif e.kind == "set" and e.id == "kind" and G.KINDS[e.index] then
    r.kind = G.KINDS[e.index]
    local names = action_lists(r.kind)
    r.name = G.NAMED[r.kind] and (names[1] or "") or nil
  elseif e.kind == "set" and e.id == "action" and st.actions[e.index] then
    r.name = st.actions[e.index]
  elseif e.kind == "change" and e.id == "level" then
    local lv = tonumber(e.text)
    r.level = lv and lv >= 1 and lv <= 99 and tostring(math.floor(lv)) or nil
  else
    return
  end
  changed()
end)

commands.register("gambits", function()
  if ui.is_open(W) then
    ui.close(W)
    ui.close(R)
  else
    ui.open(W)
    if not st.version then start() else load_mates(true) end
  end
end)
