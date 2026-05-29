# polyfont-themes

Theme import, export, and built-in registry for polyfont.

Provides importers for VSCode JSON and TextMate tmTheme formats,
exporters for polyfont TOML and VSCode settings, and a registry of
5 built-in themes.

## Features

- `download` — enables theme downloading via HTTP

## Usage

```rust
use polyfont_themes::{ThemeRegistry, ThemeImporter};

let registry = ThemeRegistry::builtin();
let theme = registry.get("one-dark")?;

let imported = ThemeImporter::import_vscode_json("path/to/theme.json")?;
```

License: Apache-2.0
