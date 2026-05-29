# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.10.x  | Yes       |
| < 0.10   | No        |

## Reporting a Vulnerability

If you discover a security vulnerability in polyfont, please report it responsibly:

1. **Do not** open a public GitHub issue for security vulnerabilities.
2. Send an email to the maintainers by filing a [GitHub Security Advisory](https://github.com/WyattAu/polyfont/security/advisories/new).
3. Include a description of the vulnerability, steps to reproduce, and any potential impact.
4. We will acknowledge receipt within 48 hours and provide an estimated timeline for a fix.

## Disclosure Timeline

- **48 hours:** Initial response and acknowledgment
- **7 days:** Assessment and severity classification (Critical/High/Medium/Low)
- **30 days:** Fix for Critical/High vulnerabilities
- **90 days:** Fix for Medium/Low vulnerabilities (or documented exception with justification)

## Dependencies

polyfont depends on the following major external libraries:

- **tree-sitter** (C library, feature-gated): Used for syntax parsing. Source commits are pinned in Cargo.lock.
- **tower-lsp**: LSP server framework. Audited as part of tower ecosystem.
- **tokio**: Async runtime. Maintained by the Tokio team.
- **serde/serde_json**: Serialization. Maintained by the serde team.

All Rust dependencies are sourced from crates.io and validated via `cargo audit` and `cargo deny` in CI.

## Supply Chain

- All dependencies are pinned via `Cargo.lock`
- No binary dependencies; all code is compiled from source
- Tree-sitter grammars are compiled from tagged upstream releases
- CI runs `cargo audit` and `cargo deny check` on every push
