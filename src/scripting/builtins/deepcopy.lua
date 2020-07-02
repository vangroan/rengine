
--[[
  Recursively copy table and inner values.

  From: http://lua-users.org/wiki/CopyTable
--]]
function table.deepcopy(orig, copies)
  copies = copies or {}
  local orig_type = type(orig)
  local copy
  if orig_type == 'table' then
    if copies[orig] then
      copy = copies[orig]
    else
      copy = {}
      copies[orig] = copy
      for orig_key, orig_value in next, orig, nil do
        copy[table.deepcopy(orig_key, copies)] = table.deepcopy(orig_value, copies)
      end
      setmetatable(copy, table.deepcopy(getmetatable(orig), copies))
    end
  else -- number, string, boolean, etc
    copy = orig
  end
  return copy
end
