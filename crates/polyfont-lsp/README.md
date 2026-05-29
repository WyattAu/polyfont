# polyfont-lsp

LSP server providing per-token font assignments for polyfont-aware editors.

## Features

- Custom notification: `polyfont/fontAssignments`
- Custom request: `polyfont/requestFontAssignments`
- Custom request: `polyfont/suggestFonts`
- Tree-sitter tokenization with naive fallback
- UTF-16 position encoding (LSP spec compliant)
- Debounced document synchronization

## Running

```bash
polyfont-lsp
```

License: Apache-2.0
