# Polyfont for Helix

## Overview

Helix runs in the terminal and cannot natively switch fonts on a per-token basis. Terminal emulators render all characters with a single font (or a configured fallback chain), so there is no API to apply different fonts to different syntax scopes.

The workaround uses [Kitty terminal](https://sw.kovidgoyal.net/kitty/)'s `symbol_map` feature to map Unicode code point ranges to specific fonts. This is an approximation, not true per-scope rendering.

## How It Works

Kitty's `symbol_map` configuration maps Unicode ranges to font families. For example:

```
symbol_map U+2200-U+22FF JetBrains Mono
symbol_map U+2500-U+257F Fira Code
```

The `polyfont kitty` command analyzes your `.polyfont.toml` config and generates `symbol_map` entries that map characteristic Unicode ranges to the fonts assigned to relevant scopes. For instance, if your config assigns a specific font to `keyword.control`, the generated config might map common operator symbols used in control flow to that font.

### Limitations

This approach has fundamental limitations:

- **Same character, different scopes:** The same Unicode character can appear in different scopes (e.g., `=` as assignment vs. `=` in a comparison). `symbol_map` applies one font per character regardless of scope.
- **Approximation only:** The mapping is based on typical Unicode ranges associated with scope categories, not on actual token classification.
- **Bold/italic only for scope differentiation:** Weight and style changes via terminal escape codes remain the primary way to differentiate scopes in Helix itself.

## Setup

### Prerequisites

- Kitty terminal (v0.20+)
- polyfont CLI

### Step 1: Install the CLI

```bash
cargo install polyfont-cli
```

### Step 2: Create your polyfont config

Create `.polyfont.toml` in your project root or `~/.polyfont.toml`:

```toml
version = 1

[default]
family = "Fira Code"
fallbacks = ["JetBrains Mono", "monospace"]

[[rules]]
scope = "operator"
[rules.font]
family = "JetBrains Mono"

[[rules]]
scope = "punctuation"
[rules.font]
family = "Fira Code"

[[rules]]
scope = "string"
[rules.font]
family = "Source Code Pro"
weight = "light"
```

### Step 3: Generate Kitty config

```bash
polyfont kitty
```

This outputs `symbol_map` lines. For example:

```
symbol_map U+2200-U+22FF JetBrains Mono
symbol_map U+2190-U+21FF Fira Code
symbol_map U+2500-U+257F Fira Code
symbol_map U+25A0-U+25FF JetBrains Mono
```

### Step 4: Add to Kitty config

Append the output to your Kitty configuration file at `~/.config/kitty/kitty.conf`:

```bash
polyfont kitty >> ~/.config/kitty/kitty.conf
```

Or copy specific lines manually.

### Step 5: Restart Kitty and run Helix

```bash
kitty +kitten @ launch --type=tab hx
```

Or simply open a new Kitty terminal and launch Helix normally.

## Known Limitations

| Limitation | Detail |
|------------|--------|
| Not true per-scope | Characters are mapped by Unicode range, not by syntax scope |
| One font per character | A character always uses the same font regardless of context |
| Scope coverage is approximate | Not all scope-to-range mappings have clear Unicode signatures |
| Only works in Kitty | Other terminals (Alacritty, WezTerm, etc.) use different config formats |
| Requires font installation | Mapped fonts must be installed on your system |
| No weight/style per map | `symbol_map` only controls font family, not weight or italic |

## WezTerm Alternative

If you use WezTerm instead of Kitty, WezTerm has its own `font_rules` system that can achieve similar Unicode-range-based font switching. The `polyfont kitty` output can be adapted to WezTerm's `harfbuzz_features` or `font_rules` format, but this is not currently automated.
