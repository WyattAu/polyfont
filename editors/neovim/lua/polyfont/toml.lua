local M = {}

local is_list = vim.islist or vim.tbl_islist

local function trim(s)
  return s:match("^%s*(.-)%s*$")
end

local function parse_string(raw)
  if raw:match('^".*"$') then
    return raw:sub(2, -2):gsub('\\"', '"')
  elseif raw:match("^'.*'$") then
    return raw:sub(2, -2)
  end
  return nil
end

local function parse_number(raw)
  if raw:match("^-?%d+%.%d+$") then
    return tonumber(raw)
  elseif raw:match("^-?%d+$") then
    return tonumber(raw)
  end
  return nil
end

local function parse_boolean(raw)
  if raw == "true" then return true end
  if raw == "false" then return false end
  return nil
end

local function parse_array(raw)
  raw = trim(raw)
  if raw:sub(1, 1) ~= "[" or raw:sub(-1) ~= "]" then
    return nil
  end
  local inner = trim(raw:sub(2, -2))
  if inner == "" then return {} end

  local items = {}
  local depth = 0
  local current = ""
  local in_string = false
  local string_char = nil

  for i = 1, #inner do
    local c = inner:sub(i, i)
    if in_string then
      current = current .. c
      if c == string_char and inner:sub(i - 1, i - 1) ~= "\\" then
        in_string = false
      end
    elseif c == '"' or c == "'" then
      in_string = true
      string_char = c
      current = current .. c
    elseif c == "[" then
      depth = depth + 1
      current = current .. c
    elseif c == "]" then
      depth = depth - 1
      current = current .. c
    elseif c == "," and depth == 0 then
      local item = trim(current)
      if item ~= "" then
        table.insert(items, M.parse_value(item))
      end
      current = ""
    else
      current = current .. c
    end
  end

  local last = trim(current)
  if last ~= "" then
    table.insert(items, M.parse_value(last))
  end

  return items
end

function M.parse_value(raw)
  raw = trim(raw)
  if raw == "" then return nil end

  local s = parse_string(raw)
  if s ~= nil then return s end

  local b = parse_boolean(raw)
  if b ~= nil then return b end

  local n = parse_number(raw)
  if n ~= nil then return n end

  local a = parse_array(raw)
  if a ~= nil then return a end

  return raw
end

local function split_path(path)
  local parts = {}
  for part in path:gmatch("[^.]+") do
    table.insert(parts, part)
  end
  return parts
end

local function resolve_table(root, parts, for_array)
  local current = root
  for i = 1, #parts do
    local part = parts[i]
    if current[part] == nil then
      current[part] = {}
    end
    if is_list(current[part]) then
      current = current[part][#current[part]]
    else
      current = current[part]
    end
  end
  return current
end

local function enter_array(root, parts)
  local parent = root
  for i = 1, #parts - 1 do
    local part = parts[i]
    if parent[part] == nil then
      parent[part] = {}
    end
    if is_list(parent[part]) then
      parent = parent[part][#parent[part]]
    else
      parent = parent[part]
    end
  end

  local name = parts[#parts]
  if parent[name] == nil then
    parent[name] = {}
  end

  local new_entry = {}
  table.insert(parent[name], new_entry)
  return new_entry
end

function M.parse(text)
  local root = {}
  local current = root

  for line in text:gmatch("[^\r\n]+") do
    line = line:gsub("#.*$", "")
    line = trim(line)

    if line == "" then
      -- skip
    elseif line:match("^%[%[") then
      local path = line:match("^%[%[(.+)%]%]$")
      if not path then error("Invalid array of tables syntax: " .. line) end
      local parts = split_path(path)
      current = enter_array(root, parts)
    elseif line:match("^%[") then
      local path = line:match("^%[(.+)%]$")
      if not path then error("Invalid table syntax: " .. line) end
      local parts = split_path(path)
      current = resolve_table(root, parts)
    else
      local key, raw_value = line:match("^([%w%._-]+)%s*=%s*(.+)$")
      if key and raw_value then
        current[key] = M.parse_value(raw_value)
      end
    end
  end

  return root
end

return M
