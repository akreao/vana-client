-- Curated best-in-slot references per level cap, job and set: {slot name = item name}. Compiled from public
-- guides (the source of each job's lists is noted beside it). A slot not listed takes the automatic reference: the
-- best-scoring piece in the whole item list at the cap.

local M = {}

M.LISTS = {
  [75] = {},
  [99] = {},
}

-- The curated slots of a set, or nil.
function M.lookup(cap, job, set_name)
  local by_cap = M.LISTS[cap]
  local by_job = by_cap and by_cap[job]
  return by_job and by_job[set_name] or nil
end

return M
