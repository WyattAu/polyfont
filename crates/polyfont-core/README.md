# polyfont-core

Core types and engine for per-token font resolution in syntax-highlighted code.

## Types

- `TokenInfo` — a text token with position and scope
- `FontSpec` — font specification (family, weight, style, axes)
- `FontRule` — maps a scope pattern to a font
- `ScopeMatchEngine` — resolves tokens to font assignments

## Usage

```rust
use polyfont_core::{FontSpec, FontRule, ScopeMatchEngine, TokenInfo};

let engine = ScopeMatchEngine::from_rules(vec![
    FontRule { scope: "keyword".into(), font: FontSpec::default_font("Maple Mono") },
]);

let token = TokenInfo {
    text: "fn".into(),
    range: Default::default(),
    scope: "keyword".into(),
    modifiers: vec![],
};

let assignment = engine.resolve_token(&token);
assert!(assignment.is_some());
```

License: Apache-2.0
