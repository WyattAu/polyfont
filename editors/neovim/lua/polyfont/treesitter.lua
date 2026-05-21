local M = {}

local BOLD_WEIGHTS = {
  bold = true,
  ["semi-bold"] = true,
  ["extra-bold"] = true,
  black = true,
  medium = true,
}

local ITALIC_STYLES = {
  italic = true,
  oblique = true,
}

local CAPTURE_SCOPE_MAP = {
  ["keyword"] = "keyword",
  ["keyword.function"] = "keyword.function",
  ["keyword.operator"] = "keyword.operator",
  ["keyword.return"] = "keyword.return",
  ["keyword.debug"] = "keyword.debug",
  ["keyword.exception"] = "keyword.exception",
  ["keyword.conditional"] = "keyword.control.conditional",
  ["keyword.repeat"] = "keyword.control.repeat",
  ["keyword.import"] = "keyword.control.import",
  ["keyword.directive"] = "keyword.directive",
  ["function"] = "entity.name.function",
  ["function.call"] = "entity.name.function.call",
  ["function.builtin"] = "support.function",
  ["function.macro"] = "entity.name.function.macro",
  ["method"] = "entity.name.function.method",
  ["method.call"] = "entity.name.function.method.call",
  ["constructor"] = "entity.name.function.constructor",
  ["comment"] = "comment",
  ["comment.documentation"] = "comment.documentation",
  ["comment.error"] = "comment.error",
  ["comment.warning"] = "comment.warning",
  ["comment.hint"] = "comment.hint",
  ["comment.todo"] = "comment.todo",
  ["comment.note"] = "comment.note",
  ["string"] = "string",
  ["string.regexp"] = "string.regexp",
  ["string.escape"] = "string.escape",
  ["string.special"] = "string.special",
  ["string.special.symbol"] = "string.special.symbol",
  ["string.special.url"] = "string.special.url",
  ["string.special.path"] = "string.special.path",
  ["variable"] = "variable",
  ["variable.builtin"] = "support.variable",
  ["variable.parameter"] = "variable.parameter",
  ["variable.member"] = "variable.member",
  ["constant"] = "constant",
  ["constant.builtin"] = "constant.builtin",
  ["constant.macro"] = "constant.macro",
  ["number"] = "constant.numeric",
  ["number.float"] = "constant.numeric.float",
  ["type"] = "entity.name.type",
  ["type.builtin"] = "support.type",
  ["type.definition"] = "entity.name.type.definition",
  ["operator"] = "keyword.operator",
  ["punctuation"] = "punctuation",
  ["punctuation.bracket"] = "punctuation.bracket",
  ["punctuation.delimiter"] = "punctuation.delimiter",
  ["punctuation.special"] = "punctuation.special",
  ["tag"] = "entity.name.tag",
  ["tag.attribute"] = "entity.other.attribute-name",
  ["tag.delimiter"] = "entity.name.tag.delimiter",
  ["attribute"] = "entity.other.attribute-name",
  ["namespace"] = "entity.name.namespace",
  ["boolean"] = "constant.language.boolean",
  ["character"] = "constant.character",
  ["character.special"] = "constant.character.special",
  ["conditional"] = "keyword.control.conditional",
  ["repeat"] = "keyword.control.repeat",
  ["exception"] = "keyword.control.exception",
  ["include"] = "keyword.control.import",
  ["label"] = "entity.name.label",
  ["text"] = "text",
  ["text.title"] = "markup.heading",
  ["text.literal"] = "markup.raw",
  ["text.uri"] = "string.special.url",
  ["text.math"] = "markup.math",
  ["text.note"] = "markup.note",
  ["text.reference"] = "markup.reference",
  ["text.emphasis"] = "markup.italic",
  ["text.strike"] = "markup.strikethrough",
  ["text.strong"] = "markup.bold",
  ["text.underline"] = "markup.underline",
  ["module"] = "entity.name.namespace",
  ["string.documentation"] = "string.documentation",
  ["diff.plus"] = "diff.plus",
  ["diff.minus"] = "diff.minus",
  ["diff.delta"] = "diff.delta",
}

M.capture_scope_map = CAPTURE_SCOPE_MAP

function M.scope_from_capture(capture_name)
  return CAPTURE_SCOPE_MAP[capture_name]
end

function M.is_bold(weight)
  return BOLD_WEIGHTS[weight] or false
end

function M.is_italic(style)
  return ITALIC_STYLES[style] or false
end

function M.scope_to_hl_group(scope)
  return "Polyfont" .. scope:gsub("[%.%-]", "_"):gsub("^%l", string.upper)
end

function M.scope_matches(scope, pattern)
  if pattern == "*" then
    return true
  end

  local negated = false
  if pattern:sub(1, 1) == "-" then
    negated = true
    pattern = pattern:sub(2)
  end

  local pattern_parts = vim.split(pattern, "%.", { plain = true })
  local scope_parts = vim.split(scope, "%.", { plain = true })

  if #scope_parts < #pattern_parts then
    return negated
  end

  local matched = true
  for i, part in ipairs(pattern_parts) do
    if part ~= "*" and scope_parts[i] ~= part then
      matched = false
      break
    end
  end

  if negated then
    return not matched
  end
  return matched
end

function M.scope_specificity(scope)
  local count = 0
  for _ in scope:gmatch("[^.]+") do
    count = count + 1
  end
  return count
end

function M.scope_matches_selector(scope, selector)
  for pattern in selector:gmatch("[^,]+") do
    pattern = pattern:match("^%s*(.-)%s*$")
    if pattern ~= "" and M.scope_matches(scope, pattern) then
      return true
    end
  end
  return false
end

function M.resolve_scope(scope, rules)
  local best_rule = nil
  local best_specificity = -1
  local best_index = math.huge

  for i, rule in ipairs(rules) do
    if M.scope_matches_selector(scope, rule.scope) then
      local spec = M.scope_specificity(rule.scope)
      if spec > best_specificity or (spec == best_specificity and i < best_index) then
        best_rule = rule
        best_specificity = spec
        best_index = i
      end
    end
  end

  return best_rule
end

function M.get_buffer_tokens(bufnr)
  bufnr = bufnr or 0
  local ok, parser = pcall(vim.treesitter.get_parser, bufnr)
  if not ok or not parser then
    return {}
  end

  local lang = parser:lang()
  local ok2, query = pcall(vim.treesitter.query.parse, lang, vim.treesitter.query.get(lang, "highlights"))
  if not ok2 or not query then
    return {}
  end

  local tree = parser:parse()[1]
  if not tree then
    return {}
  end

  local tokens = {}
  for capture_id, node, metadata in query:iter_captures(tree:root(), bufnr) do
    local capture_name = query.captures[capture_id]
    local scope = M.scope_from_capture(capture_name)
    if scope then
      local start_row, start_col, end_row, end_col = node:range()
      table.insert(tokens, {
        capture = capture_name,
        scope = scope,
        start_row = start_row,
        start_col = start_col,
        end_row = end_row,
        end_col = end_col,
      })
    end
  end

  return tokens
end

return M
