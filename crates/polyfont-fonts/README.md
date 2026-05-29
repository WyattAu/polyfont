# polyfont-fonts

Cross-platform font discovery and management for polyfont.

Provides a unified interface for discovering installed fonts across
Linux (`fc-list`), macOS (`system_profiler`), and Windows (PowerShell).

## Features

- `download` — enables font downloading via HTTP (requires `reqwest`, `sha2`, `zip`)

## Usage

```rust
use polyfont_fonts::FontDiscovery;

let discovery = polyfont_fonts::system_discovery();
let fonts = discovery.list_fonts()?;
let mono = fonts.iter().find(|f| f.monospace);
```

License: Apache-2.0
