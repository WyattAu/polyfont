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
        }
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
