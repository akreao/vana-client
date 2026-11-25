-- What is left (the checklist and spell book add-ons): a job's unlearned spells by level, every area's quests and
-- where we stand in them. One window, two tabs; //left prints summaries too.

local WIN = "completion"
local AREAS = { "San d'Oria", "Bastok", "Windurst", "Jeuno", "Other Areas", "Outlands", "Aht Urhgan", "Crystal War",
  "Abyssea", "Adoulin", "Coalition" }
local DIM = { 150, 150, 150, 255 }
local GOLD = { 232, 200, 120, 255 }

local view = { tab = 1, job = nil, area = 1 }

-- The areas the game's quest menu lists (the server's and the installed expansions decide), as {number, name}; all of
-- them on a client from before game.quest_areas.
local function areas()
  local out = {}
  if not game.quest_areas then
    for i, name in ipairs(AREAS) do out[i] = { i - 1, name } end
    return out
  end
  for _, a in ipairs(game.quest_areas()) do out[#out + 1] = { a, AREAS[a + 1] } end
  return out
end

local function area_names(list)
  local out = {}
  for i, a in ipairs(list) do out[i] = a[2] end
  return out
end

local function say(t) chat.add("Completion Log: " .. t) end

local function job_names()
  local out = {}
  for id = 1, 22 do out[id] = res.job(id) or tostring(id) end
  return out
end

-- The spells a job learns that we have not, by level: {id, name, level}.
local function unlearned(job)
  local known = {}
  for _, id in ipairs(game.spells()) do known[id] = true end
  local out = {}
  for id = 1, 1023 do
    local r = res.spell(id)
    local lv = r and r.levels and r.levels[job]
    if lv and lv > 0 and lv <= 99 and not known[id] and r.kind ~= 8 then
      out[#out + 1] = { id = id, name = r.name, level = lv }
    end
  end
  table.sort(out, function(a, b) return a.level == b.level and a.name < b.name or a.level < b.level end)
  return out
end

local function spell_rows(job)
  local have = (game.jobs().levels or {})[job] or 0
  local rows = {}
  for _, s in ipairs(unlearned(job)) do
    rows[#rows + 1] = { s.name, tostring(s.level), icon = { kind = "spell", id = s.id }, colour = s.level > have and DIM or nil }
  end
  return rows
end

local function state(q)
  if q.completed and not q.current then return "done" end
  if q.current then return "under way" end
  return ""
end

-- The quest files hold unused entries with Japanese working names (SDクエスト111, ATクエスト00, (仮)): on a client in
-- any language but Japanese, a name with kana, kanji or a full-width sign is one of them (in the English files up to
-- 44% of an area's entries); the game's own lists never show one we have not taken. The client's language shows in
-- Cure's name (spell 1: ケアル in Japanese).
local function japanese(name)
  return name:find("[\227-\233]") ~= nil or name:find("\239\188") ~= nil
end

local function placeholders()
  local cure = res.spell(1)
  return not (cure and cure.name and japanese(cure.name))
end

-- XI: Forever Offline and other servers that stop at Treasures of Aht Urhgan (no Wings of the Goddess, so no Crystal
-- War area): the old areas still hold quests from much later updates, Voidwatch (2010), Monstrosity (2012), Trusts
-- (2013), Records of Eminence (2014) and Unity (2015). Those are left out unless taken.
local LATE = { "^Trust: ", "^VW Op%. ", "Voidwatch", "^Monstrosity$", "^Survival of the Wisest$",
  "^Records of Eminence$", "^Unity Concord$" }

local function late(q)
  for _, p in ipairs(LATE) do
    if q.name:find(p) then return true end
  end
  return false
end

-- Whether the server stops before WotG; a client without game.quest_areas cannot tell, so nothing is held back.
local function before_wotg()
  if not game.quest_areas then return false end
  for _, a in ipairs(game.quest_areas()) do
    if a == 7 then return false end
  end
  return true
end

local function listed(q, drop, era)
  return q.current or q.completed or not ((drop and japanese(q.name)) or (era and late(q)))
end

local function quest_rows(area)
  local list = game.quests()[area] or {}
  local drop, era = placeholders(), before_wotg()
  local rows = {}
  for _, q in ipairs(list) do
    if listed(q, drop, era) then
      local s = state(q)
      rows[#rows + 1] = { q.name .. (q.repeatable and " (repeatable)" or ""), s, colour = s == "done" and DIM or (s == "under way" and GOLD or nil) }
    end
  end
  return rows
end

local function main_job()
  local p = game.player()
  return p and p.main_job or 1
end

local function open()
  view.job = view.job or main_job()
  view.shown = areas()
  if view.area > #view.shown then view.area = 1 end
  local area = view.shown[view.area]
  ui.window(WIN, {
    title = "Completion Log", width = 460,
    elements = {
      { type = "tabs", id = "tabs", items = { "Spells", "Quests" }, at = view.tab },
      { type = "choice", id = "job", label = "Job", items = job_names(), at = view.job, page = 1 },
      { type = "list", id = "spells", rows = spell_rows(view.job), height = 12, page = 1,
        columns = { { title = "Spell", width = 330 }, { title = "Level", width = 80, align = "right" } } },
      { type = "choice", id = "area", label = "Area", items = area_names(view.shown), at = view.area, page = 2 },
      { type = "list", id = "quests", rows = area and quest_rows(area[1]) or {}, height = 12, page = 2,
        columns = { { title = "Quest", width = 320 }, { title = "", width = 90, align = "right" } } },
    },
  })
  ui.open(WIN)
end

ui.on(WIN, function(_, e)
  if e.kind ~= "set" then return end
  if e.id == "tabs" then
    view.tab = e.index
  elseif e.id == "job" then
    view.job = e.index
    ui.update(WIN, "spells", { rows = spell_rows(view.job), at = 1 })
  elseif e.id == "area" then
    view.area = e.index
    local area = view.shown[view.area]
    ui.update(WIN, "quests", { rows = area and quest_rows(area[1]) or {}, at = 1 })
  end
end)

local function job_id(s)
  s = (s or ""):upper()
  for id = 1, 22 do
    if res.job(id) == s then return id end
  end
end

commands.register("left", function(what, arg)
  if not settings.get("on") then return end
  what = (what or ""):lower()
  if what == "" then
    if ui.is_open(WIN) then ui.close(WIN) else open() end
  elseif what == "spells" then
    local job = arg and job_id(arg) or main_job()
    if not job then say("no job " .. arg .. ".") return end
    local have = (game.jobs().levels or {})[job] or 0
    local list, now = unlearned(job), {}
    for _, s in ipairs(list) do
      if s.level <= have then now[#now + 1] = s.name end
    end
    say(("%s: %d spells not learned, %d of them at level %d or under%s"):format(res.job(job), #list, #now, have,
      #now > 0 and (": " .. table.concat(now, ", ", 1, math.min(#now, 12)) .. (#now > 12 and ", ..." or "")) or "."))
  elseif what == "quests" then
    local all, era = game.quests(), before_wotg()
    for _, a in ipairs(areas()) do
      local name = a[2]
      local done, going, total = 0, 0, 0
      local drop = placeholders()
      for _, q in ipairs(all[a[1]] or {}) do
        if listed(q, drop, era) then
          total = total + 1
          if q.current then going = going + 1 elseif q.completed then done = done + 1 end
        end
      end
      if total > 0 then say(("%s: %d of %d done, %d under way."):format(name, done, total, going)) end
    end
  else
    say("//left, //left spells [job], //left quests")
  end
end)
