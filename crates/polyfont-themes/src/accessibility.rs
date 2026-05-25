use polyfont_config::PolyfontConfig;

#[cfg(test)]
use polyfont_config::{DefaultFontConfig, FontConfig};

const MAX_FONT_FAMILIES: usize = 6;
const MIN_FONT_SIZE_PT: f32 = 8.0;

#[derive(Debug, Clone, PartialEq)]
pub enum AccessibilityIssue {
    LowContrastFonts,
    SimilarFonts(String, String),
    MissingFallback,
    TooManyFonts(usize),
    VerySmallSize(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccessibilityReport {
    pub issues: Vec<AccessibilityIssue>,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontCategory {
    DyslexiaFriendly,
    HighLegibility,
    VariableWeight,
    Monospace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FontRecommendation {
    pub family: String,
    pub reason: String,
    pub category: FontCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContrastResult {
    pub fonts_similar: bool,
    pub recommendation: Option<String>,
}

pub struct AccessibilityChecker;

impl AccessibilityChecker {
    pub fn check_config(config: &PolyfontConfig) -> AccessibilityReport {
        let mut issues = Vec::new();

        let mut families: Vec<String> = Vec::new();

        if let Some(ref default) = config.default {
            families.push(default.family.clone());
            collect_size_issue(&default.size, &mut issues);
            collect_fallback_issue(&default.fallbacks, &mut issues);
        }

        for rule in &config.rules {
            if !families.contains(&rule.font.family) {
                families.push(rule.font.family.clone());
            }
            collect_size_issue(&rule.font.size, &mut issues);
            collect_fallback_issue(&rule.font.fallbacks, &mut issues);
        }

        if families.len() > MAX_FONT_FAMILIES {
            issues.push(AccessibilityIssue::TooManyFonts(families.len()));
        }

        for i in 0..families.len() {
            for j in (i + 1)..families.len() {
                if names_are_similar(&families[i], &families[j]) {
                    issues.push(AccessibilityIssue::SimilarFonts(
                        families[i].clone(),
                        families[j].clone(),
                    ));
                }
            }
        }

        let score = compute_score(&issues);

        AccessibilityReport { issues, score }
    }

    #[must_use]
    pub fn recommend_dyslexia_fonts() -> Vec<FontRecommendation> {
        vec![
            FontRecommendation {
                family: "OpenDyslexic".to_string(),
                reason: "Designed specifically for dyslexic readers with weighted bottoms"
                    .to_string(),
                category: FontCategory::DyslexiaFriendly,
            },
            FontRecommendation {
                family: "Atkinson Hyperlegible".to_string(),
                reason: "Created by Braille Institute for maximum legibility".to_string(),
                category: FontCategory::HighLegibility,
            },
            FontRecommendation {
                family: "Lexend".to_string(),
                reason: "Designed to improve reading proficiency with variable spacing".to_string(),
                category: FontCategory::VariableWeight,
            },
            FontRecommendation {
                family: "Comic Neue".to_string(),
                reason: "Informal style reduces letter confusion for some dyslexic readers"
                    .to_string(),
                category: FontCategory::DyslexiaFriendly,
            },
            FontRecommendation {
                family: "Read Regular".to_string(),
                reason: "Purpose-built for dyslexic readers with distinct letter shapes"
                    .to_string(),
                category: FontCategory::DyslexiaFriendly,
            },
        ]
    }

    #[must_use]
    pub fn check_contrast_between_fonts(font1: &str, font2: &str) -> ContrastResult {
        if names_are_similar(font1, font2) {
            return ContrastResult {
                fonts_similar: true,
                recommendation: Some(format!(
                    "Fonts '{font1}' and '{font2}' appear similar. Consider using more visually distinct fonts."
                )),
            };
        }

        ContrastResult {
            fonts_similar: false,
            recommendation: None,
        }
    }
}

fn collect_size_issue(size: &Option<f32>, issues: &mut Vec<AccessibilityIssue>) {
    if let Some(s) = size
        && *s < MIN_FONT_SIZE_PT
    {
        issues.push(AccessibilityIssue::VerySmallSize(*s));
    }
}

fn collect_fallback_issue(fallbacks: &[String], issues: &mut Vec<AccessibilityIssue>) {
    if fallbacks.is_empty() {
        issues.push(AccessibilityIssue::MissingFallback);
    }
}

fn names_are_similar(a: &str, b: &str) -> bool {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    if a_lower == b_lower {
        return true;
    }

    let a_words: Vec<&str> = a_lower.split_whitespace().collect();
    let b_words: Vec<&str> = b_lower.split_whitespace().collect();

    for word in &a_words {
        if b_words.contains(word) && *word != "mono" && *word != "code" && *word != "font" {
            return true;
        }
    }

    let a_base = a_lower.split(' ').next().unwrap_or(&a_lower);
    let b_base = b_lower.split(' ').next().unwrap_or(&b_lower);

    if a_base.len() >= 4 && b_base.len() >= 4 {
        let prefix_len = a_base.len().min(b_base.len()).min(5);
        if a_base[..prefix_len] == b_base[..prefix_len] {
            return true;
        }
    }

    false
}

fn compute_score(issues: &[AccessibilityIssue]) -> f32 {
    if issues.is_empty() {
        return 1.0;
    }

    let penalty: f32 = issues
        .iter()
        .map(|issue| match issue {
            AccessibilityIssue::TooManyFonts(_) => 0.15,
            AccessibilityIssue::SimilarFonts(_, _) => 0.10,
            AccessibilityIssue::VerySmallSize(_) => 0.12,
            AccessibilityIssue::MissingFallback => 0.08,
            AccessibilityIssue::LowContrastFonts => 0.10,
        })
        .sum();

    (1.0 - penalty).max(0.0)
}

#[cfg(test)]
fn make_config(families: &[&str]) -> PolyfontConfig {
    let default = if families.is_empty() {
        None
    } else {
        Some(DefaultFontConfig {
            family: families[0].to_string(),
            fallbacks: vec!["sans-serif".to_string()],
            weight: polyfont_core::FontWeight::default(),
            style: polyfont_core::FontStyle::default(),
            size: Some(12.0),
            axes: vec![],
        })
    };

    let rules = families[1..]
        .iter()
        .map(|&family| {
            use polyfont_config::RuleConfig;
            RuleConfig {
                scope: family.to_lowercase().replace(' ', "_"),
                font: FontConfig {
                    family: family.to_string(),
                    fallbacks: vec!["sans-serif".to_string()],
                    weight: polyfont_core::FontWeight::default(),
                    style: polyfont_core::FontStyle::default(),
                    size: Some(12.0),
                    axes: vec![],
                },
            }
        })
        .collect();

    PolyfontConfig {
        version: 1,
        default,
        rules,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommend_dyslexia_fonts_not_empty() {
        let recs = AccessibilityChecker::recommend_dyslexia_fonts();
        assert!(!recs.is_empty());
        assert!(recs.iter().any(|r| r.family == "OpenDyslexic"));
        assert!(recs.iter().any(|r| r.family == "Atkinson Hyperlegible"));
        assert!(recs.iter().any(|r| r.family == "Lexend"));
        assert!(recs.iter().any(|r| r.family == "Comic Neue"));
        assert!(recs.iter().any(|r| r.family == "Read Regular"));
    }

    #[test]
    fn test_check_config_too_many_fonts() {
        let families = [
            "Fira Code",
            "JetBrains Mono",
            "IBM Plex Mono",
            "Source Code Pro",
            "Maple Mono",
            "Monaspace Neon",
            "Monaspace Argon",
        ];
        let config = make_config(&families);
        let report = AccessibilityChecker::check_config(&config);
        assert!(
            report
                .issues
                .iter()
                .any(|i| matches!(i, AccessibilityIssue::TooManyFonts(n) if *n == 7))
        );
        assert!(report.score < 1.0);
    }

    #[test]
    fn test_check_config_good_score() {
        let config = make_config(&["Fira Code", "Source Code Pro"]);
        let report = AccessibilityChecker::check_config(&config);
        assert!(report.score >= 0.8);
    }

    #[test]
    fn test_check_contrast_similar_fonts() {
        let result = AccessibilityChecker::check_contrast_between_fonts("Fira Code", "Fira Mono");
        assert!(result.fonts_similar);
        assert!(result.recommendation.is_some());
    }

    #[test]
    fn test_check_contrast_different_fonts() {
        let result = AccessibilityChecker::check_contrast_between_fonts("Fira Code", "Comic Neue");
        assert!(!result.fonts_similar);
        assert!(result.recommendation.is_none());
    }
}
