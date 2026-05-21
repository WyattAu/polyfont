# Polyfont Requirements Specification

## 1. Problem Statement

Code editors support per-token **color** highlighting but not per-token **font** highlighting.
Developers who want visual differentiation beyond color (e.g., monospace for code, sans-serif
for comments, serif for strings, decorative for keywords) have no solution.

## 2. Stakeholders

| Stakeholder | Role | Concerns | Priority |
|-------------|------|----------|----------|
| Developer (primary) | End user | Easy config, works in their editor | H |
| Theme author | Config author | Declarative format, portability | H |
| Editor integrator | Plugin developer | Clean API, extensibility | M |

## 3. Functional Requirements (EARS Format)

### FR-001: Font Rule Configuration
**When** a user provides a polyfont configuration file,
**the system shall** map syntax scopes (TextMate scopes or Tree-sitter captures) to font families,
**so that** each token type can be rendered in a specified font.

### FR-002: VSCode Integration
**When** the polyfont VSCode extension is active and a supported file is open,
**the system shall** apply per-token font families via `editor.tokenColorCustomizations.textMateRules`,
**so that** tokens are rendered with the configured font in VSCode 1.97+.

### FR-003: Neovim Integration
**When** the polyfont Neovim plugin is active and a supported file is open,
**the system shall** generate Neovim highlight groups with font metadata and communicate
font mappings to supporting GUIs (Goneovim, Neovide with patches),
**so that** tokens can be rendered with different fonts in supporting Neovim GUIs.

### FR-004: Kitty Terminal Integration
**When** the polyfont CLI is used with Kitty terminal,
**the system shall** generate `symbol_map` configurations mapping Unicode Private Use Area
ranges to specified fonts per token type,
**so that** per-token font rendering works in Kitty terminal via Unicode remapping.

### FR-005: Configuration File Format
**When** a user creates a `.polyfont.toml` file in their project root or home directory,
**the system shall** parse font rules with syntax scope selectors, font family names,
fallback fonts, and optional weight/style/size overrides,
**so that** configuration is declarative and portable.

### FR-006: Font Validation
**When** the system loads a font rule referencing a font family,
**the system shall** validate the font is available on the system and report missing fonts
with actionable error messages,
**so that** users get immediate feedback on misconfiguration.

### FR-007: Scope Resolution
**When** multiple font rules match a token's scope,
**the system shall** apply the most specific rule (longest scope match),
**so that** granular overrides work correctly (e.g., `entity.name.function` beats `entity`).

### FR-008: Live Reload
**When** a `.polyfont.toml` configuration file is modified,
**the system shall** reload and reapply font rules within 500ms,
**so that** users see changes without restarting the editor.

## 4. Non-Functional Requirements

| ID | Category | Requirement | Metric |
|----|----------|-------------|--------|
| NFR-001 | Performance | Token processing latency | < 5ms per 1000 tokens |
| NFR-002 | Performance | Config parse time | < 50ms |
| NFR-003 | Compatibility | VSCode version | >= 1.97 (for fontFamily in textMateRules) |
| NFR-004 | Compatibility | Neovim version | >= 0.9 |
| NFR-005 | Compatibility | Kitty version | >= 0.35 |
| NFR-006 | Portability | Platforms | Linux, macOS, Windows |
| NFR-007 | Usability | Setup time | < 5 minutes |
| NFR-008 | Reliability | Zero data loss | No file modification |

## 5. MoSCoW Priority Matrix

| Priority | Requirements |
|----------|-------------|
| MUST | FR-001, FR-002, FR-005, FR-007 |
| SHOULD | FR-003, FR-004, FR-006 |
| COULD | FR-008 |
| WON'T (v0.1) | Helix/Zed integration, custom GUI, image-based rendering |

## 6. Acceptance Criteria

- [ ] A `.polyfont.toml` file can map TextMate scopes to font families
- [ ] VSCode extension applies fonts per-token via textMateRules (requires VSCode 1.97+)
- [ ] Neovim plugin generates highlight groups and font metadata
- [ ] CLI tool validates configuration and reports errors
- [ ] Scope specificity resolution works correctly (most specific match wins)
- [ ] System handles missing fonts gracefully with clear error messages
