-- XI: Forever Offline's hardcore modes as its server tells them: "[HC-ICONS] !Name=1,Other=3" names every hardcore
-- character in the area (the "!" replaces what was known), "[HC-ICONS] Name=2" one arriving later.

local M = {}

M.NAMES = { "iron", "steel", "silver", "random" }
M.LABELS = { "Iron", "Steel", "Silver", "Random" }

-- known: name -> mode (1-4), changed in place by one line's text (what follows the mark).
function M.read(known, text)
  text = (text or ""):gsub("^%s+", ""):gsub("%s+$", "")
  if text:sub(1, 1) == "!" then
    for k in pairs(known) do known[k] = nil end
    text = text:sub(2)
  end
  for name, mode in text:gmatch("([^,=]+)=(%d+)") do
    mode = tonumber(mode)
    known[name] = M.NAMES[mode] and mode or nil
  end
  return known
end

return M
