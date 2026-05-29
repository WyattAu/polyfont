use crate::{FontAssignment, FontRule, TokenInfo};

/// Core trait for scope-to-font resolution engines.
///
/// Implementations map [`TokenInfo`] scopes to [`FontAssignment`] values using
/// an ordered set of [`FontRule`]s. Rules are sorted by specificity so that
/// more specific scope patterns take priority.
pub trait PolyfontEngine: Send + Sync {
    /// Add a [`FontRule`] and re-sort by specificity.
    fn add_rule(&mut self, rule: FontRule);

    /// Remove all rules whose scope exactly matches `scope`.
    fn remove_rule(&mut self, scope: &str);

    /// Returns the current rules in specificity order (most specific first).
    fn rules(&self) -> &[FontRule];

    /// Resolve a single token to a [`FontAssignment`].
    ///
    /// Returns `None` when no rule's scope pattern matches the token.
    fn resolve_token(&self, token: &TokenInfo) -> Option<FontAssignment>;

    /// Resolve each token in `tokens` independently.
    ///
    /// See [`resolve_token`](PolyfontEngine::resolve_token).
    fn resolve_all(&self, tokens: &[TokenInfo]) -> Vec<Option<FontAssignment>>;

    /// Remove all rules, leaving the engine empty.
    fn clear(&mut self);
}

/// Scope matching engine that resolves TextMate scopes to font assignments.
///
/// Rules are kept sorted in descending [`FontRule::specificity`] order so that
/// the first matching rule during resolution is always the most specific one.
pub struct ScopeMatchEngine {
    rules: Vec<FontRule>,
}

impl ScopeMatchEngine {
    /// Create an empty engine with no rules.
    #[must_use]
    pub const fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Create an engine pre-loaded with `rules`, sorted by specificity.
    #[must_use]
    pub fn from_rules(rules: Vec<FontRule>) -> Self {
        let mut engine = Self { rules };
        engine.sort_rules();
        engine
    }

    fn sort_rules(&mut self) {
        self.rules
            .sort_by_key(|b| std::cmp::Reverse(b.specificity()));
    }

    /// Check if `scope` matches `pattern`.
    ///
    /// A `"*"` pattern matches any scope. Otherwise the pattern matches when it
    /// is either identical to `scope` or is a prefix followed by a `.` separator
    /// (i.e. hierarchical TextMate scope matching).
    #[must_use]
    pub fn scope_matches(scope: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if scope == pattern {
            return true;
        }
        scope.len() > pattern.len()
            && scope.as_bytes()[pattern.len()] == b'.'
            && scope.starts_with(pattern)
    }
}

impl PolyfontEngine for ScopeMatchEngine {
    fn add_rule(&mut self, rule: FontRule) {
        self.rules.push(rule);
        self.sort_rules();
    }

    fn remove_rule(&mut self, scope: &str) {
        self.rules.retain(|r| r.scope != scope);
    }

    fn rules(&self) -> &[FontRule] {
        &self.rules
    }

    fn resolve_token(&self, token: &TokenInfo) -> Option<FontAssignment> {
        for rule in &self.rules {
            if Self::scope_matches(&token.scope, &rule.scope) {
                return Some(FontAssignment {
                    scope: token.scope.clone(),
                    font: rule.font.clone(),
                    specificity: rule.specificity(),
                    is_active: true,
                });
            }
        }
        None
    }

    fn resolve_all(&self, tokens: &[TokenInfo]) -> Vec<Option<FontAssignment>> {
        tokens.iter().map(|t| self.resolve_token(t)).collect()
    }

    fn clear(&mut self) {
        self.rules.clear();
    }
}

impl Default for ScopeMatchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontRule, FontSpec};
    use crate::token::{Position, Range, TokenInfo};

    #[test]
    fn test_new_engine_has_no_rules() {
        let engine = ScopeMatchEngine::new();
        assert!(engine.rules().is_empty());
    }

    #[test]
    fn test_add_rule() {
        let mut engine = ScopeMatchEngine::new();
        engine.add_rule(FontRule {
            scope: "keyword".to_string(),
            font: FontSpec::default_font("TestFont"),
        });
        assert_eq!(engine.rules().len(), 1);
    }

    #[test]
    fn test_remove_rule() {
        let mut engine = ScopeMatchEngine::new();
        engine.add_rule(FontRule {
            scope: "keyword".to_string(),
            font: FontSpec::default_font("TestFont"),
        });
        engine.remove_rule("keyword");
        assert!(engine.rules().is_empty());
    }

    #[test]
    fn test_scope_matches_exact() {
        assert!(ScopeMatchEngine::scope_matches("keyword", "keyword"));
    }

    #[test]
    fn test_scope_matches_hierarchical() {
        assert!(ScopeMatchEngine::scope_matches(
            "keyword.control",
            "keyword"
        ));
        assert!(ScopeMatchEngine::scope_matches(
            "entity.name.function",
            "entity"
        ));
        assert!(ScopeMatchEngine::scope_matches(
            "entity.name.function",
            "entity.name"
        ));
    }

    #[test]
    fn test_scope_matches_wildcard() {
        assert!(ScopeMatchEngine::scope_matches("anything", "*"));
        assert!(ScopeMatchEngine::scope_matches("keyword.control", "*"));
    }

    #[test]
    fn test_scope_no_false_positive() {
        assert!(!ScopeMatchEngine::scope_matches(
            "keyword",
            "keyword.control"
        ));
        assert!(!ScopeMatchEngine::scope_matches("keywordx", "keyword"));
    }

    #[test]
    fn test_resolve_token_most_specific_wins() {
        let mut engine = ScopeMatchEngine::new();
        engine.add_rule(FontRule {
            scope: "keyword".to_string(),
            font: FontSpec::default_font("FontA"),
        });
        engine.add_rule(FontRule {
            scope: "keyword.control".to_string(),
            font: FontSpec::default_font("FontB"),
        });

        let token = TokenInfo {
            text: "if".to_string(),
            range: Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 2 },
            },
            scope: "keyword.control".to_string(),
            modifiers: vec![],
        };

        let result = engine.resolve_token(&token).unwrap();
        assert_eq!(result.font.family, "FontB");
    }

    #[test]
    fn test_resolve_all() {
        let mut engine = ScopeMatchEngine::new();
        engine.add_rule(FontRule {
            scope: "keyword".to_string(),
            font: FontSpec::default_font("FontA"),
        });

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
                text: "x".to_string(),
                range: Range {
                    start: Position { line: 0, column: 3 },
                    end: Position { line: 0, column: 4 },
                },
                scope: "variable".to_string(),
                modifiers: vec![],
            },
        ];

        let results = engine.resolve_all(&tokens);
        let matched: Vec<_> = results.iter().flatten().collect();
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].scope, "keyword");
    }

    #[test]
    fn test_clear() {
        let mut engine = ScopeMatchEngine::new();
        engine.add_rule(FontRule {
            scope: "keyword".to_string(),
            font: FontSpec::default_font("FontA"),
        });
        engine.clear();
        assert!(engine.rules().is_empty());
    }
}
