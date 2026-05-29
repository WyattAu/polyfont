//! Software rendering engine for per-token multi-font text output.
//!
//! Accepts annotated tokens (with font assignments) and renders them to SVG
//! or (with the `software` feature) PNG. Uses `fontdb` for font discovery
//! and metrics extraction.
//!
//! # Features
//!
//! - `software` -- Enable CPU-based font metrics via `fontdb` and PNG rendering
//!   via a built-in bitmap font.
//! - `gpu` -- Placeholder for future GPU-accelerated rendering.
//!
//! # Example (SVG output)
//!
//! ```
//! use polyfont_render::{RenderEngine, RenderConfig, RenderOutput};
//!
//! let engine = RenderEngine::new(RenderConfig::default()).unwrap();
//! let output = engine.render_tokens_svg(&[]);
//! match output {
//!     RenderOutput::Svg(svg) => assert!(svg.contains("<svg")),
//!     _ => unreachable!(),
//! }
//! ```

use std::collections::HashMap;

use polyfont_core::{FontSpec, PolyfontEngine, TokenInfo};
use thiserror::Error;
use tracing::info;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Rendering errors.
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("no rendering backend available")]
    NoBackend,

    #[error("font cache error: {0}")]
    FontCache(String),

    #[error("metrics extraction failed for {family}: {reason}")]
    Metrics { family: String, reason: String },

    #[error("shaping error: {0}")]
    Shaping(String),

    #[error("compositing error: {0}")]
    Compositing(String),

    #[error("GPU initialization failed: {0}")]
    GpuInit(String),

    #[error("window creation failed: {0}")]
    Window(String),

    #[error("software path not enabled; compile with --features software")]
    SoftwarePathNotEnabled,

    #[error("invalid font size: {0}")]
    InvalidFontSize(f32),

    #[error("invalid configuration: {0}")]
    Config(String),
}

// ---------------------------------------------------------------------------
// RenderConfig
// ---------------------------------------------------------------------------

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
    /// Default font family used when no assignment is present.
    pub default_font_family: String,
    /// Background color for PNG output (hex RGB, e.g. "1e1e2e").
    pub background_color: String,
    /// Foreground color for PNG output (hex RGB, e.g. "cdd6f4").
    pub foreground_color: String,
    /// Left padding in pixels.
    pub padding_x: f32,
    /// Top padding in pixels.
    pub padding_y: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            font_size: 14.0,
            line_height: 1.4,
            default_font_family: "monospace".to_string(),
            background_color: "1e1e2e".to_string(),
            foreground_color: "cdd6f4".to_string(),
            padding_x: 10.0,
            padding_y: 10.0,
        }
    }
}

impl RenderConfig {
    /// Create a new builder-style config with defaults.
    #[must_use]
    pub fn builder() -> Self {
        Self::default()
    }

    /// Set font size and return self.
    #[must_use]
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set line height multiplier and return self.
    #[must_use]
    pub fn with_line_height(mut self, lh: f32) -> Self {
        self.line_height = lh;
        self
    }

    /// Set viewport dimensions and return self.
    #[must_use]
    pub fn with_viewport(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set default font family and return self.
    #[must_use]
    pub fn with_default_font(mut self, family: &str) -> Self {
        self.default_font_family = family.to_string();
        self
    }

    /// Compute pixel line height from font size and multiplier.
    #[must_use]
    pub fn pixel_line_height(&self) -> f32 {
        self.font_size * self.line_height
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), RenderError> {
        if self.font_size <= 0.0 {
            return Err(RenderError::InvalidFontSize(self.font_size));
        }
        if self.line_height <= 0.0 {
            return Err(RenderError::Config("line_height must be positive".into()));
        }
        if self.width == 0 || self.height == 0 {
            return Err(RenderError::Config(
                "viewport dimensions must be nonzero".into(),
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// FontMetrics
// ---------------------------------------------------------------------------

/// Per-font metrics extracted from font tables or estimated.
#[derive(Debug, Clone)]
pub struct FontMetrics {
    /// Font family name.
    pub family: String,
    /// Ascent in design units (positive value above baseline).
    pub ascent: f32,
    /// Descent in design units (positive value below baseline).
    pub descent: f32,
    /// Line gap in design units.
    pub line_gap: f32,
    /// Units per em of the font.
    pub units_per_em: u16,
}

impl FontMetrics {
    /// Create metrics for a monospace fallback font.
    #[must_use]
    pub fn monospace_fallback(family: &str) -> Self {
        Self {
            family: family.to_string(),
            ascent: 12.0,
            descent: 3.0,
            line_gap: 0.0,
            units_per_em: 16,
        }
    }

    /// Scale ascent from design units to pixel size.
    #[must_use]
    pub fn ascent_px(&self, font_size: f32) -> f32 {
        if self.units_per_em == 0 {
            return 0.0;
        }
        self.ascent * font_size / self.units_per_em as f32
    }

    /// Scale descent from design units to pixel size.
    #[must_use]
    pub fn descent_px(&self, font_size: f32) -> f32 {
        if self.units_per_em == 0 {
            return 0.0;
        }
        self.descent * font_size / self.units_per_em as f32
    }

    /// Scale line gap from design units to pixel size.
    #[must_use]
    pub fn line_gap_px(&self, font_size: f32) -> f32 {
        if self.units_per_em == 0 {
            return 0.0;
        }
        self.line_gap * font_size / self.units_per_em as f32
    }

    /// Approximate character width for a monospace font at the given pixel size.
    #[must_use]
    pub fn char_width_px(&self, font_size: f32) -> f32 {
        if self.units_per_em == 0 {
            return font_size * 0.6;
        }
        // Assume em-width roughly 60% of font_size for monospace.
        font_size * 0.6
    }

    /// Approximate text width for a string.
    #[must_use]
    pub fn text_width_px(&self, text: &str, font_size: f32) -> f32 {
        text.chars().count() as f32 * self.char_width_px(font_size)
    }
}

// ---------------------------------------------------------------------------
// FontCache
// ---------------------------------------------------------------------------

/// Caches font family metrics. Uses `fontdb` when the `software` feature is
/// enabled; otherwise uses estimated metrics.
pub struct FontCache {
    metrics: HashMap<String, FontMetrics>,
}

impl FontCache {
    /// Create a new empty font cache.
    #[must_use]
    pub fn new() -> Self {
        info!("initializing font cache");
        Self {
            metrics: HashMap::new(),
        }
    }

    /// Register a font family with a custom `FontMetrics`.
    pub fn register_family(&mut self, family: &str, metrics: FontMetrics) {
        info!(family, "registering font family in cache");
        self.metrics.insert(family.to_string(), metrics);
    }

    /// Register a font family with fallback metrics.
    pub fn register_family_fallback(&mut self, family: &str) {
        if !self.metrics.contains_key(family) {
            self.register_family(family, FontMetrics::monospace_fallback(family));
        }
    }

    /// Look up metrics for a family. Returns `None` if not registered.
    #[must_use]
    pub fn get(&self, family: &str) -> Option<&FontMetrics> {
        self.metrics.get(family)
    }

    /// Look up metrics or return a default for unknown families.
    #[must_use]
    pub fn get_or_default(&self, family: &str) -> FontMetrics {
        self.get(family)
            .cloned()
            .unwrap_or_else(|| FontMetrics::monospace_fallback(family))
    }

    /// Return all registered family names.
    #[must_use]
    pub fn families(&self) -> Vec<&str> {
        self.metrics.keys().map(String::as_str).collect()
    }

    /// Return the number of registered families.
    #[must_use]
    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    /// Return true if no families are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }

    /// Clear all cached metrics.
    pub fn clear(&mut self) {
        self.metrics.clear();
    }
}

impl Default for FontCache {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// BaselineAligner
// ---------------------------------------------------------------------------

/// Computes Y offsets to align glyphs from different fonts on a common baseline.
pub struct BaselineAligner;

impl BaselineAligner {
    /// Given a slice of font metrics for the active fonts on a line,
    /// compute the maximum ascent and descent (in pixels) and return per-family
    /// Y offsets so all glyphs share the same baseline.
    ///
    /// Returns a map from family name to (y_offset, baseline_y) where:
    /// - `y_offset` is the distance from the top of the line to the font's baseline
    /// - `baseline_y` is the common baseline position from the top of the line
    #[must_use]
    pub fn compute_offsets(
        families: &[&FontMetrics],
        font_size: f32,
    ) -> HashMap<String, (f32, f32)> {
        let mut max_ascent = 0.0_f32;
        let mut max_descent = 0.0_f32;

        for m in families {
            let a = m.ascent_px(font_size);
            let d = m.descent_px(font_size);
            if a > max_ascent {
                max_ascent = a;
            }
            if d > max_descent {
                max_descent = d;
            }
        }

        let baseline_y = max_ascent;
        let mut offsets = HashMap::new();
        for m in families {
            let this_ascent = m.ascent_px(font_size);
            let y_offset = baseline_y - this_ascent;
            offsets.insert(m.family.clone(), (y_offset, baseline_y));
        }
        offsets
    }

    /// Compute the baseline offset for a single font (no alignment needed).
    #[must_use]
    pub fn single_font_baseline(metrics: &FontMetrics, font_size: f32) -> f32 {
        metrics.ascent_px(font_size)
    }
}

// ---------------------------------------------------------------------------
// GlyphPosition and RenderedLine (kept from original skeleton)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// RenderOutput
// ---------------------------------------------------------------------------

/// Render output format.
#[derive(Debug, Clone)]
pub enum RenderOutput {
    /// SVG string with per-token font styling.
    Svg(String),
    /// PNG pixel buffer (RGBA, width x height).
    Png {
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    },
}

// ---------------------------------------------------------------------------
// RasterizedGlyph (for software/PNG path)
// ---------------------------------------------------------------------------

/// A rasterized glyph with position and pixel data.
#[derive(Debug, Clone)]
pub struct RasterizedGlyph {
    /// Width of the glyph bitmap in pixels.
    pub width: u32,
    /// Height of the glyph bitmap in pixels.
    pub height: u32,
    /// Horizontal bearing (offset from origin to left of bitmap).
    pub bearing_x: i32,
    /// Vertical bearing (offset from baseline to top of bitmap).
    pub bearing_y: i32,
    /// Horizontal advance width.
    pub advance: f32,
    /// RGBA pixel data, row-major, top-to-bottom.
    pub pixels: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Bitmap font (software feature only)
// ---------------------------------------------------------------------------

#[cfg(feature = "software")]
mod bitmap_font {
    use super::RasterizedGlyph;

    /// 5x8 bitmap font for ASCII printable characters (0x20..=0x7E).
    /// Each character is 8 rows, each row is a u8 where bits 7..3 are the 5 columns
    /// (bit 7 = leftmost pixel, bit 3 = rightmost pixel).
    const GLYPHS: &[u8] = &[
        // ' ' (0x20)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // '!' (0x21)
        0x20, 0x20, 0x20, 0x20, 0x20, 0x00, 0x20, 0x00, // '"' (0x22)
        0x50, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // '#' (0x23)
        0x50, 0xF8, 0x50, 0xF8, 0x50, 0x00, 0x00, 0x00, // '$' (0x24)
        0x20, 0x78, 0xA0, 0x70, 0x28, 0xF0, 0x20, 0x00, // '%' (0x25)
        0xC0, 0xC8, 0x10, 0x20, 0x44, 0x8C, 0x00, 0x00, // '&' (0x26)
        0x40, 0xA0, 0xA0, 0x40, 0xA8, 0x90, 0x68, 0x00, // '\'' (0x27)
        0x20, 0x20, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, // '(' (0x28)
        0x10, 0x20, 0x40, 0x40, 0x40, 0x20, 0x10, 0x00, // ')' (0x29)
        0x40, 0x20, 0x10, 0x10, 0x10, 0x20, 0x40, 0x00, // '*' (0x2A)
        0x00, 0x88, 0x50, 0x20, 0x50, 0x88, 0x00, 0x00, // '+' (0x2B)
        0x00, 0x20, 0x20, 0xF8, 0x20, 0x20, 0x00, 0x00, // ',' (0x2C)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x20, 0x40, // '-' (0x2D)
        0x00, 0x00, 0x00, 0xF8, 0x00, 0x00, 0x00, 0x00, // '.' (0x2E)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, // '/' (0x2F)
        0x08, 0x08, 0x10, 0x20, 0x40, 0x80, 0x80, 0x00, // '0' (0x30)
        0x70, 0x88, 0x98, 0xA8, 0xC8, 0x88, 0x70, 0x00, // '1' (0x31)
        0x20, 0x60, 0x20, 0x20, 0x20, 0x20, 0x70, 0x00, // '2' (0x32)
        0x70, 0x88, 0x08, 0x10, 0x20, 0x40, 0xF8, 0x00, // '3' (0x33)
        0x70, 0x88, 0x08, 0x30, 0x08, 0x88, 0x70, 0x00, // '4' (0x34)
        0x10, 0x30, 0x50, 0x90, 0xF8, 0x10, 0x10, 0x00, // '5' (0x35)
        0xF8, 0x80, 0xF0, 0x08, 0x08, 0x88, 0x70, 0x00, // '6' (0x36)
        0x30, 0x40, 0x80, 0xF0, 0x88, 0x88, 0x70, 0x00, // '7' (0x37)
        0xF8, 0x08, 0x10, 0x10, 0x20, 0x20, 0x20, 0x00, // '8' (0x38)
        0x70, 0x88, 0x88, 0x70, 0x88, 0x88, 0x70, 0x00, // '9' (0x39)
        0x70, 0x88, 0x88, 0x78, 0x08, 0x10, 0x60, 0x00, // ':' (0x3A)
        0x00, 0x00, 0x20, 0x00, 0x00, 0x20, 0x00, 0x00, // ';' (0x3B)
        0x00, 0x00, 0x20, 0x00, 0x00, 0x20, 0x20, 0x40, // '<' (0x3C)
        0x10, 0x20, 0x40, 0x80, 0x40, 0x20, 0x10, 0x00, // '=' (0x3D)
        0x00, 0x00, 0xF8, 0x00, 0xF8, 0x00, 0x00, 0x00, // '>' (0x3E)
        0x40, 0x20, 0x10, 0x08, 0x10, 0x20, 0x40, 0x00, // '?' (0x3F)
        0x70, 0x88, 0x08, 0x10, 0x20, 0x00, 0x20, 0x00, // '@' (0x40)
        0x70, 0x88, 0x98, 0xA8, 0xA0, 0x80, 0x70, 0x00, // 'A' (0x41)
        0x20, 0x50, 0x88, 0x88, 0xF8, 0x88, 0x88, 0x00, // 'B' (0x42)
        0xF0, 0x88, 0x88, 0xF0, 0x88, 0x88, 0xF0, 0x00, // 'C' (0x43)
        0x70, 0x88, 0x80, 0x80, 0x80, 0x88, 0x70, 0x00, // 'D' (0x44)
        0xF0, 0x88, 0x88, 0x88, 0x88, 0x88, 0xF0, 0x00, // 'E' (0x45)
        0xF8, 0x80, 0x80, 0xF0, 0x80, 0x80, 0xF8, 0x00, // 'F' (0x46)
        0xF8, 0x80, 0x80, 0xF0, 0x80, 0x80, 0x80, 0x00, // 'G' (0x47)
        0x70, 0x88, 0x80, 0x98, 0x88, 0x88, 0x70, 0x00, // 'H' (0x48)
        0x88, 0x88, 0x88, 0xF8, 0x88, 0x88, 0x88, 0x00, // 'I' (0x49)
        0x70, 0x20, 0x20, 0x20, 0x20, 0x20, 0x70, 0x00, // 'J' (0x4A)
        0x38, 0x10, 0x10, 0x10, 0x10, 0x90, 0x60, 0x00, // 'K' (0x4B)
        0x88, 0x90, 0xA0, 0xC0, 0xA0, 0x90, 0x88, 0x00, // 'L' (0x4C)
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xF8, 0x00, // 'M' (0x4D)
        0x88, 0xD8, 0xA8, 0xA8, 0x88, 0x88, 0x88, 0x00, // 'N' (0x4E)
        0x88, 0xC8, 0xA8, 0x98, 0x88, 0x88, 0x88, 0x00, // 'O' (0x4F)
        0x70, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70, 0x00, // 'P' (0x50)
        0xF0, 0x88, 0x88, 0xF0, 0x80, 0x80, 0x80, 0x00, // 'Q' (0x51)
        0x70, 0x88, 0x88, 0x88, 0xA8, 0x90, 0x68, 0x00, // 'R' (0x52)
        0xF0, 0x88, 0x88, 0xF0, 0xA0, 0x90, 0x88, 0x00, // 'S' (0x53)
        0x70, 0x88, 0x80, 0x70, 0x08, 0x88, 0x70, 0x00, // 'T' (0x54)
        0xF8, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00, // 'U' (0x55)
        0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70, 0x00, // 'V' (0x56)
        0x88, 0x88, 0x88, 0x88, 0x88, 0x50, 0x20, 0x00, // 'W' (0x57)
        0x88, 0x88, 0x88, 0xA8, 0xA8, 0xD8, 0x88, 0x00, // 'X' (0x58)
        0x88, 0x88, 0x50, 0x20, 0x50, 0x88, 0x88, 0x00, // 'Y' (0x59)
        0x88, 0x88, 0x50, 0x20, 0x20, 0x20, 0x20, 0x00, // 'Z' (0x5A)
        0xF8, 0x08, 0x10, 0x20, 0x40, 0x80, 0xF8, 0x00, // '[' (0x5B)
        0x70, 0x40, 0x40, 0x40, 0x40, 0x40, 0x70, 0x00, // '\' (0x5C)
        0x80, 0x80, 0x40, 0x20, 0x10, 0x08, 0x08, 0x00, // ']' (0x5D)
        0x70, 0x10, 0x10, 0x10, 0x10, 0x10, 0x70, 0x00, // '^' (0x5E)
        0x20, 0x50, 0x88, 0x00, 0x00, 0x00, 0x00, 0x00, // '_' (0x5F)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF8, 0x00, // '`' (0x60)
        0x80, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 'a' (0x61)
        0x00, 0x00, 0x70, 0x08, 0x78, 0x88, 0x78, 0x00, // 'b' (0x62)
        0x80, 0x80, 0xF0, 0x88, 0x88, 0x88, 0xF0, 0x00, // 'c' (0x63)
        0x00, 0x00, 0x70, 0x88, 0x80, 0x88, 0x70, 0x00, // 'd' (0x64)
        0x08, 0x08, 0x78, 0x88, 0x88, 0x88, 0x78, 0x00, // 'e' (0x65)
        0x00, 0x00, 0x70, 0x88, 0xF8, 0x80, 0x70, 0x00, // 'f' (0x66)
        0x10, 0x20, 0x70, 0x20, 0x20, 0x20, 0x20, 0x00, // 'g' (0x67)
        0x00, 0x00, 0x78, 0x88, 0x88, 0x78, 0x08, 0x70, // 'h' (0x68)
        0x80, 0x80, 0xF0, 0x88, 0x88, 0x88, 0x88, 0x00, // 'i' (0x69)
        0x20, 0x00, 0x60, 0x20, 0x20, 0x20, 0x70, 0x00, // 'j' (0x6A)
        0x08, 0x00, 0x18, 0x08, 0x08, 0x08, 0x88, 0x70, // 'k' (0x6B)
        0x80, 0x80, 0x88, 0x90, 0xE0, 0x90, 0x88, 0x00, // 'l' (0x6C)
        0x60, 0x20, 0x20, 0x20, 0x20, 0x20, 0x70, 0x00, // 'm' (0x6D)
        0x00, 0x00, 0xD0, 0xA8, 0xA8, 0xA8, 0x88, 0x00, // 'n' (0x6E)
        0x00, 0x00, 0xF0, 0x88, 0x88, 0x88, 0x88, 0x00, // 'o' (0x6F)
        0x00, 0x00, 0x70, 0x88, 0x88, 0x88, 0x70, 0x00, // 'p' (0x70)
        0x00, 0x00, 0xF0, 0x88, 0x88, 0xF0, 0x80, 0x80, // 'q' (0x71)
        0x00, 0x00, 0x78, 0x88, 0x88, 0x78, 0x08, 0x08, // 'r' (0x72)
        0x00, 0x00, 0xB0, 0xC8, 0x80, 0x80, 0x80, 0x00, // 's' (0x73)
        0x00, 0x00, 0x70, 0x80, 0x70, 0x08, 0x70, 0x00, // 't' (0x74)
        0x20, 0x20, 0xF8, 0x20, 0x20, 0x28, 0x10, 0x00, // 'u' (0x75)
        0x00, 0x00, 0x88, 0x88, 0x88, 0x88, 0x78, 0x00, // 'v' (0x76)
        0x00, 0x00, 0x88, 0x88, 0x88, 0x50, 0x20, 0x00, // 'w' (0x77)
        0x00, 0x00, 0x88, 0x88, 0xA8, 0xA8, 0x50, 0x00, // 'x' (0x78)
        0x00, 0x00, 0x88, 0x50, 0x20, 0x50, 0x88, 0x00, // 'y' (0x79)
        0x00, 0x00, 0x88, 0x88, 0x78, 0x08, 0x70, 0x00, // 'z' (0x7A)
        0x00, 0x00, 0xF8, 0x10, 0x20, 0x40, 0xF8, 0x00, // '{' (0x7B)
        0x10, 0x20, 0x20, 0x40, 0x20, 0x20, 0x10, 0x00, // '|' (0x7C)
        0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00, // '}' (0x7D)
        0x40, 0x20, 0x20, 0x10, 0x20, 0x20, 0x40, 0x00, // '~' (0x7E)
        0x00, 0x00, 0x00, 0x68, 0x90, 0x00, 0x00, 0x00,
    ];

    /// Look up the 8-row bitmap for an ASCII printable character.
    #[must_use]
    pub fn glyph_rows(ch: char) -> Option<&'static [u8; 8]> {
        let idx = ch as usize;
        if idx < 0x20 || idx > 0x7E {
            return None;
        }
        let offset = (idx - 0x20) * 8;
        // SAFETY: GLYPHS has exactly 95*8 = 760 bytes, and offset is 0..744.
        Some(unsafe { &*(GLYPHS.as_ptr().add(offset) as *const [u8; 8]) })
    }

    /// Rasterize a character into a `RasterizedGlyph` scaled to `font_size`.
    /// Returns `None` for non-ASCII characters.
    #[must_use]
    pub fn rasterize(ch: char, font_size: f32, color: (u8, u8, u8)) -> Option<RasterizedGlyph> {
        let rows = glyph_rows(ch)?;
        let scale = font_size / 8.0;
        let w = (5.0 * scale).round() as u32;
        let h = (8.0 * scale).round() as u32;
        let mut pixels = vec![0u8; (w * h) as usize * 4];

        for (row_idx, &row_byte) in rows.iter().enumerate() {
            let y_base = (row_idx as f32 * scale).round() as u32;
            for col in 0u32..5u32 {
                let x_base = (col as f32 * scale).round() as u32;
                if row_byte & (0x80 >> col) != 0 {
                    let span = (scale).round() as u32;
                    for dy in 0..span {
                        for dx in 0..span {
                            let px = x_base + dx;
                            let py = y_base + dy;
                            if px < w && py < h {
                                let off = ((py * w + px) * 4) as usize;
                                pixels[off] = color.0;
                                pixels[off + 1] = color.1;
                                pixels[off + 2] = color.2;
                                pixels[off + 3] = 255;
                            }
                        }
                    }
                }
            }
        }

        let advance = w as f32;
        Some(RasterizedGlyph {
            width: w,
            height: h,
            bearing_x: 0,
            bearing_y: 0,
            advance,
            pixels,
        })
    }
}

// ---------------------------------------------------------------------------
// PngRenderer (software feature only)
// ---------------------------------------------------------------------------

#[cfg(feature = "software")]
mod png_renderer {
    use super::{FontCache, RasterizedGlyph, RenderConfig, bitmap_font};

    /// Renders text tokens into a pixel buffer using the built-in bitmap font.
    pub struct PngRenderer<'a> {
        config: &'a RenderConfig,
        cache: &'a FontCache,
    }

    impl<'a> PngRenderer<'a> {
        pub fn new(config: &'a RenderConfig, cache: &'a FontCache) -> Self {
            Self { config, cache }
        }

        /// Render a list of (text, font_family) pairs into an RGBA pixel buffer.
        pub fn render(&self, lines: &[Vec<(&str, &str)>]) -> (u32, u32, Vec<u8>) {
            let line_h = self.config.pixel_line_height();
            let total_lines = lines.len().max(1);
            let height = (self.config.padding_y * 2.0 + total_lines as f32 * line_h) as u32;
            let max_width: usize = lines
                .iter()
                .map(|line| {
                    line.iter()
                        .map(|(text, family)| {
                            let m = self.cache.get_or_default(family);
                            m.text_width_px(text, self.config.font_size) as usize
                        })
                        .sum::<usize>()
                })
                .max()
                .unwrap_or(0);
            let width = (self.config.padding_x * 2.0 + max_width as f32) as u32;
            let mut pixels = vec![0u8; width as usize * height as usize * 4];

            let bg = parse_hex(&self.config.background_color);
            let fg = parse_hex(&self.config.foreground_color);

            for row in 0..height as usize {
                for col in 0..width as usize {
                    let off = (row * width as usize + col) * 4;
                    pixels[off] = bg.0;
                    pixels[off + 1] = bg.1;
                    pixels[off + 2] = bg.2;
                    pixels[off + 3] = 255;
                }
            }

            for (line_idx, line) in lines.iter().enumerate() {
                let mut x = self.config.padding_x;
                let y = self.config.padding_y + line_idx as f32 * line_h + self.config.font_size; // baseline at bottom of em-box

                for (text, _family) in line {
                    for ch in text.chars() {
                        if let Some(glyph) = bitmap_font::rasterize(ch, self.config.font_size, fg) {
                            let gx = x + glyph.bearing_x as f32;
                            let gy = y - glyph.height as f32;
                            blit_glyph(
                                &mut pixels,
                                width as usize,
                                height as usize,
                                &glyph,
                                gx,
                                gy,
                            );
                            x += glyph.advance;
                        } else {
                            x += self.config.font_size * 0.6;
                        }
                    }
                }
            }

            (width, height, pixels)
        }
    }

    fn blit_glyph(
        buf: &mut [u8],
        buf_w: usize,
        buf_h: usize,
        glyph: &RasterizedGlyph,
        gx: f32,
        gy: f32,
    ) {
        let ox = gx.round().max(0) as usize;
        let oy = gy.round().max(0) as usize;
        for row in 0..glyph.height as usize {
            let py = oy + row;
            if py >= buf_h {
                break;
            }
            for col in 0..glyph.width as usize {
                let px = ox + col;
                if px >= buf_w {
                    break;
                }
                let src_off = (row * glyph.width as usize + col) * 4;
                if glyph.pixels[src_off + 3] > 0 {
                    let dst_off = (py * buf_w + px) * 4;
                    buf[dst_off..dst_off + 4].copy_from_slice(&glyph.pixels[src_off..src_off + 4]);
                }
            }
        }
    }

    fn parse_hex(hex: &str) -> (u8, u8, u8) {
        let val = u32::from_str_radix(hex, 16).unwrap_or(0xFFFFFF);
        ((val >> 16) as u8, (val >> 8) as u8, val as u8)
    }
}

// ---------------------------------------------------------------------------
// ScopeAnnotator
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// RenderEngine
// ---------------------------------------------------------------------------

/// The main rendering engine.
///
/// Coordinates font cache, baseline alignment, and output generation.
/// Produces SVG output (always available) or PNG output (with `software` feature).
pub struct RenderEngine {
    cache: FontCache,
    config: RenderConfig,
}

impl RenderEngine {
    /// Create a new rendering engine with the given configuration.
    pub fn new(config: RenderConfig) -> Result<Self, RenderError> {
        config.validate()?;
        info!(
            width = config.width,
            height = config.height,
            font_size = config.font_size,
            "initializing render engine"
        );
        Ok(Self {
            cache: FontCache::new(),
            config,
        })
    }

    /// Create a rendering engine without validation (for testing).
    #[must_use]
    pub fn new_unchecked(config: RenderConfig) -> Self {
        Self {
            cache: FontCache::new(),
            config,
        }
    }

    /// Register a font family with fallback metrics.
    pub fn register_font(&mut self, family: &str) {
        self.cache.register_family_fallback(family);
    }

    /// Register a font family with specific metrics.
    pub fn register_font_with_metrics(&mut self, family: &str, metrics: FontMetrics) {
        self.cache.register_family(family, metrics);
    }

    /// Get the current configuration.
    #[must_use]
    pub fn config(&self) -> &RenderConfig {
        &self.config
    }

    /// Get the font cache.
    #[must_use]
    pub fn cache(&self) -> &FontCache {
        &self.cache
    }

    /// Get a mutable reference to the font cache.
    pub fn cache_mut(&mut self) -> &mut FontCache {
        &mut self.cache
    }

    // -----------------------------------------------------------------------
    // SVG rendering
    // -----------------------------------------------------------------------

    /// Render annotated tokens to SVG output.
    ///
    /// Each token is a pair of `(TokenInfo, Option<FontSpec>)`. If the font
    /// spec is `None`, the engine's default font family is used.
    #[must_use]
    pub fn render_tokens_svg(&self, tokens: &[(TokenInfo, Option<FontSpec>)]) -> RenderOutput {
        if tokens.is_empty() {
            return RenderOutput::Svg(self::svg::empty_svg(self.config.width, self.config.height));
        }

        let lines = self::svg::group_into_lines(tokens);
        let line_h = self.config.pixel_line_height();

        // Collect active families per line for baseline computation.
        let mut line_families: Vec<Vec<String>> = Vec::with_capacity(lines.len());
        for line in &lines {
            let mut families = Vec::new();
            for (_token, maybe_spec) in line {
                let family = maybe_spec
                    .as_ref()
                    .map(|s| s.family.clone())
                    .unwrap_or_else(|| self.config.default_font_family.clone());
                if !families.contains(&family) {
                    families.push(family);
                }
            }
            line_families.push(families);
        }

        // Compute baselines per line.
        let mut line_baselines: Vec<f32> = Vec::with_capacity(lines.len());
        for families in &line_families {
            let owned_metrics: Vec<FontMetrics> = families
                .iter()
                .map(|f| self.cache.get_or_default(f))
                .collect();
            let metrics_refs: Vec<&FontMetrics> = owned_metrics.iter().collect();
            let offsets = BaselineAligner::compute_offsets(&metrics_refs, self.config.font_size);
            let baseline = offsets
                .values()
                .map(|(_, b)| *b)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(self.config.font_size * 0.8);
            line_baselines.push(baseline);
        }

        let mut svg_parts: Vec<String> = Vec::new();

        // SVG header.
        let total_height = (self.config.padding_y * 2.0 + lines.len() as f32 * line_h) as u32;
        let max_width = lines
            .iter()
            .map(|line| {
                let mut x = 0.0_f32;
                for (token, maybe_spec) in line {
                    let family = maybe_spec
                        .as_ref()
                        .map(|s| &s.family)
                        .unwrap_or(&self.config.default_font_family);
                    let m = self.cache.get_or_default(family);
                    x += m.text_width_px(&token.text, self.config.font_size);
                }
                x
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        let total_width = (self.config.padding_x * 2.0 + max_width) as u32;

        svg_parts.push(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            total_width, total_height, total_width, total_height
        ));

        // Background rect.
        let bg_color = format!("#{}", self.config.background_color);
        svg_parts.push(format!(
            r#"  <rect width="{}" height="{}" fill="{}"/>"#,
            total_width, total_height, bg_color
        ));

        // Render each token as a <text> element.
        for (line_idx, line) in lines.iter().enumerate() {
            let mut x = self.config.padding_x;
            let baseline_y =
                self.config.padding_y + line_idx as f32 * line_h + line_baselines[line_idx];

            for (token, maybe_spec) in line {
                let family = maybe_spec
                    .as_ref()
                    .map(|s| &s.family)
                    .unwrap_or(&self.config.default_font_family);
                let m = self.cache.get_or_default(family);

                let fg_color = format!("#{}", self.config.foreground_color);
                let escaped_text = xml_escape(&token.text);
                svg_parts.push(format!(
                    r#"  <text x="{:.1}" y="{:.1}" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
                    x,
                    baseline_y,
                    family,
                    self.config.font_size,
                    fg_color,
                    escaped_text
                ));

                x += m.text_width_px(&token.text, self.config.font_size);
            }
        }

        svg_parts.push("</svg>".to_string());
        RenderOutput::Svg(svg_parts.join("\n"))
    }

    /// Render tokens using a `ScopeAnnotator` for font resolution.
    #[must_use]
    pub fn render_tokens_with_annotator(
        &self,
        tokens: &[polyfont_core::TokenInfo],
        annotator: &ScopeAnnotator,
    ) -> RenderOutput {
        let annotated: Vec<(TokenInfo, Option<FontSpec>)> = tokens
            .iter()
            .map(|t| {
                let family = annotator.annotate(t);
                let spec = family.as_deref().map(FontSpec::default_font);
                (t.clone(), spec)
            })
            .collect();
        self.render_tokens_svg(&annotated)
    }

    // -----------------------------------------------------------------------
    // Legacy glyph-position rendering (kept for backward compat)
    // -----------------------------------------------------------------------

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

        let line_height = self.config.pixel_line_height();

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

            let m = self.cache.get_or_default(&family);
            let advance = m.text_width_px(&token.text, self.config.font_size);

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

    // -----------------------------------------------------------------------
    // PNG rendering (software feature only)
    // -----------------------------------------------------------------------

    /// Render tokens to PNG output using the built-in bitmap font.
    ///
    /// Requires the `software` feature.
    #[cfg(feature = "software")]
    pub fn render_tokens_png(
        &self,
        tokens: &[(TokenInfo, Option<FontSpec>)],
    ) -> Result<RenderOutput, RenderError> {
        let lines = self::svg::group_into_lines(tokens);
        let render_lines: Vec<Vec<(&str, &str)>> = lines
            .iter()
            .map(|line| {
                line.iter()
                    .map(|(token, maybe_spec)| {
                        let family = maybe_spec
                            .as_ref()
                            .map(|s| s.family.as_str())
                            .unwrap_or(&self.config.default_font_family);
                        (token.text.as_str(), family)
                    })
                    .collect()
            })
            .collect();

        let renderer = png_renderer::PngRenderer::new(&self.config, &self.cache);
        let (width, height, pixels) = renderer.render(&render_lines);
        Ok(RenderOutput::Png {
            width,
            height,
            pixels,
        })
    }
}

// ---------------------------------------------------------------------------
// SVG helpers (private)
// ---------------------------------------------------------------------------

mod svg {
    use super::{FontSpec, TokenInfo};

    /// Group tokens into lines based on their line numbers.
    pub(super) fn group_into_lines(
        tokens: &[(TokenInfo, Option<FontSpec>)],
    ) -> Vec<Vec<&(TokenInfo, Option<FontSpec>)>> {
        if tokens.is_empty() {
            return Vec::new();
        }

        let mut lines: Vec<(u32, usize, usize)> = Vec::new();
        let mut current_line = tokens[0].0.range.start.line;
        let mut start = 0;

        for (idx, (token, _)) in tokens.iter().enumerate() {
            if token.range.start.line != current_line {
                lines.push((current_line, start, idx));
                current_line = token.range.start.line;
                start = idx;
            }
        }
        lines.push((current_line, start, tokens.len()));

        lines
            .into_iter()
            .map(|(_, s, e)| tokens[s..e].iter().collect())
            .collect()
    }

    pub(super) fn empty_svg(width: u32, height: u32) -> String {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            width, height, width, height
        )
    }
}

/// Escape text for use in SVG/XML content.
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// RenderError display tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use polyfont_core::{FontRule, FontSpec, Position, Range, TokenInfo};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn make_token(text: &str, line: u32, scope: &str) -> TokenInfo {
        let col = text.len() as u32;
        TokenInfo {
            text: text.to_string(),
            range: Range {
                start: Position { line, column: 0 },
                end: Position { line, column: col },
            },
            scope: scope.to_string(),
            modifiers: vec![],
        }
    }

    #[allow(dead_code)]
    fn make_engine_with_rules(_rules: Vec<FontRule>) -> RenderEngine {
        RenderEngine::new_unchecked(RenderConfig::default())
    }

    fn default_render_engine() -> RenderEngine {
        let mut engine = RenderEngine::new_unchecked(RenderConfig::default());
        engine.register_font("Mono");
        engine.register_font("Serif");
        engine.register_font("Sans");
        engine.register_font("MonoA");
        engine.register_font("MonoB");
        engine.register_font("MonoC");
        engine.register_font("MonoD");
        engine.register_font("MonoE");
        engine.register_font("MonoF");
        engine
    }

    // -----------------------------------------------------------------------
    // RenderConfig tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_render_config_default() {
        let config = RenderConfig::default();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!((config.font_size - 14.0).abs() < f32::EPSILON);
        assert!((config.line_height - 1.4).abs() < f32::EPSILON);
        assert_eq!(config.default_font_family, "monospace");
    }

    #[test]
    fn test_render_config_builder() {
        let config = RenderConfig::builder()
            .with_font_size(16.0)
            .with_line_height(1.6)
            .with_viewport(1024, 768)
            .with_default_font("FiraCode");
        assert!((config.font_size - 16.0).abs() < f32::EPSILON);
        assert!((config.line_height - 1.6).abs() < f32::EPSILON);
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert_eq!(config.default_font_family, "FiraCode");
    }

    #[test]
    fn test_render_config_pixel_line_height() {
        let config = RenderConfig::builder()
            .with_font_size(10.0)
            .with_line_height(1.5);
        assert!((config.pixel_line_height() - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_render_config_validate_ok() {
        let config = RenderConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_render_config_validate_negative_size() {
        let config = RenderConfig::builder().with_font_size(-1.0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_render_config_validate_zero_line_height() {
        let config = RenderConfig::builder().with_line_height(0.0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_render_config_validate_zero_viewport() {
        let config = RenderConfig::builder().with_viewport(0, 600);
        assert!(config.validate().is_err());
    }

    // -----------------------------------------------------------------------
    // FontMetrics tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_font_metrics_monospace_fallback() {
        let m = FontMetrics::monospace_fallback("Test");
        assert_eq!(m.family, "Test");
        assert_eq!(m.units_per_em, 16);
        assert!((m.ascent - 12.0).abs() < f32::EPSILON);
        assert!((m.descent - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_font_metrics_ascent_px() {
        let m = FontMetrics::monospace_fallback("f");
        let px = m.ascent_px(32.0);
        assert!((px - 24.0).abs() < 0.01);
    }

    #[test]
    fn test_font_metrics_descent_px() {
        let m = FontMetrics::monospace_fallback("f");
        let px = m.descent_px(32.0);
        assert!((px - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_font_metrics_zero_units_per_em() {
        let m = FontMetrics {
            family: "x".to_string(),
            ascent: 10.0,
            descent: 2.0,
            line_gap: 1.0,
            units_per_em: 0,
        };
        assert!((m.ascent_px(14.0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_font_metrics_char_width_px() {
        let m = FontMetrics::monospace_fallback("f");
        let w = m.char_width_px(14.0);
        assert!((w - 8.4).abs() < 0.01);
    }

    #[test]
    fn test_font_metrics_text_width_px() {
        let m = FontMetrics::monospace_fallback("f");
        let w = m.text_width_px("hello", 14.0);
        assert!((w - 42.0).abs() < 0.01);
    }

    // -----------------------------------------------------------------------
    // FontCache tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_font_cache_new() {
        let cache = FontCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_font_cache_register_family() {
        let mut cache = FontCache::new();
        let m = FontMetrics::monospace_fallback("Test");
        cache.register_family("Test", m);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_font_cache_register_family_fallback() {
        let mut cache = FontCache::new();
        cache.register_family_fallback("Mono");
        assert!(cache.get("Mono").is_some());
    }

    #[test]
    fn test_font_cache_register_duplicate() {
        let mut cache = FontCache::new();
        cache.register_family_fallback("A");
        cache.register_family_fallback("A");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_font_cache_get() {
        let mut cache = FontCache::new();
        cache.register_family_fallback("Found");
        assert!(cache.get("Found").is_some());
        assert!(cache.get("Missing").is_none());
    }

    #[test]
    fn test_font_cache_get_or_default() {
        let cache = FontCache::new();
        let m = cache.get_or_default("Unknown");
        assert_eq!(m.family, "Unknown");
    }

    #[test]
    fn test_font_cache_families() {
        let mut cache = FontCache::new();
        cache.register_family_fallback("A");
        cache.register_family_fallback("B");
        let mut families = cache.families();
        families.sort();
        assert_eq!(families, vec!["A", "B"]);
    }

    #[test]
    fn test_font_cache_clear() {
        let mut cache = FontCache::new();
        cache.register_family_fallback("X");
        cache.clear();
        assert!(cache.is_empty());
    }

    // -----------------------------------------------------------------------
    // BaselineAligner tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_baseline_single_font() {
        let m = FontMetrics::monospace_fallback("Mono");
        let offsets = BaselineAligner::compute_offsets(&[&m], 14.0);
        assert_eq!(offsets.len(), 1);
        let (_, baseline) = offsets.get("Mono").copied().unwrap();
        // With units_per_em=16, ascent=12, font_size=14: ascent_px = 12*14/16 = 10.5
        assert!((baseline - 10.5).abs() < 0.01);
    }

    #[test]
    fn test_baseline_two_fonts_different_ascent() {
        let m1 = FontMetrics {
            family: "Tall".to_string(),
            ascent: 16.0,
            descent: 3.0,
            line_gap: 0.0,
            units_per_em: 20,
        };
        let m2 = FontMetrics {
            family: "Short".to_string(),
            ascent: 10.0,
            descent: 2.0,
            line_gap: 0.0,
            units_per_em: 20,
        };
        let offsets = BaselineAligner::compute_offsets(&[&m1, &m2], 14.0);
        // Tall: ascent_px = 16*14/20 = 11.2
        // Short: ascent_px = 10*14/20 = 7.0
        // max_ascent = 11.2, baseline = 11.2
        // Tall offset: 11.2 - 11.2 = 0.0
        // Short offset: 11.2 - 7.0 = 4.2
        let (tall_off, tall_base) = offsets.get("Tall").copied().unwrap();
        let (short_off, short_base) = offsets.get("Short").copied().unwrap();
        assert!((tall_off - 0.0).abs() < 0.01);
        assert!((short_off - 4.2).abs() < 0.01);
        assert!((tall_base - short_base).abs() < 0.01);
    }

    #[test]
    fn test_baseline_three_fonts() {
        let m1 = FontMetrics::monospace_fallback("A");
        let m2 = FontMetrics::monospace_fallback("B");
        let m3 = FontMetrics::monospace_fallback("C");
        let offsets = BaselineAligner::compute_offsets(&[&m1, &m2, &m3], 14.0);
        assert_eq!(offsets.len(), 3);
    }

    #[test]
    fn test_baseline_empty() {
        let offsets = BaselineAligner::compute_offsets(&[], 14.0);
        assert!(offsets.is_empty());
    }

    #[test]
    fn test_single_font_baseline() {
        let m = FontMetrics::monospace_fallback("f");
        let b = BaselineAligner::single_font_baseline(&m, 14.0);
        // ascent_px = 12*14/16 = 10.5
        assert!((b - 10.5).abs() < 0.01);
    }

    // -----------------------------------------------------------------------
    // RenderError tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_render_error_display() {
        assert_eq!(
            RenderError::NoBackend.to_string(),
            "no rendering backend available"
        );
        assert_eq!(
            RenderError::SoftwarePathNotEnabled.to_string(),
            "software path not enabled; compile with --features software"
        );
    }

    #[test]
    fn test_render_error_font_cache() {
        assert_eq!(
            RenderError::FontCache("missing font".into()).to_string(),
            "font cache error: missing font"
        );
    }

    #[test]
    fn test_render_error_metrics() {
        let err = RenderError::Metrics {
            family: "Test".into(),
            reason: "bad table".into(),
        };
        assert_eq!(
            err.to_string(),
            "metrics extraction failed for Test: bad table"
        );
    }

    // -----------------------------------------------------------------------
    // RenderEngine creation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_render_engine_new() {
        let engine = RenderEngine::new(RenderConfig::default());
        assert!(engine.is_ok());
    }

    #[test]
    fn test_render_engine_new_unchecked() {
        let engine = RenderEngine::new_unchecked(RenderConfig::default());
        assert_eq!(engine.config().font_size, 14.0);
    }

    #[test]
    fn test_render_engine_register_font() {
        let mut engine = RenderEngine::new_unchecked(RenderConfig::default());
        engine.register_font("JetBrains Mono");
        assert!(engine.cache().get("JetBrains Mono").is_some());
    }

    #[test]
    fn test_render_engine_cache_mut() {
        let mut engine = RenderEngine::new_unchecked(RenderConfig::default());
        engine.cache_mut().register_family_fallback("Custom");
        assert!(engine.cache().get("Custom").is_some());
    }

    // -----------------------------------------------------------------------
    // SVG rendering tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_render_svg_empty() {
        let engine = RenderEngine::new_unchecked(RenderConfig::default());
        let output = engine.render_tokens_svg(&[]);
        match output {
            RenderOutput::Svg(svg) => {
                assert!(svg.contains("<svg"));
                assert!(svg.contains("xmlns"));
            }
            _ => panic!("expected SVG output"),
        }
    }

    #[test]
    fn test_render_svg_single_token() {
        let engine = default_render_engine();
        let token = make_token("fn", 0, "keyword");
        let output = engine.render_tokens_svg(&[(token, Some(FontSpec::default_font("Mono")))]);
        match output {
            RenderOutput::Svg(svg) => {
                assert!(svg.contains("font-family=\"Mono\""));
                assert!(svg.contains(">fn<"));
            }
            _ => panic!("expected SVG"),
        }
    }

    #[test]
    fn test_render_svg_default_font() {
        let engine = RenderEngine::new_unchecked(RenderConfig::default());
        let token = make_token("hello", 0, "text");
        let output = engine.render_tokens_svg(&[(token, None)]);
        match output {
            RenderOutput::Svg(svg) => {
                assert!(svg.contains("font-family=\"monospace\""));
            }
            _ => panic!("expected SVG"),
        }
    }

    #[test]
    fn test_render_svg_multi_font() {
        let engine = default_render_engine();
        let tokens = vec![
            (
                make_token("fn", 0, "keyword"),
                Some(FontSpec::default_font("Mono")),
            ),
            (
                make_token("main", 0, "function"),
                Some(FontSpec::default_font("Serif")),
            ),
            (
                make_token("()", 0, "punctuation"),
                Some(FontSpec::default_font("Sans")),
            ),
        ];
        let output = engine.render_tokens_svg(&tokens);
        match output {
            RenderOutput::Svg(svg) => {
                assert!(svg.contains("font-family=\"Mono\""));
                assert!(svg.contains("font-family=\"Serif\""));
                assert!(svg.contains("font-family=\"Sans\""));
            }
            _ => panic!("expected SVG"),
        }
    }

    #[test]
    fn test_render_svg_multiline() {
        let engine = default_render_engine();
        let tokens = vec![
            (
                make_token("fn", 0, "kw"),
                Some(FontSpec::default_font("Mono")),
            ),
            (
                make_token("let", 1, "kw"),
                Some(FontSpec::default_font("Serif")),
            ),
            (
                make_token("x", 1, "var"),
                Some(FontSpec::default_font("Mono")),
            ),
            (
                make_token("}", 2, "punct"),
                Some(FontSpec::default_font("Mono")),
            ),
        ];
        let output = engine.render_tokens_svg(&tokens);
        match output {
            RenderOutput::Svg(svg) => {
                let count = svg.matches("<text ").count();
                assert_eq!(count, 4);
            }
            _ => panic!("expected SVG"),
        }
    }

    #[test]
    fn test_render_svg_multiline_y_increasing() {
        let engine = default_render_engine();
        let tokens = vec![
            (
                make_token("a", 0, "x"),
                Some(FontSpec::default_font("Mono")),
            ),
            (
                make_token("b", 1, "x"),
                Some(FontSpec::default_font("Mono")),
            ),
        ];
        let output = engine.render_tokens_svg(&tokens);
        let RenderOutput::Svg(svg) = output else {
            panic!("expected SVG")
        };
        let text_elems: Vec<&str> = svg.lines().filter(|l| l.contains("<text ")).collect();
        assert_eq!(text_elems.len(), 2);
        let y0: f32 = text_elems[0]
            .split("y=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let y1: f32 = text_elems[1]
            .split("y=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!(y1 > y0);
    }

    #[test]
    fn test_render_svg_x_positions_advance() {
        let engine = default_render_engine();
        let tokens = vec![
            (
                make_token("a", 0, "x"),
                Some(FontSpec::default_font("Mono")),
            ),
            (
                make_token("b", 0, "x"),
                Some(FontSpec::default_font("Mono")),
            ),
        ];
        let output = engine.render_tokens_svg(&tokens);
        let RenderOutput::Svg(svg) = output else {
            panic!("expected SVG")
        };
        let text_elems: Vec<&str> = svg.lines().filter(|l| l.contains("<text ")).collect();
        assert_eq!(text_elems.len(), 2);
        let x0: f32 = text_elems[0]
            .split("x=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let x1: f32 = text_elems[1]
            .split("x=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!(x1 > x0);
    }

    #[test]
    fn test_render_svg_unicode() {
        let engine = default_render_engine();
        let token = TokenInfo {
            text: "你好".to_string(),
            range: Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 2 },
            },
            scope: "text".to_string(),
            modifiers: vec![],
        };
        let output = engine.render_tokens_svg(&[(token, Some(FontSpec::default_font("Mono")))]);
        match output {
            RenderOutput::Svg(svg) => {
                assert!(svg.contains("你好"));
            }
            _ => panic!("expected SVG"),
        }
    }

    #[test]
    fn test_render_svg_xml_escape() {
        let engine = default_render_engine();
        let token = make_token("<div>", 0, "tag");
        let output = engine.render_tokens_svg(&[(token, Some(FontSpec::default_font("Mono")))]);
        match output {
            RenderOutput::Svg(svg) => {
                assert!(svg.contains("&lt;div&gt;"));
                assert!(!svg.contains("<div>"));
            }
            _ => panic!("expected SVG"),
        }
    }

    #[test]
    fn test_render_svg_contains_background() {
        let engine = RenderEngine::new_unchecked(RenderConfig::default());
        let token = make_token("x", 0, "t");
        let output = engine.render_tokens_svg(&[(token, None)]);
        let RenderOutput::Svg(svg) = output else {
            panic!("expected SVG")
        };
        assert!(svg.contains("<rect"));
        assert!(svg.contains(&format!("fill=\"#{}\"", engine.config().background_color)));
    }

    #[test]
    fn test_render_svg_font_size() {
        let engine = RenderEngine::new_unchecked(RenderConfig::builder().with_font_size(18.0));
        let token = make_token("a", 0, "t");
        let output = engine.render_tokens_svg(&[(token, None)]);
        let RenderOutput::Svg(svg) = output else {
            panic!("expected SVG")
        };
        assert!(svg.contains("font-size=\"18"));
    }

    #[test]
    fn test_render_svg_eight_font_families() {
        let engine = default_render_engine();
        let tokens: Vec<(TokenInfo, Option<FontSpec>)> = (0..10)
            .map(|i| {
                let families = [
                    "Mono", "Serif", "Sans", "MonoA", "MonoB", "MonoC", "MonoD", "MonoE", "MonoF",
                    "Mono",
                ];
                (
                    make_token(&format!("t{}", i), 0, &format!("s{}", i)),
                    Some(FontSpec::default_font(families[i])),
                )
            })
            .collect();
        let output = engine.render_tokens_svg(&tokens);
        let RenderOutput::Svg(svg) = output else {
            panic!("expected SVG")
        };
        assert!(svg.contains("font-family=\"MonoA\""));
        assert!(svg.contains("font-family=\"MonoB\""));
        assert!(svg.contains("font-family=\"MonoC\""));
        assert!(svg.contains("font-family=\"MonoD\""));
        assert!(svg.contains("font-family=\"MonoE\""));
        assert!(svg.contains("font-family=\"MonoF\""));
    }

    // -----------------------------------------------------------------------
    // Annotator + render pipeline tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_render_with_annotator() {
        let scope_engine = polyfont_core::ScopeMatchEngine::from_rules(vec![
            FontRule {
                scope: "keyword".to_string(),
                font: FontSpec::default_font("Mono"),
            },
            FontRule {
                scope: "string".to_string(),
                font: FontSpec::default_font("Serif"),
            },
        ]);
        let annotator = ScopeAnnotator::new(scope_engine);
        let engine = default_render_engine();
        let output = engine.render_tokens_with_annotator(
            &[
                make_token("fn", 0, "keyword"),
                make_token("\"hi\"", 0, "string"),
            ],
            &annotator,
        );
        let RenderOutput::Svg(svg) = output else {
            panic!("expected SVG")
        };
        assert!(svg.contains("font-family=\"Mono\""));
        assert!(svg.contains("font-family=\"Serif\""));
    }

    // -----------------------------------------------------------------------
    // Legacy render_tokens tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_render_engine_tokens() {
        let scope_engine = polyfont_core::ScopeMatchEngine::from_rules(vec![FontRule {
            scope: "*".to_string(),
            font: FontSpec::default_font("Mono"),
        }]);
        let annotator = ScopeAnnotator::new(scope_engine);
        let mut engine = RenderEngine::new_unchecked(RenderConfig::default());
        engine.cache_mut().register_family_fallback("Mono");

        let tokens = vec![
            make_token("fn", 0, "keyword"),
            make_token("main", 0, "entity.name.function"),
        ];

        let lines = engine.render_tokens(&tokens, &annotator).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].glyphs.len(), 2);
    }

    #[test]
    fn test_render_engine_multiline_tokens() {
        let scope_engine = polyfont_core::ScopeMatchEngine::from_rules(vec![FontRule {
            scope: "*".to_string(),
            font: FontSpec::default_font("Mono"),
        }]);
        let annotator = ScopeAnnotator::new(scope_engine);
        let mut engine = RenderEngine::new_unchecked(RenderConfig::default());
        engine.cache_mut().register_family_fallback("Mono");

        let tokens = vec![make_token("fn", 0, "kw"), make_token("let", 1, "kw")];

        let lines = engine.render_tokens(&tokens, &annotator).unwrap();
        assert_eq!(lines.len(), 2);
    }

    // -----------------------------------------------------------------------
    // xml_escape tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_xml_escape_basic() {
        assert_eq!(xml_escape("hello"), "hello");
    }

    #[test]
    fn test_xml_escape_ampersand() {
        assert_eq!(xml_escape("a&b"), "a&amp;b");
    }

    #[test]
    fn test_xml_escape_lt_gt() {
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_xml_escape_quotes() {
        assert_eq!(xml_escape("a\"b'c"), "a&quot;b&apos;c");
    }

    // -----------------------------------------------------------------------
    // RenderOutput tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_render_output_svg_clone() {
        let output = RenderOutput::Svg("<svg/>".to_string());
        let cloned = output.clone();
        match cloned {
            RenderOutput::Svg(s) => assert_eq!(s, "<svg/>"),
            _ => panic!("expected SVG"),
        }
    }

    #[test]
    fn test_render_output_png_clone() {
        let output = RenderOutput::Png {
            width: 10,
            height: 10,
            pixels: vec![0u8; 400],
        };
        let cloned = output.clone();
        match cloned {
            RenderOutput::Png { width, height, .. } => {
                assert_eq!(width, 10);
                assert_eq!(height, 10);
            }
            _ => panic!("expected Png"),
        }
    }

    // -----------------------------------------------------------------------
    // document rendering test (full pipeline)
    // -----------------------------------------------------------------------

    #[test]
    fn test_render_document_full_pipeline() {
        let scope_engine = polyfont_core::ScopeMatchEngine::from_rules(vec![
            FontRule {
                scope: "keyword".to_string(),
                font: FontSpec::default_font("Mono"),
            },
            FontRule {
                scope: "entity.name.function".to_string(),
                font: FontSpec::default_font("Serif"),
            },
            FontRule {
                scope: "*".to_string(),
                font: FontSpec::default_font("Sans"),
            },
        ]);
        let annotator = ScopeAnnotator::new(scope_engine);
        let mut engine = RenderEngine::new_unchecked(RenderConfig::default());
        engine.register_font("Mono");
        engine.register_font("Serif");
        engine.register_font("Sans");

        let tokens = vec![
            make_token("fn", 0, "keyword"),
            make_token("main", 0, "entity.name.function"),
            make_token("() {", 0, "punctuation"),
            make_token("  let", 1, "keyword"),
            make_token("x", 1, "variable"),
            make_token("= 1;", 1, "punctuation"),
            make_token("}", 2, "punctuation"),
        ];

        let output = engine.render_tokens_with_annotator(&tokens, &annotator);
        let RenderOutput::Svg(svg) = output else {
            panic!("expected SVG")
        };
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("font-family=\"Mono\""));
        assert!(svg.contains("font-family=\"Serif\""));
        assert!(svg.contains("font-family=\"Sans\""));
    }

    // -----------------------------------------------------------------------
    // Software feature tests
    // -----------------------------------------------------------------------

    #[test]
    #[cfg(feature = "software")]
    fn test_bitmap_font_glyph_rows() {
        let rows = bitmap_font::glyph_rows('A');
        assert!(rows.is_some());
        let r = rows.unwrap();
        assert_eq!(r.len(), 8);
    }

    #[test]
    #[cfg(feature = "software")]
    fn test_bitmap_font_non_ascii_returns_none() {
        assert!(bitmap_font::glyph_rows('\u{00E9}').is_none());
    }

    #[test]
    #[cfg(feature = "software")]
    fn test_bitmap_font_rasterize() {
        let glyph = bitmap_font::rasterize('A', 8.0, (255, 255, 255)).unwrap();
        assert_eq!(glyph.width, 5);
        assert_eq!(glyph.height, 8);
        assert!(!glyph.pixels.is_empty());
    }

    #[test]
    #[cfg(feature = "software")]
    fn test_png_render_empty() {
        let engine = RenderEngine::new_unchecked(RenderConfig::default());
        let result = engine.render_tokens_png(&[]);
        assert!(result.is_ok());
        match result.unwrap() {
            RenderOutput::Png {
                width,
                height,
                pixels,
            } => {
                assert_eq!(width, 20); // padding_x * 2
                assert!(height > 0);
                assert!(!pixels.is_empty());
            }
            _ => panic!("expected PNG"),
        }
    }

    #[test]
    #[cfg(feature = "software")]
    fn test_png_render_single_line() {
        let mut engine = RenderEngine::new_unchecked(RenderConfig::default());
        engine.register_font("Mono");
        let token = make_token("AB", 0, "text");
        let result = engine.render_tokens_png(&[(token, Some(FontSpec::default_font("Mono")))]);
        assert!(result.is_ok());
    }
}
