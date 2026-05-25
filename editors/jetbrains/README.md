# Polyfont IntelliJ Plugin

Per-token font highlighting for IntelliJ IDEA.

## Prerequisites

- JDK 17
- IntelliJ IDEA 2024.1+

## Build

```sh
./gradlew buildPlugin
```

The plugin ZIP is written to `build/distributions/`.

## Install

Copy the ZIP to the IntelliJ plugins directory and restart:

```sh
cp build/distributions/polyfont-*.zip ~/.local/share/JetBrains/ IdeaIC2024.1/plugins/
```

Or install from disk via Settings > Plugins > gear icon > Install Plugin from Disk.

## Usage

1. Open a project containing a `.polyfont.toml` config file.
2. Right-click in the editor and select **Apply Polyfont Fonts**.
3. A notification confirms how many rules were applied.
4. Configure additional rules via Settings > Appearance > Polyfont.

## Config Format

`.polyfont.toml` example:

```toml
[[rules]]
scope = "keyword"
font.family = "JetBrains Mono"

[[rules]]
scope = "string"
font.family = "Fira Code"
```

## Limitations

IntelliJ applies one font per editor via `EditorColorsScheme`. Per-token font rendering requires a custom editor renderer that hooks into the text drawing pipeline, which is not yet implemented in this skeleton. The current implementation registers `TextAttributesKey` entries per scope, which can carry foreground color and effects but not a true font-family override per token.
