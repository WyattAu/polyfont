# Polyfont for Zed Editor

> **Placeholder — Not Functional**
> This extension is a non-functional stub. Zed's extension API does not yet
> expose per-scope `font_family` overrides. The code compiles but returns an
> error for every command. Do not install this expecting working behaviour.

## Current Status

Zed does not yet support per-scope font families in its stable public API. Zed's GPUI rendering engine is architecturally capable of per-highlight font family assignment, but the extension API does not currently expose per-scope `font_family` overrides as a stable feature.

## Planned Integration

When Zed's theme system exposes per-scope font settings, polyfont will ship a Zed extension that:

1. Reads `.polyfont.toml` from the project root or home directory
2. Maps polyfont scope rules to Zed's `text_style_overrides` syntax
3. Generates a Zed theme JSON with font family assignments per syntax scope

Track progress in the [v0.7 roadmap](../../ROADMAP.md).

## Workaround: Generate Theme JSON Manually

You can use the `polyfont vscode` CLI command to generate JSON font override entries. The output uses the same TextMate scope selectors that Zed's theme system understands, so the entries can be adapted into a Zed theme file.

### Setup

1. Install the polyfont CLI:

```bash
cargo install polyfont-cli
```

2. Create `.polyfont.toml` in your project root or `~/.polyfont.toml`:

```toml
version = 1

[default]
family = "Fira Code"
fallbacks = ["JetBrains Mono", "monospace"]

[[rules]]
scope = "keyword"
[rules.font]
family = "Maple Mono"
weight = "bold"

[[rules]]
scope = "entity.name.function"
[rules.font]
family = "Monaspace Argon"
weight = "semi-bold"

[[rules]]
scope = "comment"
[rules.font]
family = "IBM Plex Mono"
style = "italic"
```

3. Generate the override JSON:

```bash
polyfont vscode
```

This outputs an array of `textMateRules` entries with `scope`, `settings.foreground`, and `settings.fontFamily` fields.

4. Adapt the output for Zed. Zed themes use a flat dictionary keyed by scope name. Transform each rule into Zed theme format. For example, a VSCode rule:

```json
{
  "scope": "keyword",
  "settings": {
    "fontFamily": "Maple Mono Bold",
    "foreground": "#c678dd"
  }
}
```

becomes a Zed theme entry (when per-scope font support lands):

```json
{
  "keyword": {
    "font_family": "Maple Mono",
    "font_weight": 700
  }
}
```

5. Place the resulting theme JSON in your Zed settings. In `~/.config/zed/settings.json`:

```json
{
  "theme_overrides": {
    // Paste adapted entries here once Zed supports them
  }
}
```

### Checking for Updates

Zed moves fast. Check the Zed changelog or `zed --version` for updates to per-scope font support. Once available, the manual JSON adaptation step will be replaced by a `polyfont zed` CLI command.
