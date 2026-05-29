//! Custom multi-font rendering engine for polyfont.
//!
//! Composites text from multiple font families into a single output.
//!
//! # Features
//!
//! - `gpu` -- Enable GPU-accelerated rendering (requires Vulkan/Metal/DX12)
//! - `software` -- Enable CPU-based software rendering
//!
//! # Status
//!
//! This is a skeleton crate. Both `gpu` and `software` features are
//! placeholder feature flags; no rendering backends are wired yet.
//! The rendering pipeline is under development (see ROADMAP_v2.md).

use std::collections::HashSet;

use polyfont_core::PolyfontEngine;
use thiserror::Error;
use tracing::info;

/// Rendering errors.
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("no rendering backend available")]
    NoBackend,

    #[error("font atlas error: {0}")]
    FontAtlas(String),

    #[error("shaping error: {0}")]
    Shaping(String),

    #[error("compositing error: {0}")]
    Compositing(String),

    #[error("GPU initialization failed: {0}")]
    GpuInit(String),

    #[error("window creation failed: {0}")]
    Window(String),
}

/// A glyph position in the rendered output.
#[derive(Debug, Clone)]
pub struct GlyphPosition {
    /// Font family this glyph belongs to.
    pub font_family: String,
    /// X position in pixels.
    pub x: f32,
    /// Y position (baseline) in pixels.
    pub y: f32,
    /// Glyph ID in the font.
    pub glyph_id: u32,
    /// Advance width.
    pub advance: f32,
}

/// A rendered line of multi-font text.
#[derive(Debug, Clone)]
pub struct RenderedLine {
    /// Glyphs in this line, each potentially from a different font.
    pub glyphs: Vec<GlyphPosition>,
    /// Total line width in pixels.
    pub width: f32,
    /// Line height in pixels.
    pub height: f32,
}

/// Configuration for the rendering engine.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Viewport width in pixels.
    pub width: u32,
    /// Viewport height in pixels.
    pub height: u32,
    /// Font size in points.
    pub font_size: f32,
    /// Line height multiplier (1.0 = tight, 1.5 = comfortable).
    pub line_height: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            font_size: 14.0,
            line_height: 1.4,
        }
    }
}

/// Font atlas management.
///
/// Maintains a cache of rasterized glyphs per font family.
/// In GPU mode, these are uploaded as textures. In software mode,
/// they are kept as alpha masks.
pub struct FontAtlas {
    families: HashSet<String>,
}

impl FontAtlas {
    /// Create a new empty font atlas.
    #[must_use]
    pub fn new() -> Self {
        info!("initializing font atlas");
        Self {
            families: HashSet::new(),
        }
    }

    /// Register a font family for use in rendering.
    pub fn register_family(&mut self, family: &str) {
        if !self.families.contains(family) {
            info!(family, "registering font family in atlas");
            self.families.insert(family.to_string());
        }
    }

    /// List all registered font families.
    #[must_use]
    pub fn registered_families(&self) -> Vec<&String> {
        let mut families: Vec<&String> = self.families.iter().collect();
        families.sort();
        families
    }

    /// Clear all cached glyphs and registered families.
    pub fn clear(&mut self) {
        self.families.clear();
    }
}

impl Default for FontAtlas {
    fn default() -> Self {
        Self::new()
    }
}

/// Scope-to-font annotator.
///
/// Maps token scopes to font families using the polyfont engine.
pub struct ScopeAnnotator {
    engine: polyfont_core::ScopeMatchEngine,
}

impl ScopeAnnotator {
    /// Create a new annotator from font rules.
    #[must_use]
    pub fn new(engine: polyfont_core::ScopeMatchEngine) -> Self {
        Self { engine }
    }

    /// Annotate a token with its assigned font family.
    #[must_use]
    pub fn annotate(&self, token: &polyfont_core::TokenInfo) -> Option<String> {
        self.engine
            .resolve_token(token)
            .map(|a| a.font.family.clone())
    }
}

/// The main rendering engine.
///
/// Coordinates font atlas management, text shaping, and compositing.
pub struct RenderEngine {
    atlas: FontAtlas,
    config: RenderConfig,
}

impl RenderEngine {
    /// Create a new rendering engine with the given configuration.
    #[must_use]
    pub fn new(config: RenderConfig) -> Self {
        info!(
            width = config.width,
            height = config.height,
            font_size = config.font_size,
            "initializing render engine"
        );
        Self {
            atlas: FontAtlas::new(),
            config,
        }
    }

    /// Register a font family for rendering.
    pub fn register_font(&mut self, family: &str) {
        self.atlas.register_family(family);
    }

    /// Get the current configuration.
    #[must_use]
    pub fn config(&self) -> &RenderConfig {
        &self.config
    }

    /// Render a sequence of tokens into positioned glyphs.
    ///
    /// Returns `Err(RenderError::NoBackend)` until a rendering backend is initialized.
    pub fn render_tokens(
        &self,
        tokens: &[polyfont_core::TokenInfo],
        annotator: &ScopeAnnotator,
    ) -> Result<Vec<RenderedLine>, RenderError> {
        let mut lines: Vec<RenderedLine> = Vec::new();
        let mut current_glyphs: Vec<GlyphPosition> = Vec::new();
        let mut current_x = 0.0_f32;
        let mut current_line = 0_u32;

        let line_height = self.config.font_size * self.config.line_height;

        for token in tokens {
            if token.range.start.line != current_line && !current_glyphs.is_empty() {
                let width = current_x;
                lines.push(RenderedLine {
                    glyphs: std::mem::take(&mut current_glyphs),
                    width,
                    height: line_height,
                });
                current_x = 0.0;
                current_line = token.range.start.line;
            }

            let family = annotator
                .annotate(token)
                .unwrap_or_else(|| "monospace".to_string());

            // Approximate glyph positions using character count.
            // Real implementation would use cosmic-text or harfbuzz shaping.
            let char_count = token.text.chars().count() as f32;
            let advance = char_count * self.config.font_size * 0.6;

            current_glyphs.push(GlyphPosition {
                font_family: family,
                x: current_x,
                y: (token.range.start.line as f32 + 1.0) * line_height,
                glyph_id: 0,
                advance,
            });
            current_x += advance;
        }

        if !current_glyphs.is_empty() {
            lines.push(RenderedLine {
                glyphs: current_glyphs,
                width: current_x,
                height: line_height,
            });
        }

        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polyfont_core::{FontRule, FontSpec, Position, Range, TokenInfo};

    #[test]
    fn test_font_atlas_register() {
        let mut atlas = FontAtlas::new();
        atlas.register_family("Test");
        atlas.register_family("Test"); // duplicate ignored
        assert_eq!(atlas.registered_families().len(), 1);
    }

    #[test]
    fn test_scope_annotator() {
        let engine = polyfont_core::ScopeMatchEngine::from_rules(vec![FontRule {
            scope: "keyword".to_string(),
            font: FontSpec::default_font("Maple Mono"),
        }]);
        let annotator = ScopeAnnotator::new(engine);
        let token = TokenInfo {
            text: "fn".to_string(),
            range: Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 2 },
            },
            scope: "keyword".to_string(),
            modifiers: vec![],
        };
        assert_eq!(annotator.annotate(&token), Some("Maple Mono".to_string()));
    }

    #[test]
    fn test_render_engine_tokens() {
        let engine = polyfont_core::ScopeMatchEngine::from_rules(vec![FontRule {
            scope: "*".to_string(),
            font: FontSpec::default_font("Mono"),
        }]);
        let annotator = ScopeAnnotator::new(engine);
        let render = RenderEngine::new(RenderConfig::default());

        let tokens = vec![
            TokenInfo {
                text: "fn".to_string(),
                range: Range {
                    start: Position { line: 0, column: 0 },
                    end: Position { line: 0, column: 2 },
                },
                scope: "keyword".to_string(),
                modifiers: vec![],
            },
            TokenInfo {
                text: "main".to_string(),
                range: Range {
                    start: Position { line: 0, column: 3 },
                    end: Position { line: 0, column: 7 },
                },
                scope: "entity.name.function".to_string(),
                modifiers: vec![],
            },
        ];

        let lines = render.render_tokens(&tokens, &annotator).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].glyphs.len(), 2);
    }

    #[test]
    fn test_render_config_default() {
        let config = RenderConfig::default();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!((config.font_size - 14.0).abs() < f32::EPSILON);
    }
}
