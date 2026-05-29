# polyfont-parse

Tree-sitter-based tokenization for polyfont.

Produces per-token scope classifications using tree-sitter grammars,
enabling accurate syntax highlighting for the polyfont LSP server.

## Supported Languages

Enable via cargo features: `rust`, `typescript`, `javascript`, `python`, `go`, `c`, `cpp`, `json`, `toml`, `lua`.

## Usage

```rust
use polyfont_parse::TreeSitterParser;

let parser = TreeSitterParser::for_language("rust")?;
let tokens = parser.tokenize("fn main() {}")?;
for token in &tokens {
    println!("{}: {}", token.scope, token.text);
}
```

License: Apache-2.0
