use std::collections::{BTreeMap, HashMap};

use polyfont_core::{FontAssignment, FontRule};

#[cfg(test)]
use polyfont_core::{FontSpec, FontStyle, FontWeight};

/// Common TextMate scope names used in syntax highlighting.
pub mod constants {
    pub const SCOPE_KEYWORD: &str = "keyword";
    pub const SCOPE_COMMENT: &str = "comment";
    pub const SCOPE_STRING: &str = "string";
    pub const SCOPE_FUNCTION: &str = "entity.name.function";
    pub const SCOPE_VARIABLE: &str = "variable";
    pub const SCOPE_CONSTANT: &str = "constant";
    pub const SCOPE_TYPE: &str = "entity.name.type";
    pub const SCOPE_NUMBER: &str = "constant.numeric";
    pub const SCOPE_OPERATOR: &str = "keyword.operator";
    pub const SCOPE_PUNCTUATION: &str = "punctuation";
    pub const SCOPE_TAG: &str = "entity.name.tag";
    pub const SCOPE_ATTRIBUTE: &str = "entity.other.attribute-name";
}

pub use constants::*;

/// A parsed TextMate scope pattern with support for wildcards and negation.
#[derive(Debug, Clone)]
pub struct ScopePattern {
    segments: Vec<PatternSegment>,
    negated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PatternSegment {
    Literal(String),
    Wildcard,
}

impl ScopePattern {
    /// Parse a dot-separated scope pattern string. Supports `*` wildcards and `-` prefix for negation.
    ///
    /// # Errors
    ///
    /// Returns [`ScopeError::EmptyPattern`] if the pattern is empty or whitespace-only.
    pub fn parse(pattern: &str) -> Result<Self, ScopeError> {
        let trimmed = pattern.trim();
        if trimmed.is_empty() {
            return Err(ScopeError::EmptyPattern);
        }

        let (negated, inner) = trimmed
            .strip_prefix('-')
            .map_or((false, trimmed), |rest| (true, rest.trim()));

        if inner.is_empty() {
            return Err(ScopeError::EmptyPattern);
        }

        let segments = inner
            .split('.')
            .map(|s| {
                if s == "*" {
                    PatternSegment::Wildcard
                } else {
                    PatternSegment::Literal(s.to_owned())
                }
            })
            .collect();

        Ok(Self { segments, negated })
    }

    /// Check if this pattern matches the given scope.
    #[must_use]
    pub fn matches_scope(&self, scope: &str) -> bool {
        self.matches_raw(scope)
    }

    /// Raw match without negation check. Returns true if the pattern segments match the scope prefix.
    #[must_use]
    pub fn matches_raw(&self, scope: &str) -> bool {
        let scope_parts: Vec<&str> = scope.split('.').collect();
        if scope_parts.len() < self.segments.len() {
            return false;
        }

        for (i, seg) in self.segments.iter().enumerate() {
            match seg {
                PatternSegment::Wildcard => {}
                PatternSegment::Literal(lit) => {
                    if scope_parts[i] != lit {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Compute the specificity (number of literal, non-wildcard segments). Higher specificity means a more precise match.
    #[must_use]
    pub fn specificity(&self) -> usize {
        self.segments
            .iter()
            .filter(|s| **s != PatternSegment::Wildcard)
            .count()
    }
}

#[derive(Debug, Clone)]
/// A comma-separated collection of scope patterns, supporting both positive and negative (exclusion) patterns.
pub struct ScopeSelector {
    patterns: Vec<ScopePattern>,
}

impl ScopeSelector {
    /// Parse a comma-separated scope selector string. Supports `-` prefixed patterns for exclusion.
    ///
    /// # Errors
    ///
    /// Returns [`ScopeError::EmptyPattern`] if no valid patterns are found.
    pub fn parse(selector: &str) -> Result<Self, ScopeError> {
        let patterns = selector
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(ScopePattern::parse(trimmed))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        if patterns.is_empty() {
            return Err(ScopeError::EmptyPattern);
        }

        Ok(Self { patterns })
    }

    /// Check if any positive pattern matches the scope, excluding any matched by negative patterns.
    #[must_use]
    pub fn matches(&self, scope: &str) -> bool {
        let positive_matches: Vec<&ScopePattern> =
            self.patterns.iter().filter(|p| !p.negated).collect();

        let negative_patterns: Vec<&ScopePattern> =
            self.patterns.iter().filter(|p| p.negated).collect();

        for neg in &negative_patterns {
            if neg.matches_raw(scope) {
                return false;
            }
        }

        if positive_matches.is_empty() && !negative_patterns.is_empty() {
            return true;
        }

        positive_matches.iter().any(|p| p.matches_scope(scope))
    }

    /// Return the maximum specificity among positive patterns.
    #[must_use]
    pub fn specificity(&self) -> usize {
        self.patterns
            .iter()
            .filter(|p| !p.negated)
            .map(ScopePattern::specificity)
            .max()
            .unwrap_or(0)
    }
}

/// Convenience methods for one-shot scope matching without creating persistent objects.
pub struct ScopeMatcher;

impl ScopeMatcher {
    /// Check if a scope matches a selector string. Convenience wrapper around [`ScopeSelector`].
    ///
    /// # Errors
    ///
    /// Returns [`ScopeError`] if the selector string fails to parse.
    pub fn matches(scope: &str, selector: &str) -> Result<bool, ScopeError> {
        let sel = ScopeSelector::parse(selector)?;
        Ok(sel.matches(scope))
    }

    /// Check if a scope matches any of the given selector strings.
    ///
    /// # Errors
    ///
    /// Returns [`ScopeError`] if any selector string fails to parse.
    pub fn matches_any(scope: &str, selectors: &[&str]) -> Result<bool, ScopeError> {
        for selector in selectors {
            let sel = ScopeSelector::parse(selector)?;
            if sel.matches(scope) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// Result of resolving a scope to its best-matching font rule.
#[derive(Debug, Clone)]
pub struct ResolvedScope {
    /// The font assignment for this scope.
    pub assignment: FontAssignment,
    /// Index of the matching rule in the original rule list.
    pub rule_index: usize,
}

/// Resolves TextMate scopes to font rules using specificity-based matching.
pub struct ScopeResolver {
    rules: Vec<(FontRule, usize)>,
}

impl ScopeResolver {
    /// Create an empty resolver with no rules.
    #[must_use]
    pub const fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Create a resolver pre-loaded with font rules.
    #[must_use]
    pub fn from_rules(rules: Vec<FontRule>) -> Self {
        let indexed: Vec<(FontRule, usize)> =
            rules.into_iter().enumerate().map(|(i, r)| (r, i)).collect();
        Self { rules: indexed }
    }

    /// Add a font rule to the resolver.
    pub fn add_rule(&mut self, rule: FontRule) {
        let index = self.rules.len();
        self.rules.push((rule, index));
    }

    /// Resolve a scope to its best-matching font rule. Uses highest specificity, with insertion order as tiebreaker.
    ///
    /// # Panics
    ///
    /// Cannot panic in practice; the `expect` is guarded by the `match` above it.
    #[must_use]
    pub fn resolve(&self, scope: &str) -> Option<ResolvedScope> {
        let mut best: Option<(&FontRule, usize, usize)> = None;

        for (rule, rule_index) in &self.rules {
            if let Ok(selector) = ScopeSelector::parse(&rule.scope)
                && selector.matches(scope)
            {
                let specificity = selector.specificity();
                let should_replace = match &best {
                    None => true,
                    Some((_, _, best_spec)) => {
                        specificity > *best_spec
                            || (specificity == *best_spec
                                && *rule_index < best.expect("checked above").1)
                    }
                };
                if should_replace {
                    best = Some((rule, *rule_index, specificity));
                }
            }
        }

        best.map(|(rule, rule_index, specificity)| ResolvedScope {
            assignment: FontAssignment {
                scope: scope.to_owned(),
                font: rule.font.clone(),
                specificity,
                is_active: true,
            },
            rule_index,
        })
    }

    /// Resolve multiple scopes in batch.
    pub fn resolve_all<'a, I>(&self, scopes: I) -> Vec<Option<ResolvedScope>>
    where
        I: IntoIterator<Item = &'a str>,
    {
        scopes.into_iter().map(|s| self.resolve(s)).collect()
    }

    /// Remove all rules.
    pub fn clear(&mut self) {
        self.rules.clear();
    }

    /// Return the number of registered rules.
    #[must_use]
    pub const fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for ScopeResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// A node in the scope prefix tree. Internal representation for [`ScopeTree`].
#[derive(Debug, Clone, Default)]
pub struct ScopeTreeNode {
    children: BTreeMap<String, Self>,
    is_terminal: bool,
}

/// A prefix tree (trie) for efficient scope storage and prefix queries.
pub struct ScopeTree {
    root: ScopeTreeNode,
}

impl ScopeTree {
    /// Create an empty scope tree.
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: ScopeTreeNode::default(),
        }
    }

    /// Insert a dot-separated scope into the tree.
    pub fn insert(&mut self, scope: &str) {
        let mut node = &mut self.root;
        for segment in scope.split('.') {
            node = node.children.entry(segment.to_owned()).or_default();
        }
        node.is_terminal = true;
    }

    /// Check if an exact scope exists in the tree.
    #[must_use]
    pub fn contains(&self, scope: &str) -> bool {
        let mut node = &self.root;
        for segment in scope.split('.') {
            match node.children.get(segment) {
                Some(child) => node = child,
                None => return false,
            }
        }
        node.is_terminal
    }

    /// Check if any scope in the tree starts with the given prefix.
    #[must_use]
    pub fn has_prefix(&self, prefix: &str) -> bool {
        let mut node = &self.root;
        for segment in prefix.split('.') {
            match node.children.get(segment) {
                Some(child) => node = child,
                None => return false,
            }
        }
        true
    }

    /// Return all scopes that start with the given prefix.
    #[must_use]
    pub fn query_prefix(&self, prefix: &str) -> Vec<String> {
        let mut node = &self.root;
        for segment in prefix.split('.') {
            match node.children.get(segment) {
                Some(child) => node = child,
                None => return Vec::new(),
            }
        }

        let mut results = Vec::new();
        collect_scopes(node, prefix, &mut results);
        results
    }

    /// Return the number of terminal (complete) scopes in the tree.
    #[must_use]
    pub fn len(&self) -> usize {
        count_terminals(&self.root)
    }

    /// Return true if the tree contains no scopes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.root.is_terminal && self.root.children.is_empty()
    }
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_scopes(node: &ScopeTreeNode, prefix: &str, results: &mut Vec<String>) {
    if node.is_terminal {
        results.push(prefix.to_owned());
    }
    for (name, child) in &node.children {
        let child_path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        collect_scopes(child, &child_path, results);
    }
}

fn count_terminals(node: &ScopeTreeNode) -> usize {
    let mut count = usize::from(node.is_terminal);
    for child in node.children.values() {
        count += count_terminals(child);
    }
    count
}

/// Trie-based scope resolver with O(k) lookup where k is the number of segments in the scope.
pub struct TrieScopeResolver {
    root: TrieNode,
    rule_count: usize,
}

#[derive(Default)]
struct TrieNode {
    children: HashMap<String, TrieNode>,
    rule: Option<(FontRule, usize)>,
}

impl TrieScopeResolver {
    /// Create an empty trie resolver.
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: TrieNode::default(),
            rule_count: 0,
        }
    }

    /// Create a trie resolver pre-loaded with font rules.
    #[must_use]
    pub fn from_rules(rules: Vec<FontRule>) -> Self {
        let mut resolver = Self::new();
        for rule in rules {
            resolver.add_rule(rule);
        }
        resolver
    }

    /// Add a font rule to the trie.
    pub fn add_rule(&mut self, rule: FontRule) {
        let index = self.rule_count;
        let mut node = &mut self.root;
        for segment in rule.scope.split('.') {
            node = node.children.entry(segment.to_owned()).or_default();
        }
        node.rule = Some((rule, index));
        self.rule_count += 1;
    }

    /// Resolve a scope using trie traversal. Falls back to wildcard (`*`) segments when no literal match exists.
    ///
    /// # Panics
    ///
    /// Cannot panic in practice; the `expect` is guarded by the `match` above it.
    #[must_use]
    pub fn resolve(&self, scope: &str) -> Option<ResolvedScope> {
        let mut node = &self.root;
        let mut best: Option<(&FontRule, usize, usize)> = None;

        for segment in scope.split('.') {
            let next = node
                .children
                .get(segment)
                .or_else(|| node.children.get("*"));

            let Some(next) = next else {
                break;
            };

            node = next;
            if let Some((rule, rule_index)) = &node.rule {
                let specificity = rule.specificity();
                let should_replace = match &best {
                    None => true,
                    Some((_, _, best_spec)) => {
                        specificity > *best_spec
                            || (specificity == *best_spec
                                && *rule_index < best.expect("checked above").1)
                    }
                };
                if should_replace {
                    best = Some((rule, *rule_index, specificity));
                }
            }
        }

        best.map(|(rule, rule_index, specificity)| ResolvedScope {
            assignment: FontAssignment {
                scope: scope.to_owned(),
                font: rule.font.clone(),
                specificity,
                is_active: true,
            },
            rule_index,
        })
    }

    /// Resolve multiple scopes in batch.
    pub fn resolve_all(&self, scopes: &[&str]) -> Vec<Option<ResolvedScope>> {
        scopes.iter().map(|s| self.resolve(s)).collect()
    }

    /// Return the number of registered rules.
    #[must_use]
    pub const fn rule_count(&self) -> usize {
        self.rule_count
    }
}

impl Default for TrieScopeResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during scope parsing and matching.
#[derive(Debug, thiserror::Error)]
pub enum ScopeError {
    /// The scope pattern or selector is empty.
    #[error("empty scope pattern")]
    EmptyPattern,
    /// The scope pattern contains invalid characters.
    #[error("invalid scope pattern: {0}")]
    InvalidPattern(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let pattern = ScopePattern::parse("entity.name.function").unwrap();
        assert!(pattern.matches_scope("entity.name.function"));
        assert!(!pattern.matches_scope("entity.name"));
        assert!(pattern.matches_scope("entity.name.function.call"));
    }

    #[test]
    fn test_hierarchical_match() {
        let pattern = ScopePattern::parse("entity.name").unwrap();
        assert!(pattern.matches_scope("entity.name"));
        assert!(pattern.matches_scope("entity.name.function"));
        assert!(pattern.matches_scope("entity.name.function.call"));
        assert!(!pattern.matches_scope("entity.type"));
    }

    #[test]
    fn test_top_level_scope() {
        let pattern = ScopePattern::parse("entity").unwrap();
        assert!(pattern.matches_scope("entity"));
        assert!(pattern.matches_scope("entity.name"));
        assert!(pattern.matches_scope("entity.name.function"));
    }

    #[test]
    fn test_wildcard_match() {
        let pattern = ScopePattern::parse("entity.*").unwrap();
        assert!(pattern.matches_scope("entity.name"));
        assert!(pattern.matches_scope("entity.name.function"));
        assert!(pattern.matches_scope("entity.type"));
        assert!(!pattern.matches_scope("keyword"));
    }

    #[test]
    fn test_negative_match() {
        let selector = ScopeSelector::parse("keyword,-keyword.operator").unwrap();
        assert!(selector.matches("keyword.control"));
        assert!(!selector.matches("keyword.operator"));
    }

    #[test]
    fn test_comma_separated_or() {
        let selector = ScopeSelector::parse("keyword,storage.type").unwrap();
        assert!(selector.matches("keyword"));
        assert!(selector.matches("storage.type"));
        assert!(selector.matches("storage.type.function"));
        assert!(!selector.matches("comment"));
    }

    #[test]
    fn test_scope_matcher_convenience() {
        assert!(ScopeMatcher::matches("entity.name.function", "entity.name").unwrap());
        assert!(ScopeMatcher::matches("comment.line", "comment").unwrap());
        assert!(!ScopeMatcher::matches("string.quoted", "comment").unwrap());
    }

    #[test]
    fn test_scope_resolver_specificity() {
        let rules = vec![
            FontRule {
                scope: "entity".to_owned(),
                font: FontSpec {
                    family: "font-a".to_owned(),
                    fallbacks: vec![],
                    weight: FontWeight::default(),
                    style: FontStyle::default(),
                    size: None,
                    axes: vec![],
                },
            },
            FontRule {
                scope: "entity.name.function".to_owned(),
                font: FontSpec {
                    family: "font-b".to_owned(),
                    fallbacks: vec![],
                    weight: FontWeight::default(),
                    style: FontStyle::default(),
                    size: None,
                    axes: vec![],
                },
            },
        ];

        let resolver = ScopeResolver::from_rules(rules);
        let result = resolver.resolve("entity.name.function").unwrap();
        assert_eq!(result.assignment.font.family, "font-b");
        assert_eq!(result.assignment.specificity, 3);
    }

    #[test]
    fn test_scope_resolver_tiebreak_by_order() {
        let rules = vec![
            FontRule {
                scope: "keyword".to_owned(),
                font: FontSpec {
                    family: "first".to_owned(),
                    fallbacks: vec![],
                    weight: FontWeight::default(),
                    style: FontStyle::default(),
                    size: None,
                    axes: vec![],
                },
            },
            FontRule {
                scope: "keyword".to_owned(),
                font: FontSpec {
                    family: "second".to_owned(),
                    fallbacks: vec![],
                    weight: FontWeight::default(),
                    style: FontStyle::default(),
                    size: None,
                    axes: vec![],
                },
            },
        ];

        let resolver = ScopeResolver::from_rules(rules);
        let result = resolver.resolve("keyword").unwrap();
        assert_eq!(result.assignment.font.family, "first");
    }

    #[test]
    fn test_scope_tree_insert_and_query() {
        let mut tree = ScopeTree::new();
        tree.insert("entity.name.function");
        tree.insert("entity.name.type");
        tree.insert("entity.name");
        tree.insert("keyword.control");

        assert!(tree.contains("entity.name.function"));
        assert!(tree.contains("entity.name.type"));
        assert!(tree.contains("entity.name"));
        assert!(tree.contains("keyword.control"));
        assert!(!tree.contains("comment"));

        assert!(tree.has_prefix("entity"));
        assert!(tree.has_prefix("entity.name"));
        assert!(!tree.has_prefix("string"));
    }

    #[test]
    fn test_scope_tree_prefix_query() {
        let mut tree = ScopeTree::new();
        tree.insert("entity.name.function");
        tree.insert("entity.name.type");
        tree.insert("entity.other");

        let results = tree.query_prefix("entity.name");
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"entity.name.function".to_owned()));
        assert!(results.contains(&"entity.name.type".to_owned()));
    }

    #[test]
    fn test_scope_tree_len() {
        let mut tree = ScopeTree::new();
        assert!(tree.is_empty());

        tree.insert("entity.name");
        tree.insert("keyword");
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_empty_pattern_error() {
        assert!(ScopePattern::parse("").is_err());
        assert!(ScopePattern::parse("  ").is_err());
        assert!(ScopePattern::parse("-").is_err());
        assert!(ScopeSelector::parse("").is_err());
    }

    #[test]
    fn test_matches_any() {
        assert!(ScopeMatcher::matches_any("keyword.control", &["comment", "keyword"],).unwrap());
        assert!(!ScopeMatcher::matches_any("string.quoted", &["comment", "keyword"],).unwrap());
    }

    #[test]
    fn test_trie_insert_and_resolve() {
        let mut resolver = TrieScopeResolver::new();
        resolver.add_rule(FontRule {
            scope: "keyword".to_owned(),
            font: FontSpec::default_font("mono"),
        });
        let result = resolver.resolve("keyword").unwrap();
        assert_eq!(result.assignment.font.family, "mono");
    }

    #[test]
    fn test_trie_specificity() {
        let mut resolver = TrieScopeResolver::new();
        resolver.add_rule(FontRule {
            scope: "entity".to_owned(),
            font: FontSpec::default_font("font-a"),
        });
        resolver.add_rule(FontRule {
            scope: "entity.name.function".to_owned(),
            font: FontSpec::default_font("font-b"),
        });
        let result = resolver.resolve("entity.name.function").unwrap();
        assert_eq!(result.assignment.font.family, "font-b");
    }

    #[test]
    fn test_trie_partial_match() {
        let mut resolver = TrieScopeResolver::new();
        resolver.add_rule(FontRule {
            scope: "entity.name".to_owned(),
            font: FontSpec::default_font("font-a"),
        });
        let result = resolver.resolve("entity.name.function").unwrap();
        assert_eq!(result.assignment.font.family, "font-a");
    }

    #[test]
    fn test_trie_empty() {
        let resolver = TrieScopeResolver::new();
        assert!(resolver.resolve("anything").is_none());
    }

    #[test]
    fn test_trie_wildcard() {
        let mut resolver = TrieScopeResolver::new();
        resolver.add_rule(FontRule {
            scope: "entity.*".to_owned(),
            font: FontSpec::default_font("wildcard-font"),
        });
        let result = resolver.resolve("entity.name").unwrap();
        assert_eq!(result.assignment.font.family, "wildcard-font");
    }

    #[test]
    fn test_trie_from_rules() {
        let rules = vec![
            FontRule {
                scope: "keyword".to_owned(),
                font: FontSpec::default_font("mono"),
            },
            FontRule {
                scope: "string".to_owned(),
                font: FontSpec::default_font("serif"),
            },
        ];
        let resolver = TrieScopeResolver::from_rules(rules);
        let result = resolver.resolve("keyword").unwrap();
        assert_eq!(result.assignment.font.family, "mono");
    }

    #[test]
    fn test_trie_resolve_all() {
        let rules = vec![
            FontRule {
                scope: "keyword".to_owned(),
                font: FontSpec::default_font("mono"),
            },
            FontRule {
                scope: "string".to_owned(),
                font: FontSpec::default_font("serif"),
            },
        ];
        let resolver = TrieScopeResolver::from_rules(rules);
        let results = resolver.resolve_all(&["keyword", "string", "comment"]);
        assert!(results[0].is_some());
        assert!(results[1].is_some());
        assert!(results[2].is_none());
    }
}
