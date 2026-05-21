local M = {}

local config_mod = require("polyfont.config")
local highlights = require("polyfont.highlights")

M.config = nil
M.config_path = nil
M._watcher = nil
M._watcher_timer = nil

local defaults = {
  auto_start = true,
  watch_config = true,
}

M.options = {}

function M.setup(opts)
  opts = opts or {}
  M.options = vim.tbl_deep_extend("force", defaults, opts)

  local cfg, err = config_mod.load_from_cwd()
  if not cfg then
    vim.notify("[polyfont] " .. (err or "no config found"), vim.log.levels.WARN)
    return
  end

  M.config = cfg
  M.config_path = config_mod.find_config(vim.fn.getcwd())
  vim.g.polyfont_config = cfg

  highlights.apply(cfg)
  M._apply_all_buffers()

  if M.options.watch_config and M.config_path then
    M.watch(M.config_path)
  end

  vim.api.nvim_create_autocmd("FileType", {
    group = vim.api.nvim_create_augroup("polyfont", { clear = true }),
    callback = function(args)
      if M.config then
        vim.schedule(function()
          if vim.api.nvim_buf_is_loaded(args.buf) then
            highlights.apply(M.config, args.buf)
          end
        end)
      end
    end,
  })

  vim.api.nvim_create_autocmd("DirChanged", {
    group = vim.api.nvim_create_augroup("polyfont_dir", { clear = true }),
    callback = function()
      M.reload()
    end,
  })
end

function M._apply_all_buffers()
  if not M.config then
    return
  end
  for _, bufnr in ipairs(vim.api.nvim_list_bufs()) do
    if vim.api.nvim_buf_is_loaded(bufnr) and vim.bo[bufnr].buflisted then
      vim.schedule(function()
        if vim.api.nvim_buf_is_loaded(bufnr) then
          highlights.apply(M.config, bufnr)
        end
      end)
    end
  end
end

function M.reload()
  M.unwatch()

  local cfg, err = config_mod.load_from_cwd()
  if not cfg then
    vim.notify("[polyfont] " .. (err or "no config found"), vim.log.levels.WARN)
    return
  end

  M.config = cfg
  M.config_path = config_mod.find_config(vim.fn.getcwd())
  vim.g.polyfont_config = cfg

  highlights.clear_all()
  highlights.apply(cfg)
  M._apply_all_buffers()

  if M.options.watch_config and M.config_path then
    M.watch(M.config_path)
  end

  vim.notify("[polyfont] config reloaded", vim.log.levels.INFO)
end

function M.watch(path)
  M.unwatch()

  if not path or vim.fn.filereadable(path) ~= 1 then
    return
  end

  local dir = vim.fn.fnamemodify(path, ":h")
  local basename = vim.fn.fnamemodify(path, ":t")

  M._watcher = vim.loop.new_fs_event()
  if not M._watcher then
    return
  end

  M._watcher:start(dir, {}, function(err, filename, events)
    if err then
      return
    end
    if filename == basename and (events.change or events.rename) then
      if M._watcher_timer then
        M._watcher_timer:stop()
      end
      M._watcher_timer = vim.loop.new_timer()
      M._watcher_timer:start(200, 0, function()
        vim.schedule(function()
          M.reload()
        end)
      end)
    end
  end)
end

function M.unwatch()
  if M._watcher then
    M._watcher:stop()
    M._watcher:close()
    M._watcher = nil
  end
  if M._watcher_timer then
    M._watcher_timer:stop()
    M._watcher_timer:close()
    M._watcher_timer = nil
  end
end

function M.status()
  if not M.config then
    return "no config loaded"
  end
  local lines = { string.format("config: %s", M.config_path or "unknown") }
  if M.config.default then
    table.insert(lines, string.format("default font: %s (%s, %s)", M.config.default.family, M.config.default.weight, M.config.default.style))
  end
  table.insert(lines, string.format("rules: %d", #M.config.rules))
  for _, rule in ipairs(M.config.rules) do
    table.insert(lines, string.format("  %s -> %s (%s, %s)", rule.scope, rule.font.family, rule.font.weight, rule.font.style))
  end
  return table.concat(lines, "\n")
end

return M
