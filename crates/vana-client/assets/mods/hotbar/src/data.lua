-- Macro storage and resolution by job.
--
-- File: macros-<World>.json in the mod's data folder (macros.json until the world is known; a new world's file
-- starts from it). Keyed by main job:
--
--   {
--     "RDM": {
--       "library": { "<id>": <entry> },
--       "sets": { "WHM": { "<bar>": { "<slot>": <macro> } }, "-": {...} },   -- RDM/WHM's bars, RDM's with no sub
--       "pads": { "WHM": { "lt": { "up": <macro>, ... }, "rt": {...}, "lr": {...} } },   -- the crossbars, the same
--       "main": {...}, "sub": { "WHM": {...} }, "pad": {...}   -- before sets by sub job: read once to start them
--     }
--   }
--
-- Every main / sub job pair has its own bars and crossbars. A pair's first use starts its set from the old layout
-- (the main set with the sub set over it, the job's crossbars), less what another sub job's actions were (M.keep).
--
-- Slot value: a macro { name, label?, lines }, a library reference { ref = "<id>" }, or false in an old sub set
-- (blanked the main set's slot). Library entry: { name, label?, action = { kind, name, target, line? }?, lines } or a group
-- { name, label?, group = { <entry>, ... } }. Bar and slot keys are strings (JSON object keys).
--
-- Shared entries (ids "s<n>") live in sticky.json, for every world and job: { next_id, library = { "s1": <entry> } }.

local M = {}

M.MAX_BARS = 3
M.MAX_SLOTS = 12
M.PAD_SIDES = { "lt", "rt", "lr" }
M.PAD_BUTTONS = { "up", "right", "down", "left", "y", "b", "a", "x", "lb", "rb", "l3", "r3" }

local LEGACY = "macros.json"
local FILE = LEGACY
local STICKY_FILE = "sticky.json"

local deps = nil
local db = {}
local sticky = { library = {} }

local function is_table(v) return type(v) == "table" end
local function key(n) return tostring(n) end

-- ---------------------------------------------------------------------------
-- Text
-- ---------------------------------------------------------------------------

local function trim(s)
  return (tostring(s or ""):gsub("^%s+", ""):gsub("%s+$", ""))
end
M.trim = trim

-- "a | b | c" -> { "a", "b", "c" }, empty pieces dropped.
-- Lines from typed text: one a line (`sep` "\n", a text area), else between | marks (a one-line field).
function M.split_lines(text, sep)
  local out = {}
  for piece in tostring(text or ""):gmatch(sep == "\n" and "[^\n]+" or "[^|]+") do
    local line = trim(piece)
    if line ~= "" then out[#out + 1] = line end
  end
  return out
end

function M.join_lines(lines, sep)
  return table.concat(lines or {}, sep == "\n" and "\n" or " | ")
end

-- A slot's text: up to 7 characters, or two lines of 7.
local function wrap(text)
  text = trim(text)
  if #text <= 7 then return text end
  local a, b = text:match("^(%S+)%s+(.+)$")
  if a then return a:sub(1, 7) .. "\n" .. b:sub(1, 7) end
  return text:sub(1, 7)
end

function M.display_label(macro)
  if not is_table(macro) then return "" end
  local label = trim(macro.label)
  return wrap(label ~= "" and label or tostring(macro.name or ""))
end

-- A macro's name, else its first line.
function M.name_of(macro)
  return trim(macro.name) ~= "" and macro.name or ((macro.lines or {})[1] or "")
end

-- A macro's lines, or a group's members numbered, one per line.
function M.tip_text(macro)
  local lines = macro.lines or {}
  if macro.group then
    lines = {}
    for n, m in ipairs(macro.group) do lines[n] = ("%d. %s"):format(n, m.lines[1] or m.name) end
  end
  return table.concat(lines, "\n")
end

-- ---------------------------------------------------------------------------
-- Load and save
-- ---------------------------------------------------------------------------

-- d = { read, write, decode, encode, log?, lock? }
function M.init(d)
  deps = d
  return M.load()
end

-- A file's table, {} when missing or blank; nil and why when unreadable (a copy kept as <file>.bad).
local function read_json(file, raw)
  if not raw or raw:match("^%s*$") then return {} end
  local ok, parsed = pcall(deps.decode, raw)
  if ok and is_table(parsed) then return parsed end
  pcall(deps.write, file .. ".bad", raw)
  return nil, tostring(parsed)
end

-- A change to the world's file that could not be written: it lives in memory only until the next save, so a sticky
-- change keeps memory rather than reading the file afresh.
local unsaved = false

-- Returns true, or false, why and the file that failed.
function M.load()
  unsaved = false
  local raw = deps.read(FILE)
  if not raw and FILE ~= LEGACY then raw = deps.read(LEGACY) end
  local parsed, why = read_json(FILE, raw)
  db = parsed or {}
  local st, swhy = read_json(STICKY_FILE, deps.read(STICKY_FILE))
  sticky = st or {}
  if not is_table(sticky.library) then sticky.library = {} end
  if not parsed then return false, why, FILE end
  if not st then return false, swhy, STICKY_FILE end
  return true
end

function M.file() return FILE end

-- Switches to world `name`'s file (nil or "": macros.json). Returns false when unchanged, else true and load()'s.
function M.set_world(name)
  local file = name and name ~= "" and ("macros-" .. (name:gsub("[^%w%-_]", "_")) .. ".json") or LEGACY
  if file == FILE then return false end
  FILE = file
  return true, M.load()
end

local function write(file, t)
  local ok, err = pcall(function() deps.write(file, deps.encode(t)) end)
  if not ok and deps.log then deps.log("could not save " .. file .. ": " .. tostring(err)) end
  return ok, err
end

function M.save()
  local ok, err = write(FILE, db)
  unsaved = not ok
  return ok, err
end

local function save_sticky() return write(STICKY_FILE, sticky) end

local function deep_copy(v)
  if not is_table(v) then return v end
  local out = {}
  for k, x in pairs(v) do out[k] = deep_copy(x) end
  return out
end

local function last_picks() return storage and storage.character and storage.character.hb_last end

local function same(a, b)
  if not is_table(a) or not is_table(b) then return a == b end
  for k, v in pairs(a) do
    if not same(v, b[k]) then return false end
  end
  for k in pairs(b) do
    if a[k] == nil then return false end
  end
  return true
end

-- Runs f with sticky.json locked and both files read afresh (every client on this install shares sticky.json, and the
-- characters on one world share its file, so a change starts from the others' ids, entries and slots); f changes
-- memory only, then sticky.json is saved, and the world's file when f changed it (a sticky-only edit leaves it to the
-- others). When either cannot be written, all f changed (both tables and the groups' last picks) is put back, a file
-- already written is written back, and it returns nil and why. Without fs.lock (older clients) f runs unlocked.
local held = false
local function with_sticky(f)
  if held then return f() end
  local lock = deps.lock or function(_, g) return g() end
  return lock(STICKY_FILE .. ".lock", function()
    held = true
    local ok, r, why = pcall(function()
      local st = read_json(STICKY_FILE, deps.read(STICKY_FILE))
      if st then
        sticky = st
        if not is_table(sticky.library) then sticky.library = {} end
      end
      local raw = not unsaved and deps.read(FILE)
      local wd = raw and read_json(FILE, raw)
      if wd then db = wd end
      local was = { db = deep_copy(db), sticky = deep_copy(sticky), last = deep_copy(last_picks()), unsaved = unsaved }
      local out = f()
      local saved, err = save_sticky()
      local touched = not same(db, was.db)
      if saved and (unsaved or touched) then
        local wrote, werr = M.save()
        -- Only the edits still waiting in memory failed again (f left the world alone): the sticky change stands and
        -- they keep waiting.
        if not wrote and touched then
          saved, err = false, werr
          sticky = was.sticky
          save_sticky()
        end
      end
      if saved then
        if last_picks() then pcall(storage.save) end
        return out
      end
      db, sticky, unsaved = was.db, was.sticky, was.unsaved
      if storage and storage.character then
        storage.character.hb_last = was.last
        pcall(storage.save)
      end
      return nil, tostring(err)
    end)
    held = false
    if not ok then error(r, 0) end
    return r, why
  end)
end

-- ---------------------------------------------------------------------------
-- Tables
-- ---------------------------------------------------------------------------

local function child(t, k, create)
  if not is_table(t) then return nil end
  if is_table(t[k]) then return t[k] end
  if not create then return nil end
  t[k] = {}
  return t[k]
end

-- db[main][part]: "library", "main", "sub" or "pad".
local function job_part(main, part, create)
  if not main then return nil end
  return child(child(db, main, create), part, create)
end

-- The old main set (sub nil) or an old sub job's set (before sets by sub job; read only).
local function find_set(main, sub, create)
  if not sub then return job_part(main, "main", create) end
  return child(job_part(main, "sub", create), sub, create)
end

-- keep(main, sub, macro): whether the pair can use a slot's macro when its set starts from the old layout (another
-- sub job's action: false); keep_ready(): whether keep can judge everything yet (until then a pair's set is shown,
-- not saved). Set by init.lua; none keeps every slot.
M.keep = nil
M.keep_ready = nil

local function raw_slot(set, row, slot)
  local r = child(set, key(row), false)
  return r and r[key(slot)]
end

local function put_slot(set, row, slot, v)
  child(set, key(row), true)[key(slot)] = v
end

-- Removes a slot, and its row when that empties.
local function drop_slot(set, row, slot)
  local r = child(set, key(row), false)
  if not r then return end
  r[key(slot)] = nil
  if next(r) == nil then set[key(row)] = nil end
end

-- ---------------------------------------------------------------------------
-- Library
-- ---------------------------------------------------------------------------

local COMMAND = { ma = "/ma", trust = "/ma", ja = "/ja", ws = "/ws", pet = "/pet", item = "/item", equip = "/equip", cmd = "" }

-- The line an action builds: /ma "Cure" <t>, /equip ring1 "Warp Ring", /attack <t>; its `line` when set.
function M.command(action)
  if not is_table(action) or not COMMAND[action.kind] or trim(action.name) == "" then return nil end
  if trim(action.line) ~= "" then return trim(action.line) end
  local target = trim(action.target)
  if action.kind == "equip" then
    if target == "" then return nil end
    return ('/equip %s "%s"'):format(target, trim(action.name))
  end
  if action.kind == "cmd" then
    return target ~= "" and ("%s <%s>"):format(trim(action.name), target) or trim(action.name)
  end
  local line = ('%s "%s"'):format(COMMAND[action.kind], trim(action.name))
  if target ~= "" then line = line .. " <" .. target .. ">" end
  return line
end

-- An entry's lines as run: its action's line, then its own (none for a group).
function M.entry_lines(entry)
  local out = {}
  if is_table(entry.group) then return out end
  out[1] = M.command(entry.action)
  for _, l in ipairs(entry.lines or {}) do out[#out + 1] = tostring(l) end
  return out
end

local function member(e)
  return { name = e.name or "", label = e.label, lines = M.entry_lines(e), action = e.action, icon = e.icon }
end

-- One runnable macro per entry table (the runner keys on it); a saved entry is a new table.
local expanded = setmetatable({}, { __mode = "k" })

local function expand(entry, id)
  local m = expanded[entry]
  if m then return m end
  m = member(entry)
  m.ref = id
  if is_table(entry.group) then
    m.group = {}
    for k, e in ipairs(entry.group) do
      if is_table(e) then m.group[k] = member(e) end
    end
  end
  expanded[entry] = m
  return m
end

-- A shared entry's id: shown in every job's library.
function M.is_sticky(id)
  return id ~= nil and tostring(id):match("^s%d+$") ~= nil
end

local function lib_of(main, id, create)
  if M.is_sticky(id) then return sticky.library end
  return job_part(main, "library", create)
end

function M.get_entry(main, id)
  local lib = id ~= nil and lib_of(main, id, false)
  local entry = lib and lib[tostring(id)]
  return is_table(entry) and entry or nil
end

-- A library entry as slots run it, or nil.
function M.macro_of(main, id)
  local entry = M.get_entry(main, id)
  return entry and expand(entry, tostring(id)) or nil
end

-- A slot value as a macro: a reference's entry (nil when gone), else itself.
local function deref(main, v)
  if not is_table(v) then return nil end
  if v.ref ~= nil then return M.macro_of(main, v.ref) end
  return v
end

local function entry_id_order(a, b)
  local na, nb = tonumber(a), tonumber(b)
  if na and nb then return na < nb end
  return tostring(a) < tostring(b)
end

-- The job's entries, oldest first, then the shared ones: { { id, entry, shared? }, ... }.
function M.library(main)
  local out = {}
  local function add(lib, is_sticky)
    local ids = {}
    for id, entry in pairs(lib or {}) do
      if is_table(entry) then ids[#ids + 1] = id end
    end
    table.sort(ids, function(a, b)
      if is_sticky then return entry_id_order(a:sub(2), b:sub(2)) end
      return entry_id_order(a, b)
    end)
    for _, id in ipairs(ids) do out[#out + 1] = { id = id, entry = lib[id], sticky = is_sticky or nil } end
  end
  add(job_part(main, "library", false), false)
  add(sticky.library, true)
  return out
end

-- The icon kinds a macro may show.
local ICON_KINDS = { spell = true, ability = true, item = true, status = true }

local function clean_entry(e)
  local out = { name = trim(e.name) }
  if trim(e.label) ~= "" then out.label = trim(e.label) end
  local ic = e.icon
  local id = is_table(ic) and tonumber(ic.id)
  if id and ICON_KINDS[ic.kind] and id == math.floor(id) then out.icon = { kind = ic.kind, id = math.floor(id) } end
  if is_table(e.group) then
    out.group = {}
    for _, m in ipairs(e.group) do
      if is_table(m) and not is_table(m.group) then
        local c = clean_entry(m)
        if c.action or #c.lines > 0 then out.group[#out.group + 1] = c end
      end
    end
    return out
  end
  out.lines = {}
  for _, l in ipairs(e.lines or {}) do
    l = trim(l)
    if l ~= "" then out.lines[#out.lines + 1] = l end
  end
  local a = e.action
  if is_table(a) and COMMAND[a.kind] and trim(a.name) ~= "" then
    out.action = { kind = a.kind, name = trim(a.name), target = trim(a.target) }
    if trim(a.line) ~= "" then out.action.line = trim(a.line) end
  end
  return out
end

local function new_job_id(main)
  local n = 0
  for k in pairs(job_part(main, "library", true)) do n = math.max(n, tonumber(k) or 0) end
  return tostring(n + 1)
end

-- Shared ids are never reused: another world's file may still name a deleted one.
local function new_sticky_id()
  local n = tonumber(sticky.next_id) or 1
  for k in pairs(sticky.library) do n = math.max(n, (tonumber(tostring(k):sub(2)) or 0) + 1) end
  sticky.next_id = n + 1
  return "s" .. n
end

local function refers(v, id)
  return is_table(v) and v.ref ~= nil and tostring(v.ref) == id
end

-- fn(set, old) for every set of the job: each main / sub pair's bars and crossbars, and the old layout's sets (`old`
-- true: they start pairs not played yet).
local function each_set(main, fn)
  for _, part in ipairs({ "sets", "pads" }) do
    for _, set in pairs(job_part(main, part, false) or {}) do
      if is_table(set) then fn(set, false) end
    end
  end
  for _, part in ipairs({ "main", "pad" }) do
    local set = job_part(main, part, false)
    if set then fn(set, true) end
  end
  for _, set in pairs(job_part(main, "sub", false) or {}) do
    if is_table(set) then fn(set, true) end
  end
end

-- fn(job) for every job in this world's file.
local function each_job(fn)
  for job, t in pairs(db) do
    if is_table(t) then fn(job) end
  end
end

-- Whether two saved entries are the same macro (names, lines, action and group members alike).
local function same(a, b)
  if type(a) ~= type(b) then return false end
  if not is_table(a) then return a == b end
  for k, v in pairs(a) do
    if not same(v, b[k]) then return false end
  end
  for k in pairs(b) do
    if a[k] == nil then return false end
  end
  return true
end

-- The jobs of this world with macros of their own (shared ones aside), by name: { { job, n }, ... }.
function M.jobs_with_macros()
  local out = {}
  each_job(function(job)
    local n = 0
    for _, e in pairs(job_part(job, "library", false) or {}) do
      if is_table(e) then n = n + 1 end
    end
    if n > 0 then out[#out + 1] = { job = job, n = n } end
  end)
  table.sort(out, function(a, b) return a.job < b.job end)
  return out
end

-- The job's own entries (no shared ones), oldest first.
function M.own_library(main)
  local out = {}
  for _, it in ipairs(M.library(main)) do
    if not it.sticky then out[#out + 1] = it end
  end
  return out
end

-- Copies `ids` of `from`'s own library (all of them when nil) into `to`'s, skipping any `to` already has the same.
-- Returns how many were copied and skipped, and why the save failed.
-- Whether `to`'s own library has an entry the same as `entry`.
function M.has_same(to, entry)
  local c = clean_entry(entry)
  for _, have in pairs(job_part(to, "library", false) or {}) do
    if same(clean_entry(have), c) then return true end
  end
  return false
end

function M.copy_entries(from, to, ids)
  local want = nil
  if ids then
    want = {}
    for _, id in ipairs(ids) do want[tostring(id)] = true end
  end
  local copied, skipped = 0, 0
  for _, it in ipairs(M.own_library(from)) do
    if not want or want[tostring(it.id)] then
      local c = clean_entry(it.entry)
      local dup = false
      for _, have in pairs(job_part(to, "library", false) or {}) do
        dup = dup or same(clean_entry(have), c)
      end
      if dup then
        skipped = skipped + 1
      else
        local nid = new_job_id(to)
        job_part(to, "library", true)[nid] = c
        copied = copied + 1
      end
    end
  end
  if copied == 0 then return 0, skipped end
  local ok, err = M.save()
  return copied, skipped, not ok and tostring(err) or nil
end

-- Macros another mod hands over ({ name, lines, icon? } each) into `main`'s library, or the shared one: each unless the
-- same is there already. Returns how many were added and skipped, and why the save failed (then none are kept).
-- A /lua line (a mod's own command) is never taken from another mod: Hotbars would run it as the player's line.
local function lua_line(l) return type(l) == "string" and l:lower():match("^%s*/+lua") ~= nil end

function M.import_entries(main, entries, shared)
  local have = {}
  local dropped = 0
  for _, it in ipairs(M.library(main)) do
    if (it.sticky == true) == (shared == true) then have[#have + 1] = clean_entry(it.entry) end
  end
  local new = {}
  local skipped = 0
  for _, e in ipairs(is_table(entries) and entries or {}) do
    if is_table(e) and is_table(e.lines) then
      local kept = {}
      for _, l in ipairs(e.lines) do
        if lua_line(l) then dropped = dropped + 1 else kept[#kept + 1] = l end
      end
      e = { name = e.name, label = e.label, icon = e.icon, action = e.action, lines = kept }
    end
    local c = is_table(e) and not is_table(e.group) and clean_entry(e) or nil
    if c and #c.lines > 0 then
      local dup = false
      for _, h in ipairs(have) do dup = dup or same(h, c) end
      if dup then
        skipped = skipped + 1
      else
        have[#have + 1] = c
        new[#new + 1] = c
      end
    end
  end
  if #new == 0 then return 0, skipped, nil, dropped end
  if shared then
    local _, why = with_sticky(function()
      for _, c in ipairs(new) do sticky.library[new_sticky_id()] = c end
      return true
    end)
    if why then return 0, skipped, why, dropped end
    return #new, skipped, nil, dropped
  end
  local lib = job_part(main, "library", true)
  local added = {}
  for _, c in ipairs(new) do
    local id = new_job_id(main)
    lib[id] = c
    added[#added + 1] = id
  end
  local ok, err = M.save()
  if not ok then
    for _, id in ipairs(added) do lib[id] = nil end
    return 0, skipped, tostring(err), dropped
  end
  return #new, skipped, nil, dropped
end

local function uses_in(main, id)
  local n = 0
  local paired = next(job_part(main, "sets", false) or {}) ~= nil or next(job_part(main, "pads", false) or {}) ~= nil
  each_set(main, function(set, old)
    if old and paired then return end
    for _, row in pairs(set) do
      if is_table(row) then
        for _, v in pairs(row) do
          if refers(v, id) then n = n + 1 end
        end
      end
    end
  end)
  return n
end

-- How many slots name the entry (a shared one: in every job of this world).
function M.uses(main, id)
  id = tostring(id)
  if not M.is_sticky(id) then return uses_in(main, id) end
  local n = 0
  each_job(function(job) n = n + uses_in(job, id) end)
  return n
end

-- The job's slots naming `old` name `new`.
local function repoint(main, old, new)
  each_set(main, function(set)
    for _, row in pairs(set) do
      if is_table(row) then
        for sk, v in pairs(row) do
          if refers(v, old) then row[sk] = { ref = new } end
        end
      end
    end
  end)
end

local move_last

-- Makes an entry shared (every job and world) or the job's again. Returns its new id, or nil and why when a file
-- cannot be written (nothing moves). Unstuck, each job of this world with a slot naming it gets a copy (`main` always);
-- other worlds' slots naming it go empty.
function M.set_sticky(main, id, on)
  return with_sticky(function()
    id = tostring(id)
    local entry = M.get_entry(main, id)
    if not entry or on == M.is_sticky(id) then return id end
    local nid
    if on then
      nid = new_sticky_id()
      sticky.library[nid] = entry
      job_part(main, "library", false)[id] = nil
      repoint(main, id, nid)
      move_last(main, id, nid)
    else
      local function take(job)
        local jid = new_job_id(job)
        job_part(job, "library", true)[jid] = clean_entry(entry)
        repoint(job, id, jid)
        move_last(job, id, jid)
        return jid
      end
      each_job(function(job)
        if job ~= main and uses_in(job, id) > 0 then take(job) end
      end)
      nid = take(main)
      sticky.library[id] = nil
    end
    return nid
  end)
end

-- Saves an entry under `id` (nil: a new id), then makes it shared or the job's when `want` says so (nil: as it is).
-- Returns its id; when a file cannot be written, also why. A shared entry's edit or a new shared entry is then not made
-- (all is put back) and the id is nil; a job entry's change stays in memory with its id.
function M.save_entry(main, id, entry, want)
  local shared = id == nil and want or id ~= nil and M.is_sticky(tostring(id))
  local function put()
    local nid = id ~= nil and tostring(id) or shared and new_sticky_id() or new_job_id(main)
    lib_of(main, nid, true)[nid] = clean_entry(entry)
    return nid
  end
  local nid, why
  if shared then
    nid, why = with_sticky(put)
    if why then return nil, why end
  else
    nid = put()
    local ok, err = M.save()
    if not ok then why = tostring(err) end
  end
  if why or want == nil or want == M.is_sticky(nid) then return nid, why end
  local moved, mwhy = M.set_sticky(main, nid, want)
  if moved then return moved end
  return nid, mwhy
end

-- A group's last-picked member, per character.
local function last_key(macro)
  local ok, mj = pcall(function() return require("src.jobs").current() end)
  return tostring(ok and mj or "") .. ":" .. tostring(macro.ref)
end

function M.last_member(macro)
  local t = storage and storage.character and storage.character.hb_last
  local k = type(t) == "table" and tonumber(t[last_key(macro)]) or 1
  return math.max(1, math.min(k, #(macro.group or {})))
end

-- A group's last pick follows its entry to a new id (saved with the files, by with_shared).
function move_last(main, old, new)
  local t = storage and storage.character and storage.character.hb_last
  if type(t) ~= "table" then return end
  local ok = tostring(main) .. ":" .. old
  if t[ok] == nil then return end
  t[tostring(main) .. ":" .. new], t[ok] = t[ok], nil
end

function M.set_last_member(macro, k)
  if not (storage and storage.character) then return end
  if type(storage.character.hb_last) ~= "table" then storage.character.hb_last = {} end
  local lk = last_key(macro)
  if storage.character.hb_last[lk] == k then return end
  storage.character.hb_last[lk] = k
  pcall(storage.save)
end

function M.current_member(macro)
  return macro.group and macro.group[M.last_member(macro)] or nil
end

-- Empties the job's slots naming `id`. A sub set's slot over a filled main slot is blanked (false) so the main macro
-- does not show through.
local function clear_refs(main, id)
  local last = storage and storage.character and storage.character.hb_last
  if type(last) == "table" and last[tostring(main) .. ":" .. id] ~= nil then
    last[tostring(main) .. ":" .. id] = nil
    pcall(storage.save)
  end
  local main_set = find_set(main, nil, false)
  each_set(main, function(set, old)
    for rk, row in pairs(set) do
      if is_table(row) then
        for sk, v in pairs(row) do
          if refers(v, id) then
            -- An old sub set blanks the old main set's slot under it; the pairs' sets just empty theirs.
            local under = old and set ~= main_set and raw_slot(main_set, rk, sk)
            if is_table(under) and not refers(under, id) then row[sk] = false else row[sk] = nil end
          end
        end
        if next(row) == nil then set[rk] = nil end
      end
    end
  end)
end

-- Removes an entry and every slot naming it (a shared one: in every job of this world). Returns true, or false and
-- why when a file cannot be written (a shared entry then stays, slots and all).
function M.delete_entry(main, id)
  id = tostring(id)
  if M.is_sticky(id) then
    local done, why = with_sticky(function()
      sticky.library[id] = nil
      each_job(function(job) clear_refs(job, id) end)
      return true
    end)
    return done == true, why
  end
  local lib = lib_of(main, id, false)
  if lib then lib[id] = nil end
  clear_refs(main, id)
  local ok, err = M.save()
  return ok, not ok and tostring(err) or nil
end

-- ---------------------------------------------------------------------------
-- Bars
-- ---------------------------------------------------------------------------

-- The value in one set: a macro, a reference, false (blanked) or nil.
-- ---------------------------------------------------------------------------
-- Sets by main / sub job
-- ---------------------------------------------------------------------------

local function pair_key(sub) return sub or "-" end

-- Drops the slots the pair cannot use (M.keep); rows left empty go.
local function keep_only(main, sub, set)
  if not M.keep then return set end
  for rk, row in pairs(set) do
    if is_table(row) then
      for sk, v in pairs(row) do
        local m = deref(main, v)
        if m and not M.keep(main, sub, m) then row[sk] = nil end
      end
      if next(row) == nil then set[rk] = nil end
    end
  end
  return set
end

-- The old layout as main / sub: the main set with the sub set over it (its blanks blank).
local function old_bars(main, sub)
  local out = {}
  local function lay(set)
    for rk, row in pairs(set or {}) do
      if is_table(row) then
        out[rk] = out[rk] or {}
        for sk, v in pairs(row) do
          if v == false then out[rk][sk] = nil elseif is_table(v) then out[rk][sk] = deep_copy(v) end
        end
        if next(out[rk]) == nil then out[rk] = nil end
      end
    end
  end
  lay(find_set(main, nil, false))
  if sub then lay(find_set(main, sub, false)) end
  return out
end

-- The pair's `part` ("sets": its bars, "pads": its crossbars), started from the old layout on first use.
local function pair_set(main, sub, part, create)
  if not main then return nil end
  local k = pair_key(sub)
  local sets = job_part(main, part, false)
  if sets and is_table(sets[k]) then return sets[k] end
  local old = part == "sets" and old_bars(main, sub) or deep_copy(job_part(main, "pad", false) or {})
  if not create and next(old) == nil then return nil end
  old = keep_only(main, sub, old)
  -- Shown only while keep cannot judge yet: saved once it can (or when a slot changes).
  if not create and M.keep_ready and not M.keep_ready() then return old end
  sets = job_part(main, part, true)
  sets[k] = old
  M.save()
  return sets[k]
end

function M.get_raw(main, sub, bar, slot)
  return raw_slot(pair_set(main, sub, "sets", false), bar, slot)
end

-- A slot's macro as main / sub, or nil (the second value, "main", is kept for older callers).
function M.resolve(main, sub, bar, slot)
  local m = deref(main, raw_slot(pair_set(main, sub, "sets", false), bar, slot))
  if m then return m, "main" end
  return nil
end

-- Every slot as main / sub: eff[bar][slot] = { macro, from = "main" }.
function M.effective(main, sub)
  local eff = {}
  for bk, row in pairs(pair_set(main, sub, "sets", false) or {}) do
    local b = tonumber(bk)
    if b and is_table(row) then
      for sk, v in pairs(row) do
        local s = tonumber(sk)
        local macro = s and is_table(v) and deref(main, v)
        if macro then
          eff[b] = eff[b] or {}
          eff[b][s] = { macro = macro, from = "main" }
        end
      end
    end
  end
  return eff
end

function M.assign(main, sub, bar, slot, id)
  put_slot(pair_set(main, sub, "sets", true), bar, slot, { ref = tostring(id) })
  M.save()
end

-- Puts a macro on a slot: a reference stays one, anything else is copied.
function M.set_macro(main, sub, bar, slot, macro)
  if macro.ref ~= nil then
    M.assign(main, sub, bar, slot, macro.ref)
    return
  end
  local lines = {}
  for _, l in ipairs(macro.lines or {}) do lines[#lines + 1] = tostring(l) end
  local m = { name = trim(macro.name), lines = lines }
  if trim(macro.label) ~= "" then m.label = trim(macro.label) end
  put_slot(pair_set(main, sub, "sets", true), bar, slot, m)
  M.save()
end

function M.clear(main, sub, bar, slot)
  drop_slot(pair_set(main, sub, "sets", true), bar, slot)
  M.save()
end

-- Swaps what two slots show; references stay references.
function M.swap(main, sub, b1, s1, b2, s2)
  local m1 = M.resolve(main, sub, b1, s1)
  local m2 = M.resolve(main, sub, b2, s2)
  local function put(b, s, m)
    if m then M.set_macro(main, sub, b, s, m) else M.clear(main, sub, b, s) end
  end
  put(b1, s1, m2)
  put(b2, s2, m1)
end

-- ---------------------------------------------------------------------------
-- Crossbars
-- ---------------------------------------------------------------------------

function M.pad_get_raw(main, sub, side, button)
  local v = raw_slot(pair_set(main, sub, "pads", false), side, button)
  return is_table(v) and v or nil
end

function M.pad_resolve(main, sub, side, button)
  return deref(main, M.pad_get_raw(main, sub, side, button))
end

local function pad_put(main, sub, side, button, v)
  local set = pair_set(main, sub, "pads", true)
  if v then put_slot(set, side, button, v) else drop_slot(set, side, button) end
end

function M.pad_assign(main, sub, side, button, id)
  pad_put(main, sub, side, button, { ref = tostring(id) })
  M.save()
end

function M.pad_clear(main, sub, side, button)
  pad_put(main, sub, side, button, nil)
  M.save()
end

function M.pad_swap(main, sub, s1, b1, s2, b2)
  local v1, v2 = M.pad_get_raw(main, sub, s1, b1), M.pad_get_raw(main, sub, s2, b2)
  pad_put(main, sub, s1, b1, v2)
  pad_put(main, sub, s2, b2, v1)
  M.save()
end

return M
