local M = {}

local ts = require("polyfont.treesitter")

local active_providers = {}

local function build_highlight_groups(config)
  vim.g.polyfont_font_map = {}

  if config.default then
    local hl_name = "PolyfontDefault"
    vim.api.nvim_set_hl(0, hl_name, {
      bold = ts.is_bold(config.default.weight),
      italic = ts.is_italic(config.default.style),
    })
    vim.g.polyfont_font_map[hl_name] = {
      family = config.default.family,
      fallbacks = config.default.fallbacks,
      weight = config.default.weight,
      style = config.default.style,
      size = config.default.size,
      scope = "*",
    }
  end

  for _, rule in ipairs(config.rules) do
    local hl_name = ts.scope_to_hl_group(rule.scope)
    vim.api.nvim_set_hl(0, hl_name, {
      bold = ts.is_bold(rule.font.weight),
      italic = ts.is_italic(rule.font.style),
    })
    vim.g.polyfont_font_map[hl_name] = {
      family = rule.font.family,
      fallbacks = rule.font.fallbacks,
      weight = rule.font.weight,
      style = rule.font.style,
      size = rule.font.size,
      scope = rule.scope,
    }
  end

  M.build_font_metadata(config)
end

local function clear_highlight_groups()
  if vim.g.polyfont_font_map then
    for hl_name, _ in pairs(vim.g.polyfont_font_map) do
      pcall(vim.api.nvim_set_hl, 0, hl_name, {})
    end
  end
  vim.g.polyfont_font_map = {}
end

local function resolve_hl_for_scope(scope, rules)
  local rule = ts.resolve_scope(scope, rules)
  if rule then
    return ts.scope_to_hl_group(rule.scope)
  end
  return nil
end

function M.setup_decoration_provider(bufnr, config)
  bufnr = bufnr or 0

  if active_providers[bufnr] then
    vim.api.nvim_set_decoration_provider(bufnr, nil)
    active_providers[bufnr] = nil
  end

  local ns = vim.api.nvim_create_namespace("polyfont")
  local rules = vim.list_extend({}, config.rules or {})
  if config.default then
    table.insert(rules, { scope = "*", font = config.default })
  end

  local last_tick = nil

  active_providers[bufnr] = vim.api.nvim_set_decoration_provider(bufnr, function(ctx)
    if not config or #rules == 0 then
      return
    end

    if ctx.tick == last_tick then
      return
    end
    last_tick = ctx.tick

    local buf = ctx.buf

    local ok, parser = pcall(vim.treesitter.get_parser, buf)
    if not ok or not parser then
      return
    end

    local lang = parser:lang()
    local ok2, query = pcall(vim.treesitter.query.parse, lang, vim.treesitter.query.get(lang, "highlights"))
    if not ok2 or not query then
      return
    end

    parser:parse(ctx.bufnr)

    for capture_id, node in query:iter_captures(parser:trees()[1]:root(), buf, ctx.first_row, ctx.last_row + 1) do
      local capture_name = query.captures[capture_id]
      local scope = ts.scope_from_capture(capture_name)
      if scope then
        local hl_group = resolve_hl_for_scope(scope, rules)
        if hl_group then
          local start_row, start_col, end_row, end_col = node:range()
          pcall(vim.api.nvim_buf_set_extmark, buf, ns, start_row, start_col, {
            end_row = end_row,
            end_col = end_col,
            hl_group = hl_group,
            priority = 120,
            ephemeral = true,
          })
        end
      end
    end
  end)
end

function M.clear_buffer(bufnr)
  bufnr = bufnr or 0
  local ns = vim.api.nvim_create_namespace("polyfont")
  vim.api.nvim_buf_clear_namespace(bufnr, ns, 0, -1)

  if active_providers[bufnr] then
    vim.api.nvim_set_decoration_provider(bufnr, nil)
    active_providers[bufnr] = nil
  end
end

function M.font_metadata_table()
  return vim.g.polyfont_font_metadata or {}
end

function M.build_font_metadata(config)
  local metadata = {}

  if config.default then
    metadata.default = {
      family = config.default.family,
      weight = config.default.weight,
      style = config.default.style,
      fallbacks = config.default.fallbacks or {},
    }
  end

  for _, rule in ipairs(config.rules) do
    local key = rule.scope:gsub("[%.%-]", "_")
    metadata[key] = {
      scope = rule.scope,
      family = rule.font.family,
      weight = rule.font.weight,
      style = rule.font.style,
      fallbacks = rule.font.fallbacks or {},
    }
  end

  vim.g.polyfont_font_metadata = metadata
end

function M.get_font_for_scope(scope)
  local metadata = M.font_metadata_table()
  if not metadata or not M._rules then
    return nil
  end

  local rule = ts.resolve_scope(scope, M._rules)
  if not rule then
    local parts = vim.split(scope, "%.", { plain = true })
    for i = #parts - 1, 1, -1 do
      local parent = table.concat(vim.list_slice(parts, 1, i), ".")
      local parent_rule = ts.resolve_scope(parent, M._rules)
      if parent_rule then
        rule = parent_rule
        break
      end
    end
  end

  if not rule then
    return metadata.default or nil
  end

  local key = rule.scope:gsub("[%.%-]", "_")
  return metadata[key] or metadata.default
end

function M.apply(config, bufnr)
  bufnr = bufnr or 0

  M._rules = vim.list_extend({}, config.rules or {})
  if config.default then
    table.insert(M._rules, { scope = "*", font = config.default })
  end

  build_highlight_groups(config)

  if not vim.api.nvim_buf_is_loaded(bufnr) then
    return
  end

  M.setup_decoration_provider(bufnr, config)
end

function M.clear_all()
  clear_highlight_groups()
  for bufnr, _ in pairs(active_providers) do
    M.clear_buffer(bufnr)
  end
end

return M
