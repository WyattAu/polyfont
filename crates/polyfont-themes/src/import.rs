use polyfont_config::PolyfontConfig;

use crate::ThemeError;
use crate::mapping::{FontMapping, build_config_from_entries};

pub struct ThemeImporter;

impl ThemeImporter {
    pub fn import_vscode_theme(
        &self,
        json: &str,
        font_mapping: &FontMapping,
    ) -> Result<PolyfontConfig, ThemeError> {
        let value: serde_json::Value = serde_json::from_str(json)?;

        let token_colors = value
            .get("tokenColors")
            .or_else(|| value.get("settings"))
            .and_then(|s| s.as_array())
            .ok_or_else(|| ThemeError::Validation("missing tokenColors array".into()))?;

        let mut entries = Vec::new();
        for item in token_colors {
            let scope = item.get("scope").and_then(|s| s.as_str()).unwrap_or("*");

            let font_style = item
                .get("settings")
                .and_then(|s| s.get("fontStyle"))
                .and_then(|f| f.as_str())
                .unwrap_or("");

            for s in scope.split(',') {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    entries.push((trimmed.to_string(), font_style.to_string()));
                }
            }
        }

        Ok(build_config_from_entries(entries, font_mapping))
    }

    pub fn import_textmate_theme(
        &self,
        xml: &str,
        font_mapping: &FontMapping,
    ) -> Result<PolyfontConfig, ThemeError> {
        let mut entries = Vec::new();
        let mut pos = 0;

        while let Some(idx) = xml[pos..].find("<key>scope</key>") {
            let abs_idx = pos + idx;
            let after_key = &xml[abs_idx + "<key>scope</key>".len()..];

            if let Some(str_start) = after_key.find("<string>") {
                let val_start = str_start + "<string>".len();
                if let Some(str_end) = after_key[val_start..].find("</string>") {
                    let scope = after_key[val_start..val_start + str_end].trim().to_string();
                    if !scope.is_empty() {
                        if let Some((weight, style)) = extract_font_style_from_dict(after_key) {
                            let font_style_str = format!("{} {}", weight, style).trim().to_string();
                            entries.push((scope, font_style_str));
                        } else {
                            entries.push((scope, String::new()));
                        }
                    }
                    pos = abs_idx + "<key>scope</key>".len() + val_start + str_end;
                } else {
                    pos = abs_idx + "<key>scope</key>".len();
                }
            } else {
                pos = abs_idx + "<key>scope</key>".len();
            }
        }

        Ok(build_config_from_entries(entries, font_mapping))
    }
}

fn extract_font_style_from_dict(remaining: &str) -> Option<(String, String)> {
    let mut weight = String::new();
    let mut style = String::new();

    let tag_patterns = ["<key>fontStyle</key>", "<string>fontStyle</string>"];
    for pattern in &tag_patterns {
        if let Some(pos) = remaining.find(pattern) {
            let after = &remaining[pos + pattern.len()..];
            if let Some(s) = after.find("<string>") {
                let start = s + "<string>".len();
                if let Some(e) = after[start..].find("</string>") {
                    let val = &after[start..start + e];
                    for token in val.split_whitespace() {
                        match token {
                            "bold" => weight = "bold".to_string(),
                            "italic" => style = "italic".to_string(),
                            _ => {}
                        }
                    }
                }
            }
            break;
        }
    }

    if weight.is_empty() && style.is_empty() {
        None
    } else {
        Some((weight, style))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_vscode_theme_basic() {
        let json = r#"{
            "tokenColors": [
                { "scope": "keyword", "settings": { "fontStyle": "bold" } },
                { "scope": "comment", "settings": { "fontStyle": "italic" } },
                { "scope": "string", "settings": {} }
            ]
        }"#;

        let importer = ThemeImporter;
        let mapping = FontMapping::default_mapping();
        let config = importer.import_vscode_theme(json, &mapping).unwrap();

        assert_eq!(config.version, 1);
        assert!(config.default.is_some());
        assert_eq!(config.rules.len(), 3);

        assert_eq!(config.rules[0].scope, "keyword");
        assert_eq!(config.rules[0].font.family, "Maple Mono");
        assert_eq!(config.rules[0].font.weight, polyfont_core::FontWeight::Bold);

        assert_eq!(config.rules[1].scope, "comment");
        assert_eq!(config.rules[1].font.style, polyfont_core::FontStyle::Italic);

        assert_eq!(config.rules[2].scope, "string");
        assert_eq!(config.rules[2].font.family, "Source Code Pro");
    }

    #[test]
    fn import_vscode_theme_multi_scope() {
        let json = r#"{
            "tokenColors": [
                { "scope": "keyword,storage", "settings": { "fontStyle": "bold" } }
            ]
        }"#;

        let importer = ThemeImporter;
        let mapping = FontMapping::default_mapping();
        let config = importer.import_vscode_theme(json, &mapping).unwrap();

        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].scope, "keyword");
        assert_eq!(config.rules[1].scope, "storage");
    }

    #[test]
    fn import_vscode_theme_invalid_json() {
        let importer = ThemeImporter;
        let mapping = FontMapping::default_mapping();
        let result = importer.import_vscode_theme("not json", &mapping);
        assert!(result.is_err());
    }

    #[test]
    fn import_vscode_theme_missing_token_colors() {
        let json = r#"{"name": "test"}"#;
        let importer = ThemeImporter;
        let mapping = FontMapping::default_mapping();
        let result = importer.import_vscode_theme(json, &mapping);
        assert!(result.is_err());
    }

    #[test]
    fn import_textmate_theme_simple() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
    <key>settings</key>
    <array>
        <dict>
            <key>scope</key>
            <string>keyword</string>
            <key>settings</key>
            <dict>
                <key>fontStyle</key>
                <string>bold</string>
            </dict>
        </dict>
        <dict>
            <key>scope</key>
            <string>comment</string>
            <key>settings</key>
            <dict>
                <key>fontStyle</key>
                <string>italic</string>
            </dict>
        </dict>
    </array>
</dict>
</plist>"#;

        let importer = ThemeImporter;
        let mapping = FontMapping::default_mapping();
        let config = importer.import_textmate_theme(xml, &mapping).unwrap();

        assert_eq!(config.version, 1);
        assert!(config.default.is_some());
        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].scope, "keyword");
        assert_eq!(config.rules[0].font.weight, polyfont_core::FontWeight::Bold);
        assert_eq!(config.rules[1].scope, "comment");
        assert_eq!(config.rules[1].font.style, polyfont_core::FontStyle::Italic);
    }

    #[test]
    fn import_vscode_theme_validates() {
        let json = r#"{
            "tokenColors": [
                { "scope": "keyword", "settings": { "fontStyle": "bold" } }
            ]
        }"#;

        let importer = ThemeImporter;
        let mapping = FontMapping::default_mapping();
        let config = importer.import_vscode_theme(json, &mapping).unwrap();
        assert!(config.validate().is_ok());
    }
}
