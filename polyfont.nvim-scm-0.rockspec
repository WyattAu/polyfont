rockspec_format = "3.0"
package = "polyfont.nvim"
version = "0.10.0-1"
source = {
  url = "git+https://github.com/WyattAu/polyfont.git",
  tag = "v0.10.0",
}
description = {
  summary = "Per-token font highlighting for Neovim",
  detailed = [[
    Polyfont assigns different font families to different token scopes in your
    code. Keywords in one font, comments in another, strings in a third.
    Requires a GUI frontend (Goneovim, Neovide) for actual font rendering;
    terminal Neovim gets metadata storage for GUI consumption.
  ]],
  homepage = "https://github.com/WyattAu/polyfont",
  license = "Apache-2.0",
}
dependencies = {
  "nvim-treesitter",
}
build = {
  type = "builtin",
  copy_directories = {
    "editors/neovim",
  },
}
