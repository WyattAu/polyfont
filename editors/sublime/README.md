# Polyfont for Sublime Text

## Overview

Sublime Text supports per-scope font settings via `.sublime-theme` files. Each entry in a `.sublime-theme` is a dictionary that can include a `font.face` key specifying the font family for a given scope. This makes Sublime Text a natural fit for polyfont's scope-to-font mapping.

## How It Works

A `.sublime-theme` file contains a list of dictionaries, each targeting a UI element or syntax scope. The relevant entries use `class`, `scope`, and `settings` keys:

```json
[
    {
        "class": "invisible",
        "scope": "keyword",
        "settings": {
            "font.face": "Maple Mono Bold"
        }
    },
    {
        "class": "invisible",
        "scope": "entity.name.function",
        "settings": {
            "font.face": "Monaspace Argon"
        }
    },
    {
        "class": "invisible",
        "scope": "comment",
        "settings": {
            "font.face": "IBM Plex Mono Italic"
        }
    }
]
```

The `class: "invisible"` entries apply to the text area without changing any visual attribute other than what is specified in `settings`.

## Generating a `.sublime-theme` Override

### Using the CLI

The `polyfont vscode` command generates JSON output with TextMate scope selectors and font assignments. The scope selectors are compatible with Sublime Text's scope system, and the font family names can be extracted and placed into `.sublime-theme` format.

```bash
polyfont vscode
```

This outputs an array of rules like:

```json
{
    "scope": "keyword",
    "settings": {
        "fontFamily": "Maple Mono",
        "foreground": "#c678dd"
    }
}
```

### Manual Setup

1. Create your polyfont config (`.polyfont.toml` in your project root or `~/.polyfont.toml`):

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
scope = "entity.name.function"
[rules.font]
family = "Monaspace Argon"
weight = "semi-bold"

[[rules]]
scope = "comment"
[rules.font]
family = "IBM Plex Mono"
style = "italic"

[[rules]]
scope = "string"
[rules.font]
family = "Source Code Pro"
weight = "light"

[[rules]]
scope = "variable"
[rules.font]
family = "JetBrains Mono"
```

2. Create a `.sublime-theme` file. Sublime Text loads theme files from `Packages/User/`. Create `Packages/User/Polyfont.sublime-theme` (find your packages directory via **Preferences > Browse Packages**).

3. Write the theme entries. For each rule in your polyfont config, create a dictionary entry. Convert polyfont weight/style into the font face name since `font.face` accepts the full font face string (including weight and style):

```json
[
    {
        "class": "invisible",
        "scope": "keyword",
        "settings": {
            "font.face": "Maple Mono Bold"
        }
    },
    {
        "class": "invisible",
        "scope": "keyword.control",
        "settings": {
            "font.face": "Maple Mono Bold"
        }
    },
    {
        "class": "invisible",
        "scope": "entity.name.function",
        "settings": {
            "font.face": "Monaspace Argon SemiBold"
        }
    },
    {
        "class": "invisible",
        "scope": "comment",
        "settings": {
            "font.face": "IBM Plex Mono Italic"
        }
    },
    {
        "class": "invisible",
        "scope": "string",
        "settings": {
            "font.face": "Source Code Pro Light"
        }
    },
    {
        "class": "invisible",
        "scope": "variable",
        "settings": {
            "font.face": "JetBrains Mono"
        }
    }
]
```

4. Activate the theme. In `Preferences > Settings`, set:

```json
{
    "theme": "Polyfont.sublime-theme"
}
```

Or keep your existing color theme and overlay the font settings by placing the entries in a file that Sublime merges automatically.

5. Restart Sublime Text.

## Font Face Naming

The `font.face` value must match the font face name registered on your system, including weight and style. Common patterns:

| Polyfont Config | `font.face` Value |
|-----------------|-------------------|
| `family = "Fira Code"` | `"Fira Code"` |
| `family = "Maple Mono", weight = "bold"` | `"Maple Mono Bold"` |
| `family = "IBM Plex Mono", style = "italic"` | `"IBM Plex Mono Italic"` |
| `family = "Source Code Pro", weight = "light"` | `"Source Code Pro Light"` |
| `family = "Monaspace Argon", weight = "semi-bold"` | `"Monaspace Argon SemiBold"` |

Use your system's font viewer to confirm the exact face names.

## Known Limitations

| Limitation | Detail |
|------------|--------|
| Font face name sensitivity | The `font.face` value must exactly match the installed font face name |
| Per-theme, not per-file | Font overrides are tied to the active theme, not to individual files or projects |
| Theme merging | Setting a custom theme replaces your color theme; you may need to duplicate color settings |
| No dynamic reload | Theme changes require a restart or window reload (Ctrl+Shift+R) |
| Limited weight/style control | `font.face` only accepts the full face name; there is no separate weight/style key in theme entries |
| Scope inheritance | Sublime Text applies the most specific matching scope, consistent with TextMate scope resolution |
