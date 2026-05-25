use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    #[default]
    Regular = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

impl std::fmt::Display for FontWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thin => write!(f, "thin"),
            Self::ExtraLight => write!(f, "extra-light"),
            Self::Light => write!(f, "light"),
            Self::Regular => write!(f, "regular"),
            Self::Medium => write!(f, "medium"),
            Self::SemiBold => write!(f, "semi-bold"),
            Self::Bold => write!(f, "bold"),
            Self::ExtraBold => write!(f, "extra-bold"),
            Self::Black => write!(f, "black"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

impl std::fmt::Display for FontStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Italic => write!(f, "italic"),
            Self::Oblique => write!(f, "oblique"),
        }
    }
}

/// Named variable font axes for common controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NamedAxis {
    Weight,
    Width,
    Slant,
    OpticalSize,
    Italic,
}

impl NamedAxis {
    /// Returns the 4-character OpenType axis tag.
    #[must_use]
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Weight => "wght",
            Self::Width => "wdth",
            Self::Slant => "slnt",
            Self::OpticalSize => "opsz",
            Self::Italic => "ital",
        }
    }
}

impl std::fmt::Display for NamedAxis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Weight => write!(f, "weight"),
            Self::Width => write!(f, "width"),
            Self::Slant => write!(f, "slant"),
            Self::OpticalSize => write!(f, "optical-size"),
            Self::Italic => write!(f, "italic"),
        }
    }
}

/// A variable font axis value, either named or custom.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AxisValue {
    Named { axis: NamedAxis, value: f32 },
    Custom { tag: String, value: f32 },
}

impl AxisValue {
    /// Returns the axis tag (4 chars for OpenType, or custom string).
    #[must_use]
    pub fn tag(&self) -> &str {
        match self {
            Self::Named { axis, .. } => axis.tag(),
            Self::Custom { tag, .. } => tag,
        }
    }

    /// Returns the axis value.
    #[must_use]
    pub fn value(&self) -> f32 {
        match self {
            Self::Named { value, .. } | Self::Custom { value, .. } => *value,
        }
    }

    /// Formats as CSS `font-variation-settings` value (e.g., `"wght" 650`).
    #[must_use]
    pub fn to_css(&self) -> String {
        format!("\"{}\" {}", self.tag(), self.value())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSpec {
    pub family: String,
    #[serde(default)]
    pub fallbacks: Vec<String>,
    #[serde(default)]
    pub weight: FontWeight,
    #[serde(default)]
    pub style: FontStyle,
    #[serde(default)]
    pub size: Option<f32>,
    /// Variable font axis overrides. Keys are axis tags or named axes.
    #[serde(default)]
    pub axes: Vec<AxisValue>,
}

impl FontSpec {
    #[must_use]
    pub fn default_font(family: &str) -> Self {
        Self {
            family: family.to_string(),
            fallbacks: vec![],
            weight: FontWeight::default(),
            style: FontStyle::default(),
            size: None,
            axes: vec![],
        }
    }

    /// Returns CSS `font-variation-settings` string for all axes, or empty if none.
    #[must_use]
    pub fn css_variation_settings(&self) -> String {
        if self.axes.is_empty() {
            return String::new();
        }
        let parts: Vec<String> = self.axes.iter().map(AxisValue::to_css).collect();
        parts.join(", ")
    }

    /// Returns a map from axis tag to value for programmatic access.
    #[must_use]
    pub fn axes_map(&self) -> BTreeMap<String, f32> {
        self.axes
            .iter()
            .map(|a| (a.tag().to_string(), a.value()))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontRule {
    pub scope: String,
    pub font: FontSpec,
}

impl FontRule {
    #[must_use]
    pub fn specificity(&self) -> usize {
        self.scope.split('.').count()
    }
}

#[derive(Debug, Clone)]
pub struct FontAssignment {
    pub scope: String,
    pub font: FontSpec,
    pub specificity: usize,
    pub is_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specificity_single() {
        let rule = FontRule {
            scope: "keyword".to_string(),
            font: FontSpec::default_font("F"),
        };
        assert_eq!(rule.specificity(), 1);
    }

    #[test]
    fn test_specificity_dotted() {
        let rule = FontRule {
            scope: "entity.name.function".to_string(),
            font: FontSpec::default_font("F"),
        };
        assert_eq!(rule.specificity(), 3);
    }

    #[test]
    fn test_specificity_wildcard() {
        let rule = FontRule {
            scope: "*".to_string(),
            font: FontSpec::default_font("F"),
        };
        assert_eq!(rule.specificity(), 1);
    }

    #[test]
    fn test_font_spec_default() {
        let spec = FontSpec::default_font("Test");
        assert_eq!(spec.family, "Test");
        assert!(spec.fallbacks.is_empty());
        assert_eq!(spec.weight, FontWeight::Regular);
        assert_eq!(spec.style, FontStyle::Normal);
        assert!(spec.size.is_none());
        assert!(spec.axes.is_empty());
    }

    #[test]
    fn test_named_axis_tags() {
        assert_eq!(NamedAxis::Weight.tag(), "wght");
        assert_eq!(NamedAxis::Width.tag(), "wdth");
        assert_eq!(NamedAxis::Slant.tag(), "slnt");
        assert_eq!(NamedAxis::OpticalSize.tag(), "opsz");
        assert_eq!(NamedAxis::Italic.tag(), "ital");
    }

    #[test]
    fn test_axis_value_named() {
        let av = AxisValue::Named {
            axis: NamedAxis::Weight,
            value: 650.0,
        };
        assert_eq!(av.tag(), "wght");
        assert!((av.value() - 650.0).abs() < f32::EPSILON);
        assert_eq!(av.to_css(), "\"wght\" 650");
    }

    #[test]
    fn test_axis_value_custom() {
        let av = AxisValue::Custom {
            tag: "CASL".to_string(),
            value: 0.5,
        };
        assert_eq!(av.tag(), "CASL");
        assert!((av.value() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_css_variation_settings() {
        let spec = FontSpec {
            family: "Test".to_string(),
            fallbacks: vec![],
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            size: None,
            axes: vec![
                AxisValue::Named {
                    axis: NamedAxis::Weight,
                    value: 450.0,
                },
                AxisValue::Custom {
                    tag: "CASL".to_string(),
                    value: 1.0,
                },
            ],
        };
        let css = spec.css_variation_settings();
        assert_eq!(css, "\"wght\" 450, \"CASL\" 1");
    }

    #[test]
    fn test_axes_map() {
        let spec = FontSpec {
            family: "Test".to_string(),
            fallbacks: vec![],
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            size: None,
            axes: vec![AxisValue::Named {
                axis: NamedAxis::Width,
                value: 75.0,
            }],
        };
        let map = spec.axes_map();
        assert_eq!(map.len(), 1);
        assert!((map.get("wdth").copied().unwrap() - 75.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_font_weight_display() {
        assert_eq!(FontWeight::Thin.to_string(), "thin");
        assert_eq!(FontWeight::ExtraLight.to_string(), "extra-light");
        assert_eq!(FontWeight::Light.to_string(), "light");
        assert_eq!(FontWeight::Regular.to_string(), "regular");
        assert_eq!(FontWeight::Medium.to_string(), "medium");
        assert_eq!(FontWeight::SemiBold.to_string(), "semi-bold");
        assert_eq!(FontWeight::Bold.to_string(), "bold");
        assert_eq!(FontWeight::ExtraBold.to_string(), "extra-bold");
        assert_eq!(FontWeight::Black.to_string(), "black");
    }

    #[test]
    fn test_font_style_display() {
        assert_eq!(FontStyle::Normal.to_string(), "normal");
        assert_eq!(FontStyle::Italic.to_string(), "italic");
        assert_eq!(FontStyle::Oblique.to_string(), "oblique");
    }
}
