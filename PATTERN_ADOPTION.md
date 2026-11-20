# Pattern adoption

Declared per the pattern canon adoption format; every claim is verified against code in this repo.

| Pattern | Status | Evidence |
|---|---|---|
| T13 | adopted | crates/polyfont-core/src/engine.rs:8 (pure resolution core; the cli and lsp crates are shells) |
| T1 | adopted | crates/polyfont-parse/src/lib.rs:26 (source parsed into typed tokens; matching never re-reads raw text) |
