use std::path::{Path, PathBuf};

use polyfont_core::{AxisValue, FontRule, FontSpec, FontStyle, FontWeight};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

const CONFIG_FILENAME: &str = ".polyfont.toml";
const CURRENT_CONFIG_VERSION: u32 = 1;

/// Top-level polyfont configuration loaded from `.polyfont.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyfontConfig {
    pub version: u32,
    #[serde(default)]
    pub default: Option<DefaultFontConfig>,
    #[serde(default)]
    pub rules: Vec<RuleConfig>,
}

/// Default font specification applied when no scope-specific rule matches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultFontConfig {
    pub family: String,
    #[serde(default)]
    pub fallbacks: Vec<String>,
    #[serde(default = "FontWeight::default")]
    pub weight: FontWeight,
    #[serde(default = "FontStyle::default")]
    pub style: FontStyle,
    #[serde(default)]
    pub size: Option<f32>,
    #[serde(default)]
    pub axes: Vec<AxisValue>,
}

/// A single scope-to-font mapping rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub scope: String,
    pub font: FontConfig,
}

/// Font configuration within a rule (family, weight, style, fallbacks).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    #[serde(default)]
    pub fallbacks: Vec<String>,
    #[serde(default = "FontWeight::default")]
    pub weight: FontWeight,
    #[serde(default = "FontStyle::default")]
    pub style: FontStyle,
    #[serde(default)]
    pub size: Option<f32>,
    #[serde(default)]
    pub axes: Vec<AxisValue>,
}

impl PolyfontConfig {
    /// Validate this configuration for completeness and correctness.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::UnsupportedVersion`] if `version` is not the
    /// current version. Returns [`ConfigError::Validation`] if any font family
    /// or scope is empty.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.version != CURRENT_CONFIG_VERSION {
            return Err(ConfigError::UnsupportedVersion {
                found: self.version,
                expected: CURRENT_CONFIG_VERSION,
            });
        }

        if let Some(ref default) = self.default
            && default.family.trim().is_empty()
        {
            return Err(ConfigError::Validation(
                "default font family must not be empty".into(),
            ));
        }

        for (i, rule) in self.rules.iter().enumerate() {
            if rule.scope.trim().is_empty() {
                return Err(ConfigError::Validation(format!(
                    "rule at index {i} has an empty scope"
                )));
            }
            if rule.font.family.trim().is_empty() {
                return Err(ConfigError::Validation(format!(
                    "rule at index {i} (scope '{}') has an empty font family",
                    rule.scope
                )));
            }
        }

        Ok(())
    }

    /// Convert config into a flat list of [`FontRule`]s, including a default catchall.
    #[must_use]
    pub fn to_rules(&self) -> Vec<FontRule> {
        let mut rules: Vec<FontRule> = self
            .rules
            .iter()
            .map(|r| FontRule {
                scope: r.scope.clone(),
                font: FontSpec {
                    family: r.font.family.clone(),
                    fallbacks: r.font.fallbacks.clone(),
                    weight: r.font.weight,
                    style: r.font.style,
                    size: r.font.size,
                    axes: r.font.axes.clone(),
                },
            })
            .collect();

        if let Some(ref default) = self.default {
            rules.push(FontRule {
                scope: "*".to_string(),
                font: FontSpec {
                    family: default.family.clone(),
                    fallbacks: default.fallbacks.clone(),
                    weight: default.weight,
                    style: default.style,
                    size: default.size,
                    axes: default.axes.clone(),
                },
            });
        }

        rules
    }

    /// Overlay another config on top of this one. The overlay takes precedence
    /// for matching keys.
    #[must_use]
    pub fn merge(base: Self, overlay: Self) -> Self {
        let rules = overlay.rules;
        let default = overlay.default.or(base.default);
        Self {
            version: base.version,
            default,
            rules,
        }
    }
}

/// Errors that can occur during configuration loading or validation.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unsupported config version: found {found}, expected {expected}")]
    UnsupportedVersion { found: u32, expected: u32 },

    #[error("config validation failed: {0}")]
    Validation(String),

    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("no config file found searching from {0}")]
    NotFound(PathBuf),
}

/// Filesystem-based configuration loader with ancestor directory search.
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load config from a specific file path.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Io`] if the file cannot be read,
    /// [`ConfigError::Parse`] if the TOML is invalid, or any error from
    /// [`PolyfontConfig::validate`].
    pub fn load_from_path(path: &Path) -> Result<PolyfontConfig, ConfigError> {
        info!("loading config from {}", path.display());
        let content = std::fs::read_to_string(path)?;
        let config: PolyfontConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Load config by searching for `.polyfont.toml` in the given directory and
    /// its ancestors.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::NotFound`] if no config file is found, or any
    /// error from [`load_from_path`](Self::load_from_path).
    pub fn load_from_dir(start_dir: &Path) -> Result<PolyfontConfig, ConfigError> {
        let config_path = Self::find_config(start_dir)
            .ok_or_else(|| ConfigError::NotFound(start_dir.to_path_buf()))?;
        Self::load_from_path(&config_path)
    }

    /// Locate the `.polyfont.toml` file by walking up from the given directory.
    ///
    /// # Errors
    ///
    /// Returns `None` if no config file is found or the start directory cannot
    /// be canonicalized.
    pub fn find_config(start_dir: &Path) -> Option<PathBuf> {
        let canonical = start_dir.canonicalize().ok()?;
        let mut current = canonical.as_path();

        loop {
            let candidate = current.join(CONFIG_FILENAME);
            debug!("checking for config at {}", candidate.display());
            if candidate.is_file() {
                info!("found config at {}", candidate.display());
                return Some(candidate);
            }

            if let Some(parent) = current.parent() {
                current = parent;
            } else {
                warn!(
                    "no {} found searching up from {}",
                    CONFIG_FILENAME,
                    start_dir.display()
                );
                return None;
            }
        }
    }

    /// Load the primary config and merge with any overlay configs.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::NotFound`] if no config files are found, or any
    /// error from [`load_from_path`](Self::load_from_path) on the primary config.
    ///
    /// # Panics
    ///
    /// Panics if the non-empty config list is somehow empty after the check
    /// (should never happen).
    pub fn load_merged(start_dir: &Path) -> Result<PolyfontConfig, ConfigError> {
        let configs = Self::find_all_configs(start_dir);
        if configs.is_empty() {
            return Err(ConfigError::NotFound(start_dir.to_path_buf()));
        }

        let mut iter = configs.into_iter();
        let first = Self::load_from_path(&iter.next().expect("configs verified non-empty above"))?;
        let merged = iter.fold(first, |acc, path| match Self::load_from_path(&path) {
            Ok(overlay) => PolyfontConfig::merge(acc, overlay),
            Err(e) => {
                warn!("skipping config at {}: {e}", path.display());
                acc
            }
        });

        Ok(merged)
    }

    fn find_all_configs(start_dir: &Path) -> Vec<PathBuf> {
        let Ok(canonical) = start_dir.canonicalize() else {
            return Vec::new();
        };

        let mut configs: Vec<PathBuf> = WalkDir::new(&canonical)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.file_name() == CONFIG_FILENAME)
            .map(walkdir::DirEntry::into_path)
            .collect();

        configs.sort_by_key(|p| p.components().count());

        let mut ancestor_configs: Vec<PathBuf> = Vec::new();
        let mut current = canonical.as_path();
        while let Some(parent) = current.parent() {
            let candidate = parent.join(CONFIG_FILENAME);
            if candidate.is_file() {
                ancestor_configs.push(candidate);
            }
            current = parent;
        }

        ancestor_configs.reverse();
        ancestor_configs.extend(configs);
        ancestor_configs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_toml() -> &'static str {
        r#"
version = 1

[default]
family = "Fira Code"

[[rules]]
scope = "keyword"
[rules.font]
family = "Maple Mono"
weight = "bold"
"#
    }

    fn full_toml() -> &'static str {
        r#"
version = 1

[default]
family = "Fira Code"
fallbacks = ["JetBrains Mono", "monospace"]
weight = "regular"
style = "normal"

[[rules]]
scope = "keyword"
[rules.font]
family = "Maple Mono"
weight = "bold"

[[rules]]
scope = "comment"
[rules.font]
family = "IBM Plex Mono"
style = "italic"

[[rules]]
scope = "string"
[rules.font]
family = "Source Code Pro"
weight = "light"

[[rules]]
scope = "entity.name.function"
[rules.font]
family = "Fira Code"
weight = "semi-bold"

[[rules]]
scope = "variable"
[rules.font]
family = "JetBrains Mono"

[[rules]]
scope = "constant"
[rules.font]
family = "Monaspace Neon"
weight = "bold"

[[rules]]
scope = "support.function"
[rules.font]
family = "Monaspace Argon"
"#
    }

    #[test]
    fn parse_minimal_config() {
        let config: PolyfontConfig = toml::from_str(minimal_toml()).unwrap();
        assert_eq!(config.version, 1);
        assert!(config.default.is_some());
        assert_eq!(config.rules.len(), 1);
    }

    #[test]
    fn parse_full_config() {
        let config: PolyfontConfig = toml::from_str(full_toml()).unwrap();
        assert_eq!(config.version, 1);
        assert!(config.default.is_some());
        assert_eq!(config.rules.len(), 7);

        let default = config.default.as_ref().unwrap();
        assert_eq!(default.family, "Fira Code");
        assert_eq!(default.fallbacks, vec!["JetBrains Mono", "monospace"]);

        assert_eq!(config.rules[0].scope, "keyword");
        assert_eq!(config.rules[0].font.family, "Maple Mono");
        assert_eq!(config.rules[0].font.weight, FontWeight::Bold);

        assert_eq!(config.rules[1].scope, "comment");
        assert_eq!(config.rules[1].font.style, FontStyle::Italic);

        assert_eq!(config.rules[2].scope, "string");
        assert_eq!(config.rules[2].font.weight, FontWeight::Light);

        assert_eq!(config.rules[3].scope, "entity.name.function");
        assert_eq!(config.rules[3].font.weight, FontWeight::SemiBold);
    }

    #[test]
    fn validate_rejects_bad_version() {
        let config = PolyfontConfig {
            version: 99,
            default: None,
            rules: vec![],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_scope() {
        let config = PolyfontConfig {
            version: 1,
            default: None,
            rules: vec![RuleConfig {
                scope: "  ".to_string(),
                font: FontConfig {
                    family: "Test".to_string(),
                    fallbacks: vec![],
                    weight: FontWeight::default(),
                    style: FontStyle::default(),
                    size: None,
                    axes: vec![],
                },
            }],
        };
        let err = config.validate().unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn validate_rejects_empty_family() {
        let config = PolyfontConfig {
            version: 1,
            default: None,
            rules: vec![RuleConfig {
                scope: "keyword".to_string(),
                font: FontConfig {
                    family: "  ".to_string(),
                    fallbacks: vec![],
                    weight: FontWeight::default(),
                    style: FontStyle::default(),
                    size: None,
                    axes: vec![],
                },
            }],
        };
        let err = config.validate().unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn validate_rejects_empty_default_family() {
        let config = PolyfontConfig {
            version: 1,
            default: Some(DefaultFontConfig {
                family: "  ".to_string(),
                fallbacks: vec![],
                weight: FontWeight::default(),
                style: FontStyle::default(),
                size: None,
                axes: vec![],
            }),
            rules: vec![],
        };
        let err = config.validate().unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn to_rules_includes_default_as_catchall() {
        let config: PolyfontConfig = toml::from_str(minimal_toml()).unwrap();
        let rules = config.to_rules();
        assert_eq!(rules.len(), 2);
        let catchall = rules.last().unwrap();
        assert_eq!(catchall.scope, "*");
        assert_eq!(catchall.font.family, "Fira Code");
    }

    #[test]
    fn merge_overlay_takes_precedence() {
        let base: PolyfontConfig = toml::from_str(minimal_toml()).unwrap();
        let overlay = PolyfontConfig {
            version: 1,
            default: None,
            rules: vec![RuleConfig {
                scope: "comment".to_string(),
                font: FontConfig {
                    family: "Override".to_string(),
                    fallbacks: vec![],
                    weight: FontWeight::default(),
                    style: FontStyle::Italic,
                    size: None,
                    axes: vec![],
                },
            }],
        };
        let merged = PolyfontConfig::merge(base, overlay);
        assert_eq!(merged.rules.len(), 1);
        assert_eq!(merged.rules[0].scope, "comment");
        assert!(merged.default.is_some());
    }
}
