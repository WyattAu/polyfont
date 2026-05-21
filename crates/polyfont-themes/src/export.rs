use polyfont_config::PolyfontConfig;

use crate::ThemeError;

pub struct ThemeExporter;

impl ThemeExporter {
    pub fn export_config(config: &PolyfontConfig) -> Result<String, ThemeError> {
        let toml_str = toml::to_string_pretty(config)?;
        Ok(format!("# polyfont config\n{toml_str}"))
    }

    pub fn export_vscode(config: &PolyfontConfig) -> Result<String, ThemeError> {
        let mut entries = Vec::new();

        for rule in &config.rules {
            let mut font_style_parts = Vec::new();
            if rule.font.weight != polyfont_core::FontWeight::default() {
                font_style_parts.push(format!("\"{}\"", rule.font.weight));
            }
            if rule.font.style != polyfont_core::FontStyle::default() {
                font_style_parts.push(format!("\"{}\"", rule.font.style));
            }

            let font_style = if font_style_parts.is_empty() {
                "null".to_string()
            } else {
                format!("[{}]", font_style_parts.join(", "))
            };

            entries.push(format!(
                "    {{ \"scope\": \"{}\", \"settings\": {{ \"fontStyle\": {font_style} }} }}",
                rule.scope
            ));
        }

        let body = entries.join(",\n");
        Ok(format!(
            r#"{{
  "polyfont.tokenColors": [
{body}
  ]
}}"#
        ))
    }
}

#[cfg(test)]
mod tests {
    use polyfont_config::{DefaultFontConfig, FontConfig, RuleConfig};
    use polyfont_core::{FontStyle, FontWeight};

    use super::*;

    fn sample_config() -> PolyfontConfig {
        PolyfontConfig {
            version: 1,
            default: Some(DefaultFontConfig {
                family: "Fira Code".to_string(),
                fallbacks: vec![],
                weight: FontWeight::default(),
                style: FontStyle::default(),
                size: None,
            }),
            rules: vec![
                RuleConfig {
                    scope: "keyword".to_string(),
                    font: FontConfig {
                        family: "Maple Mono".to_string(),
                        fallbacks: vec![],
                        weight: FontWeight::Bold,
                        style: FontStyle::default(),
                        size: None,
                    },
                },
                RuleConfig {
                    scope: "comment".to_string(),
                    font: FontConfig {
                        family: "IBM Plex Mono".to_string(),
                        fallbacks: vec![],
                        weight: FontWeight::default(),
                        style: FontStyle::Italic,
                        size: None,
                    },
                },
            ],
        }
    }

    #[test]
    fn export_config_produces_valid_toml() {
        let config = sample_config();
        let output = ThemeExporter::export_config(&config).unwrap();

        let parsed: PolyfontConfig = toml::from_str(&output).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.rules.len(), 2);
        assert_eq!(parsed.rules[0].scope, "keyword");
        assert_eq!(parsed.rules[0].font.family, "Maple Mono");
        assert_eq!(parsed.rules[1].scope, "comment");
        assert_eq!(parsed.rules[1].font.style, FontStyle::Italic);
    }

    #[test]
    fn export_vscode_produces_valid_json() {
        let config = sample_config();
        let output = ThemeExporter::export_vscode(&config).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let colors = parsed["polyfont.tokenColors"].as_array().unwrap();
        assert_eq!(colors.len(), 2);
        assert_eq!(colors[0]["scope"], "keyword");
        assert_eq!(colors[1]["scope"], "comment");
    }

    #[test]
    fn export_vscode_font_style_null_when_default() {
        let config = PolyfontConfig {
            version: 1,
            default: Some(DefaultFontConfig {
                family: "Fira Code".to_string(),
                fallbacks: vec![],
                weight: FontWeight::default(),
                style: FontStyle::default(),
                size: None,
            }),
            rules: vec![RuleConfig {
                scope: "string".to_string(),
                font: FontConfig {
                    family: "Source Code Pro".to_string(),
                    fallbacks: vec![],
                    weight: FontWeight::default(),
                    style: FontStyle::default(),
                    size: None,
                },
            }],
        };
        let output = ThemeExporter::export_vscode(&config).unwrap();
        assert!(output.contains("null"));
    }
}
