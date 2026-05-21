vim.api.nvim_create_user_command("PolyfontSetup", function()
  require("polyfont").setup()
end, { desc = "Load and apply polyfont configuration" })

vim.api.nvim_create_user_command("PolyfontReload", function()
  require("polyfont").reload()
end, { desc = "Reload polyfont configuration" })

vim.api.nvim_create_user_command("PolyfontStatus", function()
  local status = require("polyfont").status()
  vim.notify(status, vim.log.levels.INFO)
end, { desc = "Show polyfont status" })

vim.api.nvim_create_user_command("PolyfontClear", function()
  require("polyfont.highlights").clear_all()
  vim.notify("[polyfont] cleared all highlights", vim.log.levels.INFO)
end, { desc = "Clear all polyfont highlights" })
