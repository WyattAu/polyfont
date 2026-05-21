# Polyfont

**Per-token font highlighting for code editors.** Go beyond color — use different fonts for keywords, functions, comments, strings, and more.

## What it does

Most code editors support per-token **color** highlighting. Polyfont extends this to per-token **font** highlighting:

- Keywords in **Maple Mono Bold**
- Functions in **Monaspace Argon**
- Comments in *IBM Plex Mono Italic*
- Strings in Source Code Pro Light
- Constants in Monaspace Radon

```
fn calculate_total(items: &[Item]) -> f64 {    ← keyword (Maple Mono Bold)
// Computes the total price                    ← comment (IBM Plex Mono Italic)
    let sum: f64 = items                       ← variable (JetBrains Mono)
        .iter()                                ← function (Monaspace Argon)
        .map(|item| item.price)                ← function (Monaspace Argon)
        .sum();                                ← method (Monaspace Krypton)
    sum * 1.08                                 ← constant (Monaspace Radon)
}                                              ← punctuation (Fira Code)
```

## How it works

```
┌─────────────────────────────────────────────────┐
│  .polyfont.toml                                 │
│  Config: scope → font mappings                  │
└─────────────┬───────────────────────────────────┘
              │
    ┌─────────┴──────────┐
    │   polyfont-core    │  Rust library
    │   Scope resolution │  Match tokens to fonts
    │   Font assignment  │  Specificity scoring
    └────┬────┬────┬─────┘
         │    │    │
    ┌────┘    │    └─────────────┐
    ▼         ▼                  ▼
┌────────┐ ┌─────────┐ ┌─────────────────┐
│ VSCode │ │ Neovim  │ │ CLI / LSP       │
│ Ext    │ │ Plugin  │ │ polyfont check  │
│        │ │         │ │ polyfont vscode │
│textMate│ │Tree-    │ │ polyfont neovim │
│Rules   │ │sitter   │ │ polyfont kitty  │
│API     │ │+extmarks│ │                 │
└────────┘ └─────────┘ └─────────────────┘
```

## Quick Start

### 1. Create a config file

Copy the example config and customize:

```bash
cp polyfont.example.toml .polyfont.toml
```

Or create `.polyfont.toml` in your project root or home directory:

```toml
version = 1

[default]
family = "Fira Code"
fallbacks = ["JetBrains Mono", "monospace"]

[[rules]]
scope = "keyword"
[rules.font]
family = "Maple Mono"
weight = "bold"

[[rules]]
scope = "comment"
[rules.font]
family = "IBM Plex Mono"
style = "italic"

[[rules]]
scope = "entity.name.function"
[rules.font]
family = "Monaspace Argon"
weight = "semi-bold"

[[rules]]
scope = "string"
[rules.font]
family = "Source Code Pro"
weight = "light"
```

### 2. Install the editor extension

#### VSCode (requires VSCode 1.97+)

VSCode 1.97+ added native `fontFamily` support in `textMateRules` — the cleanest integration path.

```bash
# From source
cd editors/vscode
npm install
npm run build
# Install as development extension or package as .vsix
```

Or use the CLI to generate settings:

```bash
cargo run -p polyfont-cli -- vscode > .vscode/settings.json
```

#### Neovim (requires Neovim 0.9+)

Using your plugin manager:

```lua
-- lazy.nvim
{
  "WyattAu/polyfont",
  dir = "/path/to/polyfont/editors/neovim",
  config = function()
    require("polyfont").setup()
  end,
}
```

### 3. CLI Tool

Build and use the polyfont CLI:

```bash
cargo build --release -p polyfont-cli

# Validate your config
polyfont check

# Generate VSCode settings
polyfont vscode

# Generate Neovim Lua config
polyfont neovim

# Generate Kitty terminal config
polyfont kitty

# Debug: show all resolved rules
polyfont dump
```

## Monorepo Structure

```
polyfont/
├── crates/
│   ├── polyfont-core/       # Core types: FontRule, FontSpec, TokenInfo, Engine trait
│   ├── polyfont-config/     # .polyfont.toml parsing, validation, merging
│   ├── polyfont-scope/      # TextMate scope matching, specificity resolution
│   ├── polyfont-cli/        # CLI tool (check, vscode, neovim, kitty, dump)
│   └── polyfont-lsp/        # LSP server for editor integration
├── editors/
│   ├── vscode/              # VSCode extension (TypeScript)
│   └── neovim/              # Neovim plugin (Lua)
├── polyfont.example.toml    # Example configuration
└── .specs/                  # R&D documentation
```

## Configuration Reference

### Font Rule

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `scope` | string | yes | TextMate scope selector (e.g., `"keyword"`, `"entity.name.function"`) |
| `font.family` | string | yes | Primary font family name |
| `font.fallbacks` | string[] | no | Fallback fonts in priority order |
| `font.weight` | string | no | `thin`, `extra-light`, `light`, `regular`, `medium`, `semi-bold`, `bold`, `extra-bold`, `black` |
| `font.style` | string | no | `normal`, `italic`, `oblique` |
| `font.size` | number | no | Font size in points (omit to inherit from editor) |

### Scope Matching

Scopes use TextMate notation with hierarchical matching:

- `"keyword"` matches `keyword`, `keyword.control`, `keyword.operator`
- `"entity.name.function"` matches only `entity.name.function` (most specific)
- Most specific rule wins

### Scope Reference

Common TextMate scopes for programming:

| Scope | Description |
|-------|-------------|
| `keyword` | Language keywords (if, else, fn, let) |
| `keyword.control` | Control flow keywords |
| `comment` | Comments |
| `string` | String literals |
| `entity.name.function` | Function names |
| `entity.name.type` | Type names |
| `variable` | Variables |
| `variable.parameter` | Function parameters |
| `constant` | Constants |
| `constant.numeric` | Numbers |
| `support.function` | Built-in/library functions |
| `storage.type` | Type annotations |
| `punctuation` | Punctuation characters |
| `operator` | Operators |

## Editor Compatibility

| Editor | Status | Mechanism | Notes |
|--------|--------|-----------|-------|
| VSCode 1.97+ | FULL | `textMateRules` fontFamily | Native support since Dec 2025 |
| Neovim GUI (Goneovim) | PARTIAL | Highlight groups + font metadata | Requires GUI support |
| Neovim GUI (Neovide) | PARTIAL | Bold/italic via highlight groups | Font family requires patching |
| Kitty terminal | PARTIAL | `symbol_map` + PUA ranges | Config-level, not runtime |
| Terminal (generic) | LIMITED | Bold/italic/underline only | No per-char font support |

## Development

```bash
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Run CLI
cargo run -p polyfont-cli -- check

# Build VSCode extension
cd editors/vscode && npm run build

# Format code
cargo fmt --all

# Lint
cargo clippy --workspace --all-targets -- -D warnings
```

## License

Apache License 2.0
