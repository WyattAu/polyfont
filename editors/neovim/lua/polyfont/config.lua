local M = {}

local toml = require("polyfont.toml")

local VALID_WEIGHTS = {
  thin = true,
  ["extra-light"] = true,
  light = true,
  regular = true,
  medium = true,
  ["semi-bold"] = true,
  bold = true,
  ["extra-bold"] = true,
  black = true,
}

local VALID_STYLES = {
  normal = true,
  italic = true,
  oblique = true,
}

local CURRENT_VERSION = 1

function M.validate(config)
  if type(config.version) ~= "number" then
    return false, "missing or invalid 'version' field"
  end
  if config.version ~= CURRENT_VERSION then
    return false,
      string.format("unsupported version %d (expected %d)", config.version, CURRENT_VERSION)
  end

  if config.default then
    if type(config.default.family) ~= "string" or config.default.family:match("^%s*$") then
      return false, "default font family must not be empty"
    end
  end

  if config.rules then
    if not vim.islist(config.rules) then
      return false, "'rules' must be an array"
    end
    for i, rule in ipairs(config.rules) do
      if type(rule.scope) ~= "string" or rule.scope:match("^%s*$") then
        return false, string.format("rule at index %d has an empty scope", i)
      end
      if type(rule.font) ~= "table" then
        return false, string.format("rule at index %d is missing 'font' table", i)
      end
      if type(rule.font.family) ~= "string" or rule.font.family:match("^%s*$") then
        return false,
          string.format("rule at index %d (scope '%s') has an empty font family", i, rule.scope)
      end
    end
  end

  return true
end

local function normalize_font(font)
  return {
    family = font.family or "",
    fallbacks = font.fallbacks or {},
    weight = font.weight or "regular",
    style = font.style or "normal",
    size = font.size,
  }
end

function M.normalize(raw)
  local config = {
    version = raw.version or 0,
    default = nil,
    rules = {},
  }

  if raw.default then
    config.default = normalize_font(raw.default)
  end

  if raw.rules then
    for _, rule in ipairs(raw.rules) do
      table.insert(config.rules, {
        scope = rule.scope,
        font = normalize_font(rule.font or {}),
      })
    end
  end

  return config
end

function M.find_config(start_dir)
  local current = vim.fn.fnamemodify(start_dir, ":p")
  current = current:gsub("/+$", "")

  while true do
    local candidate = current .. "/.polyfont.toml"
    if vim.fn.filereadable(candidate) == 1 then
      return candidate
    end
    local parent = vim.fn.fnamemodify(current, ":h")
    if parent == current then
      break
    end
    current = parent
  end

  local home_config = vim.fn.expand("~/.polyfont.toml")
  if vim.fn.filereadable(home_config) == 1 then
    return home_config
  end

  return nil
end

function M.load(path)
  local f = io.open(path, "r")
  if not f then
    return nil, "cannot open " .. path
  end
  local content = f:read("*a")
  f:close()

  local ok, raw = pcall(toml.parse, content)
  if not ok then
    return nil, "parse error: " .. tostring(raw)
  end

  local config = M.normalize(raw)
  local valid, err = M.validate(config)
  if not valid then
    return nil, "validation error: " .. err
  end

  return config
end

function M.load_from_cwd()
  local path = M.find_config(vim.fn.getcwd())
  if not path then
    return nil, "no .polyfont.toml found"
  end
  return M.load(path)
end

return M
