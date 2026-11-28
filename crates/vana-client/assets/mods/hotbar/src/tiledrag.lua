-- A library tile dragged from the manager onto a slot: its macro while the drag lasts, and its id kept for a drop
-- that lands after the drag ends.

local M = {}

-- Set by init.lua: drag_ended().
M.hooks = {}

M.dragging = nil
M.last = nil
-- The HUD ghost is drawn over the windows (ghost.top): canvases need not draw their own copy.
M.top = false

function M.start(id, act, label)
  M.dragging = { id = id, act = act, label = label }
  M.last = id
end

function M.stop()
  M.dragging = nil
  if M.hooks.drag_ended then M.hooks.drag_ended() end
end

return M
