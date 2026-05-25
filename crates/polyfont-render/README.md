# polyfont-render

Custom multi-font rendering engine for polyfont. Composites text from multiple font
families into a single output using GPU-accelerated or software rendering.

## Status

This is a **skeleton crate**. The rendering engine requires significant development
effort and depends on `wgpu`/`cosmic-text`/`glyphon` (GPU) or `tiny-skia`/`softbuffer`
(software) which are heavy dependencies. Enable features explicitly:

```toml
polyfont-render = { version = "0.9.0", features = ["gpu"] }
```

## Architecture (Planned)

1. **FontAtlas** -- maintains a glyph atlas per font family, GPU-uploaded texture
2. **ShapingEngine** -- uses `cosmic-text` or `parley` for text shaping
3. **ScopeAnnotator** -- maps token scopes to font families from config
4. **Compositor** -- renders multi-font text lines using `glyphon` or `tiny-skia`
5. **TerminalMode** -- intercepts terminal output for terminal multiplexer use
