use std::collections::HashMap;

use polyfont_config::{FontConfig, PolyfontConfig, RuleConfig};
use polyfont_core::{FontStyle, FontWeight};
use serde::{Deserialize, Serialize};

use crate::ThemeError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMapping {
    pub scope_fonts: HashMap<String, String>,
    pub default_family: Option<String>,
}

impl FontMapping {
    #[must_use]
    pub fn default_mapping() -> Self {
        let mut scope_fonts = HashMap::new();
        scope_fonts.insert("keyword".to_string(), "Maple Mono".to_string());
        scope_fonts.insert("comment".to_string(), "IBM Plex Mono".to_string());
        scope_fonts.insert("string".to_string(), "Source Code Pro".to_string());
        scope_fonts.insert(
            "entity.name.function".to_string(),
            "Monaspace Argon".to_string(),
        );
        scope_fonts.insert("entity.name.type".to_string(), "Monaspace Neon".to_string());
        scope_fonts.insert("variable".to_string(), "JetBrains Mono".to_string());
        scope_fonts.insert("constant".to_string(), "Monaspace Radon".to_string());
        scope_fonts.insert(
            "support.function".to_string(),
            "Monaspace Krypton".to_string(),
        );
        scope_fonts.insert("storage.type".to_string(), "Maple Mono".to_string());

        Self {
            scope_fonts,
            default_family: Some("JetBrains Mono".to_string()),
        }
    }

    pub fn from_toml(toml: &str) -> Result<Self, ThemeError> {
        let mapping: Self = toml::from_str(toml)?;
        Ok(mapping)
    }

    fn resolve_family(&self, scope: &str) -> String {
        self.scope_fonts
            .get(scope)
            .cloned()
            .or_else(|| self.find_prefix_match(scope))
            .or_else(|| self.default_family.clone())
            .unwrap_or_else(|| "monospace".to_string())
    }

    fn find_prefix_match(&self, scope: &str) -> Option<String> {
        let mut parts: Vec<&str> = scope.split('.').collect();
        while parts.len() > 1 {
            parts.pop();
            let prefix = parts.join(".");
            if let Some(family) = self.scope_fonts.get(&prefix) {
                return Some(family.clone());
            }
        }
        None
    }
}

fn parse_font_style(style_str: &str) -> (FontWeight, FontStyle) {
    let mut weight = FontWeight::default();
    let mut style = FontStyle::default();

    for token in style_str.split_whitespace() {
        match token {
            "bold" => weight = FontWeight::Bold,
            "italic" => style = FontStyle::Italic,
            "oblique" => style = FontStyle::Oblique,
            _ => {}
        }
    }

    (weight, style)
}

pub fn build_config_from_entries(
    entries: Vec<(String, String)>,
    mapping: &FontMapping,
) -> PolyfontConfig {
    let mut rules = Vec::new();

    for (scope, font_style) in &entries {
        let (weight, style) = parse_font_style(font_style);
        let family = mapping.resolve_family(scope);

        rules.push(RuleConfig {
            scope: scope.clone(),
            font: FontConfig {
                family,
                fallbacks: vec![],
                weight,
                style,
                size: None,
            },
        });
    }

    let default_family = mapping
        .default_family
        .clone()
        .unwrap_or_else(|| "monospace".to_string());

    PolyfontConfig {
        version: 1,
        default: Some(polyfont_config::DefaultFontConfig {
            family: default_family,
            fallbacks: vec![],
            weight: FontWeight::default(),
            style: FontStyle::default(),
            size: None,
        }),
        rules,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mapping_has_all_required_entries() {
        let m = FontMapping::default_mapping();
        let required = [
            "keyword",
            "comment",
            "string",
            "entity.name.function",
            "entity.name.type",
            "variable",
            "constant",
            "support.function",
            "storage.type",
        ];
        for scope in &required {
            assert!(m.scope_fonts.contains_key(*scope), "missing scope: {scope}");
        }
        assert!(m.default_family.is_some());
    }

    #[test]
    fn from_toml_roundtrip() {
        let toml = r#"
default_family = "Fira Code"

[scope_fonts]
keyword = "Maple Mono"
comment = "IBM Plex Mono"
"#;
        let m = FontMapping::from_toml(toml).unwrap();
        assert_eq!(m.scope_fonts.get("keyword").unwrap(), "Maple Mono");
        assert_eq!(m.default_family.as_deref(), Some("Fira Code"));
    }

    #[test]
    fn prefix_match_falls_back() {
        let m = FontMapping::default_mapping();
        assert_eq!(
            m.resolve_family("entity.name.function.decl"),
            "Monaspace Argon"
        );
    }

    #[test]
    fn default_used_for_unknown_scope() {
        let m = FontMapping::default_mapping();
        assert_eq!(m.resolve_family("unknown.scope"), "JetBrains Mono");
    }

    #[test]
    fn parse_font_style_bold_italic() {
        let (w, s) = parse_font_style("bold italic");
        assert_eq!(w, FontWeight::Bold);
        assert_eq!(s, FontStyle::Italic);
    }

    #[test]
    fn parse_font_style_empty() {
        let (w, s) = parse_font_style("");
        assert_eq!(w, FontWeight::Regular);
        assert_eq!(s, FontStyle::Normal);
    }
}
