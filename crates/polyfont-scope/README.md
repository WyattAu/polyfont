# polyfont-scope

TextMate scope matching and resolution engine for polyfont.

## Types

- `ScopeMatcher` — matches token scopes against TextMate scope selectors
- `ScopeSelector` — parsed representation of a TextMate scope pattern
- `ScopeResolver` — resolves the best-matching rule for a given token

## Usage

```rust
use polyfont_scope::ScopeMatcher;

let matcher = ScopeMatcher::new("source.rust meta.function keyword");
assert!(matcher.matches("keyword"));
assert!(matcher.matches("meta.function"));
```

License: Apache-2.0
