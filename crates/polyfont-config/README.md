# polyfont-config

TOML configuration loading and validation for polyfont.

## Types

- `PolyfontConfig` — top-level configuration structure
- `FontConfig` — per-font family settings
- `RuleConfig` — scope-to-font mapping rules

## Usage

```rust
use polyfont_config::PolyfontConfig;

let config = PolyfontConfig::load_file("polyfont.toml")?;
for rule in &config.rules {
    println!("{} -> {}", rule.scope, rule.font);
}
```

License: Apache-2.0
