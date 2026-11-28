-- Retail Macros: finds the game installs on this computer (or a folder picked), lists a character's macro books, and
-- copies the chosen ones into this client's macro palettes or into Hotbars' library. //retailmacros opens it.

local mcr = require("src.mcr")
mcr.init(function(b) return text.from_shift_jis(b) end)

local WIN = "retailmacros"
local PICK = "Choose a Folder..."
local FIRST = "(pick one)"
local JOBS = { "WAR", "MNK", "WHM", "BLM", "RDM", "THF", "PLD", "DRK", "BST", "BRD", "RNG", "SAM", "NIN", "DRG", "SMN",
  "BLU", "COR", "PUP", "DNC", "SCH", "GEO", "RUN" }
local NAMES = { Warrior = "WAR", Monk = "MNK", ["White Mage"] = "WHM", ["Black Mage"] = "BLM", ["Red Mage"] = "RDM",
  Thief = "THF", Paladin = "PLD", ["Dark Knight"] = "DRK", Beastmaster = "BST", Bard = "BRD", Ranger = "RNG",
  Samurai = "SAM", Ninja = "NIN", Dragoon = "DRG", Summoner = "SMN", ["Blue Mage"] = "BLU", Corsair = "COR",
  Puppetmaster = "PUP", Dancer = "DNC", Scholar = "SCH", Geomancer = "GEO", ["Rune Fencer"] = "RUN" }
local SHARED = "Shared"

-- The installs offered, the one in use, its characters (USER/<content id> folders) and the chosen one's books.
local S = { installs = {}, install = nil, own = false, chars = {}, char = nil, books = {}, at = 1, defined = false }

local function sep(p) return p:find("\\", 1, true) and not p:find("/", 1, true) and "\\" or "/" end
local function join(a, b) return a:gsub("[/\\]+$", "") .. sep(a) .. b end
local function say(msg) chat.add("Retail Macros: " .. msg) end

local function status(msg) if S.defined then ui.update(WIN, "status", { text = msg }) end end

local function read(dir, name) return fs.read_outside(join(dir, name)) end

-- A job the book's title names ("WAR", "Ninja", "nin tank"), else nil.
local function job_of(title)
  local t = " " .. title:upper():gsub("[^%w]", " ") .. " "
  for _, j in ipairs(JOBS) do
    if t:find(" " .. j .. " ", 1, true) then return j end
  end
  for name, j in pairs(NAMES) do
    if title:lower():find(name:lower(), 1, true) then return j end
  end
  return nil
end

local function main_job()
  local ok, p = pcall(game.player)
  local id = ok and p and p.main_job
  local ok2, name = pcall(res.job, id)
  return ok2 and type(name) == "string" and name:upper() or nil
end

-- The folders under `dir` that hold macro files.
local function char_dirs(dir)
  local out = {}
  for _, n in ipairs(fs.list_outside(dir)) do
    if n:sub(-1) == "/" then
      local d = join(dir, n:sub(1, -2))
      if fs.exists_outside(join(d, "mcr.ttl")) or fs.exists_outside(join(d, "mcr.dat")) then
        out[#out + 1] = { name = n:sub(1, -2), path = d }
      end
    end
  end
  return out
end

-- The books of a character folder: { title, n (macros), picked, job }.
local function load_books(dir)
  local titles = mcr.titles(read(dir, "mcr.ttl"), read(dir, "mcr_2.ttl"))
  local main = main_job()
  local out = {}
  for b = 1, mcr.BOOKS do
    local n = 0
    for s = 1, mcr.SETS do
      for _, m in ipairs(mcr.set(read(dir, mcr.file(b, s))) or {}) do
        if mcr.used(m) then n = n + 1 end
      end
    end
    out[b] = { title = titles[b], n = n, picked = false, job = job_of(titles[b]) or main or SHARED }
  end
  return out
end

local function job_index(job)
  for i, j in ipairs(JOBS) do
    if j == job then return i end
  end
  return #JOBS + 1
end

local function draw_books()
  local rows = {}
  for b, bk in ipairs(S.books) do
    rows[b] = { cells = { bk.picked and "Yes" or "", ("%d"):format(b), bk.title, bk.n > 0 and tostring(bk.n) or "-",
      bk.job }, colour = bk.n == 0 and { 150, 150, 150, 255 } or nil }
  end
  ui.update(WIN, "books", { rows = rows, at = S.at })
  local bk = S.books[S.at]
  if bk then ui.update(WIN, "job", { at = job_index(bk.job) }) end
  local n = 0
  for _, x in ipairs(S.books) do if x.picked then n = n + 1 end end
  ui.update(WIN, "act", { items = { n == #S.books and "Clear All" or "Pick All", "Copy to Macro Palettes",
    "Send to Hotbars", "Close" } })
end

local function pick_char(k)
  S.char = S.chars[k]
  S.books, S.at = S.char and load_books(S.char.path) or {}, 1
  draw_books()
  if S.char then
    local used = 0
    for _, b in ipairs(S.books) do if b.n > 0 then used = used + 1 end end
    status(("%s: %d of 40 books hold macros. Click a book to pick it."):format(S.char.name, used))
  end
end

-- A character folder's label: its content id and the first book titles named.
local function char_label(c)
  local titles = mcr.titles(read(c.path, "mcr.ttl"), read(c.path, "mcr_2.ttl"))
  local named = {}
  for b, t in ipairs(titles) do
    if t ~= ("Book%02d"):format(b) and #named < 4 then named[#named + 1] = t end
  end
  return #named > 0 and ("%s (%s)"):format(c.name, table.concat(named, ", ")) or c.name
end

-- The install (or a folder in it) the player let us read: its characters.
local function use_folder(path, own)
  S.install, S.own = path, own
  local user = nil
  if fs.exists_outside(join(path, "mcr.ttl")) or fs.exists_outside(join(path, "mcr.dat")) then
    S.chars = { { name = path:match("([^/\\]+)[/\\]*$") or path, path = path } }
  else
    for _, u in ipairs({ "USER", "user" }) do
      if not user and fs.exists_outside(join(path, u)) then user = join(path, u) end
    end
    S.chars = char_dirs(user or path)
  end
  if #S.chars == 0 then
    S.books = {}
    draw_books()
    ui.update(WIN, "char", { items = { "(none)" }, at = 1 })
    status("No macro books in " .. path .. ". Pick the FINAL FANTASY XI folder (it holds USER).")
    return
  end
  local items, at = {}, 1
  local ok, p = pcall(game.player)
  local mine = ok and p and p.id and ("%x"):format(p.id) or nil
  for k, c in ipairs(S.chars) do
    items[k] = char_label(c)
    if c.name:lower() == mine then at = k end
  end
  ui.update(WIN, "char", { items = items, at = at })
  pick_char(at)
end

-- `k` counts the dropdown's values: 1 is the "(pick one)" line, then the installs, then PICK.
local function choose_install(k)
  if k <= 1 then return end
  local i = S.installs[k - 1]
  if not i then
    fs.pick_folder("Choose the FINAL FANTASY XI folder", function(path)
      if path then use_folder(path, false) else status("No folder picked.") end
    end)
    return
  end
  fs.open_folder(i.path, "Retail Macros reads the macro books in its USER folder; it changes nothing there.",
    function(ok)
      if ok then use_folder(i.path, i.own) else status("Not allowed to read " .. i.path .. ".") end
    end)
end

local function picked_books()
  local out = {}
  for b, bk in ipairs(S.books) do
    if bk.picked then out[#out + 1] = b end
  end
  return out
end

local function copy_to_palettes()
  local books = picked_books()
  if #books == 0 then return status("Pick the books to copy first.") end
  if S.own then
    return status("That install is this client's own: its macro books are already the ones you play with.")
  end
  ui.prompt(("Copy %d book(s) over this character's books of the same numbers? The books replaced are kept in a "
    .. "backup."):format(#books), { "Copy", "Cancel" }, function(k)
    if k ~= 1 then return end
    actions.macro_import(S.char.path, books)
    status(("Copied %d book(s): the chat log says where the old ones went."):format(#books))
  end)
end

local function hotbars_loaded()
  for _, m in ipairs(mods.list()) do
    if m.id == "hotbar" then return true end
  end
  return false
end

-- The picked books' macros, by Hotbars job: { [job] = { entries } }, and the /lua lines left out.
local function entries_by_job()
  local by, lua_lines = {}, 0
  for _, b in ipairs(picked_books()) do
    local bk = S.books[b]
    local list = by[bk.job] or {}
    by[bk.job] = list
    for s = 1, mcr.SETS do
      for k, m in ipairs(mcr.set(read(S.char.path, mcr.file(b, s))) or {}) do
        local lines = {}
        for _, l in ipairs(m.lines) do
          if mcr.lua_line(l) then lua_lines = lua_lines + 1 else lines[#lines + 1] = l end
        end
        if #lines > 0 then
          local name = m.title ~= "" and m.title or ("%s%d"):format(k <= 10 and "Ctrl+" or "Alt+", (k - 1) % 10 + 1)
          list[#list + 1] = { name = name, lines = lines }
        end
      end
    end
  end
  return by, lua_lines
end

local function send_to_hotbars()
  if #picked_books() == 0 then return status("Pick the books to send first.") end
  if not hotbars_loaded() then return status("Hotbars is not loaded: turn it on in the Mods page first.") end
  local by, lua_lines = entries_by_job()
  local added, skipped, parts = 0, 0, {}
  local main = main_job() or "WAR"
  for job, list in pairs(by) do
    local shared = job == SHARED
    local ok, a, s, why = pcall(mods.call, "hotbar", "import_macros", shared and main or job, list, shared)
    if not ok or why then
      return status("Hotbars could not take them: " .. tostring(ok and why or a))
    end
    added, skipped = added + (a or 0), skipped + (s or 0)
    parts[#parts + 1] = ("%s %d"):format(job, a or 0)
  end
  table.sort(parts)
  local msg = ("%d macro(s) added to Hotbars' library (%s); %d already there."):format(added, table.concat(parts, ", "),
    skipped)
  if lua_lines > 0 then msg = msg .. (" %d /lua line(s) left out: those are yours to write."):format(lua_lines) end
  status(msg)
  say(msg)
end

local function define()
  ui.window(WIN, {
    title = "Retail Macros",
    width = 640,
    help = "Copies macro books from a game install (retail's) into this client's macro palettes or Hotbars.",
    elements = {
      { type = "text", id = "status", text = "Pick an install, or choose its folder." },
      { type = "dropdown", id = "install", label = "Install", items = { FIRST, PICK }, at = 1,
        tip = "The game installs found on this computer, or a folder you choose." },
      { type = "dropdown", id = "char", label = "Character", items = { "(none)" }, at = 1,
        tip = "A character's folder (its content id) and the book titles it has." },
      { type = "list", id = "books", height = 12,
        columns = { { title = "Pick", width = 40 }, { title = "#", width = 30 }, { title = "Book" },
          { title = "Macros", width = 60 }, { title = "Hotbars", width = 70 } },
        tip = "Click a book to pick it, or to leave it out." },
      { type = "choice", id = "job", label = "Hotbars Job for This Book", items = (function()
        local t = { table.unpack(JOBS) }
        t[#t + 1] = SHARED
        return t
      end)(), at = 1, tip = "Whose library its macros go to in Hotbars. Shared: every job's." },
      { type = "buttons", id = "act", items = { "Pick All", "Copy to Macro Palettes", "Send to Hotbars", "Close" } },
    },
  })
  ui.on(WIN, function(_, e)
    if e.kind == "set" and e.id == "install" then
      choose_install(e.index)
    elseif e.kind == "set" and e.id == "char" then
      if S.chars[e.index] then pick_char(e.index) end
    elseif e.kind == "select" and e.id == "books" then
      local bk = S.books[e.index]
      S.at = e.index
      if bk then bk.picked = not bk.picked end
      draw_books()
    elseif e.kind == "set" and e.id == "job" then
      local bk = S.books[S.at]
      if bk then
        bk.job = JOBS[e.index] or SHARED
        draw_books()
      end
    elseif e.kind == "press" and e.id == "act" then
      if e.index == 1 then
        local all = #picked_books() == #S.books
        for _, bk in ipairs(S.books) do bk.picked = not all end
        draw_books()
      elseif e.index == 2 then
        copy_to_palettes()
      elseif e.index == 3 then
        send_to_hotbars()
      else
        ui.close(WIN)
      end
    end
  end)
  S.defined = true
end

local function open()
  if not fs.game_installs or not actions.macro_import then
    return say("this client is too old for it: update vana-client.")
  end
  if not S.defined then define() end
  local ok, found = pcall(fs.game_installs)
  S.installs = {}
  local items = { FIRST }
  for _, i in ipairs(ok and found or {}) do
    S.installs[#S.installs + 1] = i
    items[#items + 1] = i.path .. (i.own and " (this client's)" or "")
  end
  items[#items + 1] = PICK
  ui.update(WIN, "install", { items = items, at = 1 })
  ui.open(WIN)
  if not ok then
    status(tostring(found))
  elseif #S.installs == 0 then
    status("No game install found: choose its folder.")
  else
    status(("%d install(s) found: pick one (you'll be asked to let this mod read it)."):format(#S.installs))
  end
end

commands.register("retailmacros", function() open() end)
