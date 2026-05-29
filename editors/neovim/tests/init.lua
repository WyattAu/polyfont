-- Add lua/polyfont to the Lua search path so tests can require("polyfont.xxx")
local root = vim.fn.fnamemodify(vim.fn.expand("<sfile>:p:h:h"), ":p")
package.path = root .. "lua/?.lua;" .. root .. "lua/?/init.lua;" .. package.path
