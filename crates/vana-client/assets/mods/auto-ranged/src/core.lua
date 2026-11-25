-- Auto Ranged's decisions, kept free of the client for the tests: when to shoot, when to wait.
--
-- A shot is started only while you are engaged with a living monster targeted, a ranged weapon or ammo on, standing
-- still, and nothing of yours under way (a shot, a spell, an item). After a shot lands it waits `delay` seconds; a
-- refusal such as "out of range" or "cannot see" pauses it a few seconds instead of trying again at once. In the
-- "hate" mode it also holds while the monster's last action (within 10 s) was on you.

local M = {}

M.MODES = { "manual", "auto", "hate" }
M.LABELS = { manual = "Manual", auto = "Auto", hate = "Until hate" }
M.INTERRUPTED = 28787
-- battle messages for your own failed shot that mean "not now": out of range, cannot see, too far, no ammo...
M.PAUSE = { [4] = true, [5] = true, [12] = true, [16] = true, [17] = true, [78] = true, [217] = true, [218] = true }

function M.new()
  return { mode = "manual", delay = 0.5, busy_until = 0, next_at = 0, hate = {}, last_pos = nil, moved_at = -math.huge,
    holding = nil }
end

-- s, the game as `g` = { now, me (player), target (entity or nil), ranged (bool) } -> true to shoot now. Also keeps
-- s.holding: why it is not shooting (for the window), nil when it is free to.
function M.tick(s, g)
  local t, me = g.now, g.me
  if me and me.pos then
    local p = me.pos
    if s.last_pos and (math.abs(s.last_pos.x - p.x) > 0.05 or math.abs(s.last_pos.z - p.z) > 0.05) then
      s.moved_at = t
    end
    s.last_pos = { x = p.x, z = p.z }
  end
  s.holding = nil
  if s.mode == "manual" or not me then return false end
  if t < s.busy_until or t < s.next_at then return false end
  if me.status ~= 1 then return false end
  local mob = g.target
  if not mob or mob.kind ~= "mob" or (mob.hpp or 0) <= 0 then return false end
  if not g.ranged then
    s.holding = "no ranged weapon or ammo on"
    return false
  end
  if t - s.moved_at < 0.4 then
    s.holding = "moving"
    return false
  end
  local h = s.hate[mob.id]
  if s.mode == "hate" and h and h.me and t - h.at < 10 then
    s.holding = "it is attacking you"
    return false
  end
  s.busy_until = t + 6 -- until the shot's own action says how it went
  return true
end

-- Every action: yours keep the shot timing; a monster's tell who it is after.
function M.action(s, a, me_id, t)
  if a.actor == me_id then
    local c = a.category
    if c == 12 then
      if a.param == M.INTERRUPTED then s.busy_until, s.next_at = 0, t + 1.5 else s.busy_until = t + 6 end
    elseif c == 2 then
      s.busy_until, s.next_at = 0, t + s.delay
    elseif c == 8 or c == 9 then
      if a.param == M.INTERRUPTED then s.busy_until, s.next_at = 0, t + 0.5 else s.busy_until = t + 12 end
    elseif c == 4 or c == 5 then
      s.busy_until, s.next_at = 0, t + 1
    end
    return
  end
  local c = a.category
  if not (c == 1 or c == 4 or c == 7 or c == 8 or c == 11) or not a.targets[1] then return end
  local on_me = false
  for _, target in ipairs(a.targets) do
    if target.id == me_id then on_me = true end
  end
  if c == 7 or c == 8 then
    -- readying a move or a spell: only one on you counts
    if on_me then s.hate[a.actor] = { me = true, at = t } end
  else
    s.hate[a.actor] = { me = on_me, at = t }
  end
end

function M.message(s, actor, message, me_id, t)
  if actor == me_id and s.mode ~= "manual" and M.PAUSE[message % 32768] then
    s.busy_until, s.next_at = 0, t + 3
  end
end

function M.reset(s)
  s.hate, s.busy_until, s.next_at = {}, 0, 0
end

return M
