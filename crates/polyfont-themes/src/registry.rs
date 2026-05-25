use polyfont_config::{DefaultFontConfig, FontConfig, PolyfontConfig, RuleConfig};
use polyfont_core::{FontStyle, FontWeight};

#[derive(Debug, Clone)]
pub struct ThemeInfo {
    pub name: String,
    pub description: String,
}

pub struct BuiltinTheme {
    pub name: String,
    pub description: String,
    pub config: PolyfontConfig,
}

pub struct ThemeRegistry;

impl ThemeRegistry {
    pub fn list_themes(&self) -> Vec<ThemeInfo> {
        static CACHE: std::sync::OnceLock<Vec<ThemeInfo>> = std::sync::OnceLock::new();
        CACHE
            .get_or_init(|| {
                builtin_themes()
                    .into_iter()
                    .map(|t| ThemeInfo {
                        name: t.name,
                        description: t.description,
                    })
                    .collect()
            })
            .clone()
    }

    pub fn get_theme(&self, name: &str) -> Option<&'static BuiltinTheme> {
        get_builtin_theme(name)
    }
}

fn get_builtin_theme(name: &str) -> Option<&'static BuiltinTheme> {
    static THEMES: std::sync::OnceLock<Vec<BuiltinTheme>> = std::sync::OnceLock::new();
    THEMES
        .get_or_init(builtin_themes)
        .iter()
        .find(|t| t.name == name)
}

fn builtin_themes() -> Vec<BuiltinTheme> {
    vec![
        BuiltinTheme {
            name: "monaspace".to_string(),
            description: "All Monaspace variants with weight differentiation".to_string(),
            config: PolyfontConfig {
                version: 1,
                default: Some(DefaultFontConfig {
                    family: "Monaspace Xenon".to_string(),
                    fallbacks: vec!["monospace".to_string()],
                    weight: FontWeight::Regular,
                    style: FontStyle::Normal,
                    size: None,
                    axes: vec![],
                }),
                rules: vec![
                    RuleConfig {
                        scope: "keyword".to_string(),
                        font: FontConfig {
                            family: "Monaspace Neon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "comment".to_string(),
                        font: FontConfig {
                            family: "Monaspace Argon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Light,
                            style: FontStyle::Italic,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "string".to_string(),
                        font: FontConfig {
                            family: "Monaspace Krypton".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Regular,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.function".to_string(),
                        font: FontConfig {
                            family: "Monaspace Argon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::SemiBold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.type".to_string(),
                        font: FontConfig {
                            family: "Monaspace Neon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::SemiBold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "variable".to_string(),
                        font: FontConfig {
                            family: "Monaspace Xenon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Regular,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "constant".to_string(),
                        font: FontConfig {
                            family: "Monaspace Radon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "support.function".to_string(),
                        font: FontConfig {
                            family: "Monaspace Krypton".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Medium,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "storage.type".to_string(),
                        font: FontConfig {
                            family: "Monaspace Neon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Italic,
                            size: None,
                            axes: vec![],
                        },
                    },
                ],
            },
        },
        BuiltinTheme {
            name: "serif-mono".to_string(),
            description: "Serif fonts for comments, sans-serif for code".to_string(),
            config: PolyfontConfig {
                version: 1,
                default: Some(DefaultFontConfig {
                    family: "IBM Plex Mono".to_string(),
                    fallbacks: vec!["monospace".to_string()],
                    weight: FontWeight::Regular,
                    style: FontStyle::Normal,
                    size: None,
                    axes: vec![],
                }),
                rules: vec![
                    RuleConfig {
                        scope: "comment".to_string(),
                        font: FontConfig {
                            family: "Source Serif 4".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Regular,
                            style: FontStyle::Italic,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "comment.block.documentation".to_string(),
                        font: FontConfig {
                            family: "Source Serif 4".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Medium,
                            style: FontStyle::Italic,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "keyword".to_string(),
                        font: FontConfig {
                            family: "IBM Plex Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "string".to_string(),
                        font: FontConfig {
                            family: "IBM Plex Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Light,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.function".to_string(),
                        font: FontConfig {
                            family: "IBM Plex Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::SemiBold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.type".to_string(),
                        font: FontConfig {
                            family: "IBM Plex Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Medium,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "variable".to_string(),
                        font: FontConfig {
                            family: "IBM Plex Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Regular,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "constant".to_string(),
                        font: FontConfig {
                            family: "IBM Plex Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "support.function".to_string(),
                        font: FontConfig {
                            family: "IBM Plex Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Medium,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "storage.type".to_string(),
                        font: FontConfig {
                            family: "IBM Plex Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                ],
            },
        },
        BuiltinTheme {
            name: "weight-differentiated".to_string(),
            description: "Same font family, different weights per scope".to_string(),
            config: PolyfontConfig {
                version: 1,
                default: Some(DefaultFontConfig {
                    family: "JetBrains Mono".to_string(),
                    fallbacks: vec!["monospace".to_string()],
                    weight: FontWeight::Regular,
                    style: FontStyle::Normal,
                    size: None,
                    axes: vec![],
                }),
                rules: vec![
                    RuleConfig {
                        scope: "keyword".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "comment".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Light,
                            style: FontStyle::Italic,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "string".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Thin,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.function".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::SemiBold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.type".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Medium,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "variable".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Regular,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "constant".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::ExtraBold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "support.function".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Medium,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "storage.type".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Black,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                ],
            },
        },
        BuiltinTheme {
            name: "minimal".to_string(),
            description: "Only 4 rules: keyword, comment, string, function".to_string(),
            config: PolyfontConfig {
                version: 1,
                default: Some(DefaultFontConfig {
                    family: "monospace".to_string(),
                    fallbacks: vec![],
                    weight: FontWeight::Regular,
                    style: FontStyle::Normal,
                    size: None,
                    axes: vec![],
                }),
                rules: vec![
                    RuleConfig {
                        scope: "keyword".to_string(),
                        font: FontConfig {
                            family: "monospace".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "comment".to_string(),
                        font: FontConfig {
                            family: "monospace".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Regular,
                            style: FontStyle::Italic,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "string".to_string(),
                        font: FontConfig {
                            family: "monospace".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Light,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.function".to_string(),
                        font: FontConfig {
                            family: "monospace".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::SemiBold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                ],
            },
        },
        BuiltinTheme {
            name: "maximal".to_string(),
            description: "17+ rules covering every common scope".to_string(),
            config: PolyfontConfig {
                version: 1,
                default: Some(DefaultFontConfig {
                    family: "JetBrains Mono".to_string(),
                    fallbacks: vec!["monospace".to_string()],
                    weight: FontWeight::Regular,
                    style: FontStyle::Normal,
                    size: None,
                    axes: vec![],
                }),
                rules: vec![
                    RuleConfig {
                        scope: "keyword".to_string(),
                        font: FontConfig {
                            family: "Maple Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "keyword.control".to_string(),
                        font: FontConfig {
                            family: "Maple Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::ExtraBold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "comment".to_string(),
                        font: FontConfig {
                            family: "IBM Plex Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Regular,
                            style: FontStyle::Italic,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "comment.block.documentation".to_string(),
                        font: FontConfig {
                            family: "IBM Plex Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Medium,
                            style: FontStyle::Italic,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "string".to_string(),
                        font: FontConfig {
                            family: "Source Code Pro".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Light,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "string.interpolated".to_string(),
                        font: FontConfig {
                            family: "Source Code Pro".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Medium,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.function".to_string(),
                        font: FontConfig {
                            family: "Monaspace Argon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::SemiBold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.function.method".to_string(),
                        font: FontConfig {
                            family: "Monaspace Argon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.type".to_string(),
                        font: FontConfig {
                            family: "Monaspace Neon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::SemiBold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.class".to_string(),
                        font: FontConfig {
                            family: "Monaspace Neon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.struct".to_string(),
                        font: FontConfig {
                            family: "Monaspace Neon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "entity.name.enum".to_string(),
                        font: FontConfig {
                            family: "Monaspace Neon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "variable".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Regular,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "variable.parameter".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Light,
                            style: FontStyle::Italic,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "constant".to_string(),
                        font: FontConfig {
                            family: "Monaspace Radon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "constant.numeric".to_string(),
                        font: FontConfig {
                            family: "Monaspace Radon".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::ExtraBold,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "support.function".to_string(),
                        font: FontConfig {
                            family: "Monaspace Krypton".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Medium,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "storage.type".to_string(),
                        font: FontConfig {
                            family: "Maple Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Bold,
                            style: FontStyle::Italic,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "punctuation".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Light,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                    RuleConfig {
                        scope: "punctuation.bracket".to_string(),
                        font: FontConfig {
                            family: "JetBrains Mono".to_string(),
                            fallbacks: vec![],
                            weight: FontWeight::Regular,
                            style: FontStyle::Normal,
                            size: None,
                            axes: vec![],
                        },
                    },
                ],
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_all_five_themes() {
        let registry = ThemeRegistry;
        let themes = registry.list_themes();
        assert_eq!(themes.len(), 5);

        let names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"monaspace"));
        assert!(names.contains(&"serif-mono"));
        assert!(names.contains(&"weight-differentiated"));
        assert!(names.contains(&"minimal"));
        assert!(names.contains(&"maximal"));
    }

    #[test]
    fn get_theme_returns_correct_theme() {
        let registry = ThemeRegistry;
        let theme = registry.get_theme("monaspace").unwrap();
        assert_eq!(theme.name, "monaspace");
    }

    #[test]
    fn get_theme_returns_none_for_unknown() {
        let registry = ThemeRegistry;
        assert!(registry.get_theme("nonexistent").is_none());
    }

    #[test]
    fn all_builtin_themes_are_valid() {
        let registry = ThemeRegistry;
        for info in registry.list_themes() {
            let theme = registry.get_theme(&info.name).unwrap();
            assert!(
                theme.config.validate().is_ok(),
                "theme '{}' failed validation",
                info.name
            );
        }
    }

    #[test]
    fn maximal_has_seventeen_plus_rules() {
        let registry = ThemeRegistry;
        let theme = registry.get_theme("maximal").unwrap();
        assert!(
            theme.config.rules.len() >= 17,
            "maximal theme has {} rules, expected >= 17",
            theme.config.rules.len()
        );
    }

    #[test]
    fn minimal_has_four_rules() {
        let registry = ThemeRegistry;
        let theme = registry.get_theme("minimal").unwrap();
        assert_eq!(theme.config.rules.len(), 4);
    }
}
