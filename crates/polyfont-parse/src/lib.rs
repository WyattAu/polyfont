use std::collections::HashMap;

use polyfont_core::{Position, Range, TokenInfo};
use thiserror::Error;
use tracing::warn;

/// Character offset encoding used by LSP clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetEncoding {
    /// Byte offsets (0-based).
    Utf8,
    /// UTF-16 code unit offsets (0-based), used by VSCode.
    Utf16,
}

/// Errors that can occur during token parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("tree-sitter parsing failed for language '{language}': {message}")]
    ParseFailed { language: String, message: String },
}

/// Trait for language-specific tokenization support.
pub trait LanguageSupport: Send + Sync {
    fn language_name(&self) -> &str;
    fn language_id(&self) -> &str;
    fn parse(
        &self,
        text: &str,
        offset_encoding: OffsetEncoding,
    ) -> Result<Vec<TokenInfo>, ParseError>;
}

/// Convert a byte offset in a text document to a line/column position.
pub fn byte_offset_to_position(
    text: &str,
    byte_offset: usize,
    offset_encoding: OffsetEncoding,
) -> Position {
    let bytes = text.as_bytes();
    let offset = byte_offset.min(bytes.len());

    let mut line: u32 = 0;
    let mut line_start: usize = 0;

    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b'\n' {
            line += 1;
            line_start = i + 1;
        }
        if i == offset {
            break;
        }
    }

    let line_text = if offset >= line_start && offset <= bytes.len() {
        &text[line_start..offset]
    } else {
        ""
    };

    let column = match offset_encoding {
        OffsetEncoding::Utf8 => line_text.len() as u32,
        OffsetEncoding::Utf16 => line_text.encode_utf16().count() as u32,
    };

    Position { line, column }
}

#[allow(dead_code)]
fn byte_offset_to_position_safe(
    text: &str,
    byte_offset: usize,
    offset_encoding: OffsetEncoding,
) -> Position {
    if byte_offset <= text.len() {
        byte_offset_to_position(text, byte_offset, offset_encoding)
    } else {
        byte_offset_to_position(text, text.len(), offset_encoding)
    }
}

/// Derive a TextMate scope string from tree-sitter highlight capture names.
pub fn scope_from_highlights(highlight_names: &[&str]) -> String {
    highlight_names.join(".")
}

#[allow(dead_code)]
struct HighlightParser {
    config: tree_sitter_highlight::HighlightConfiguration,
    language_name: String,
    language_id: String,
}

#[allow(dead_code)]
impl HighlightParser {
    fn new(
        language: tree_sitter::Language,
        language_name: &str,
        language_id: &str,
        highlights_query: &str,
        injections_query: &str,
        locals_query: &str,
    ) -> Result<Self, ParseError> {
        let config = tree_sitter_highlight::HighlightConfiguration::new(
            language,
            language_name,
            highlights_query,
            injections_query,
            locals_query,
        )
        .map_err(|e| ParseError::ParseFailed {
            language: language_name.to_owned(),
            message: e.to_string(),
        })?;

        Ok(Self {
            config,
            language_name: language_name.to_owned(),
            language_id: language_id.to_owned(),
        })
    }

    fn parse_impl(
        &self,
        text: &str,
        offset_encoding: OffsetEncoding,
    ) -> Result<Vec<TokenInfo>, ParseError> {
        let mut highlighter = tree_sitter_highlight::Highlighter::new();
        let source = text.as_bytes();

        let events: Vec<tree_sitter_highlight::HighlightEvent> =
            match highlighter.highlight(&self.config, source, None, |_| None) {
                Ok(iter) => iter.filter_map(|e| e.ok()).collect(),
                Err(e) => {
                    warn!(
                        language = %self.language_name,
                        error = %e,
                        "highlighting failed, returning empty tokens"
                    );
                    return Ok(Vec::new());
                }
            };

        let mut tokens = Vec::new();
        let mut scope_stack: Vec<String> = Vec::new();
        let mut byte_start_stack: Vec<usize> = Vec::new();
        let mut current_source_start: usize = 0;
        let mut current_source_end: usize = 0;

        for event in events {
            match event {
                tree_sitter_highlight::HighlightEvent::Source { start, end } => {
                    current_source_start = start;
                    current_source_end = end;
                }
                tree_sitter_highlight::HighlightEvent::HighlightStart(capture) => {
                    let capture_idx = capture.0;
                    let name = self
                        .config
                        .names()
                        .get(capture_idx)
                        .copied()
                        .unwrap_or("unknown");
                    scope_stack.push(name.to_owned());
                    byte_start_stack.push(current_source_start);
                }
                tree_sitter_highlight::HighlightEvent::HighlightEnd => {
                    let scope = scope_stack.pop().unwrap_or_default();
                    let byte_start = byte_start_stack.pop().unwrap_or(current_source_start);
                    let byte_end = current_source_end;

                    if byte_end > byte_start {
                        let safe_start = byte_start.min(text.len());
                        let safe_end = byte_end.min(text.len());
                        if safe_end > safe_start
                            && text.is_char_boundary(safe_start)
                            && text.is_char_boundary(safe_end)
                        {
                            let token_text = text[safe_start..safe_end].to_owned();
                            let start_pos =
                                byte_offset_to_position_safe(text, safe_start, offset_encoding);
                            let end_pos =
                                byte_offset_to_position_safe(text, safe_end, offset_encoding);

                            tokens.push(TokenInfo {
                                text: token_text,
                                range: Range {
                                    start: start_pos,
                                    end: end_pos,
                                },
                                scope,
                                modifiers: Vec::new(),
                            });
                        }
                    }
                }
            }
        }

        Ok(tokens)
    }
}

impl LanguageSupport for HighlightParser {
    fn language_name(&self) -> &str {
        &self.language_name
    }

    fn language_id(&self) -> &str {
        &self.language_id
    }

    fn parse(
        &self,
        text: &str,
        offset_encoding: OffsetEncoding,
    ) -> Result<Vec<TokenInfo>, ParseError> {
        self.parse_impl(text, offset_encoding)
    }
}

macro_rules! register_language {
    ($languages:expr, $id:expr, $name:expr, $feature_gate:expr, $lang_fn:expr, $hq:expr, $iq:expr, $lq:expr) => {
        #[cfg(feature = $feature_gate)]
        {
            match HighlightParser::new($lang_fn, $name, $id, $hq, $iq, $lq) {
                Ok(parser) => {
                    $languages.insert($id.to_owned(), Box::new(parser));
                }
                Err(e) => {
                    warn!(
                        language = $name,
                        error = %e,
                        "failed to create highlighter for language, skipping"
                    );
                }
            }
        }
    };
}

/// Multi-language token parser using tree-sitter grammars when available, with naive fallback.
pub struct TokenParser {
    languages: HashMap<String, Box<dyn LanguageSupport>>,
}

impl TokenParser {
    /// Create a new token parser.
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut languages: HashMap<String, Box<dyn LanguageSupport>> = HashMap::new();

        register_language!(
            languages,
            "rust",
            "Rust",
            "rust",
            tree_sitter_rust::language(),
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            ""
        );

        register_language!(
            languages,
            "typescript",
            "TypeScript",
            "typescript",
            tree_sitter_typescript::language_typescript(),
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
            tree_sitter_typescript::INJECTIONS_QUERY,
            tree_sitter_typescript::LOCALS_QUERY
        );

        register_language!(
            languages,
            "javascript",
            "JavaScript",
            "javascript",
            tree_sitter_typescript::language_typescript(),
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
            tree_sitter_typescript::INJECTIONS_QUERY,
            tree_sitter_typescript::LOCALS_QUERY
        );

        register_language!(
            languages,
            "python",
            "Python",
            "python",
            tree_sitter_python::language(),
            tree_sitter_python::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_python::LOCALS_QUERY
        );

        register_language!(
            languages,
            "go",
            "Go",
            "go",
            tree_sitter_go::language(),
            tree_sitter_go::HIGHLIGHTS_QUERY,
            "",
            ""
        );

        register_language!(
            languages,
            "c",
            "C",
            "c",
            tree_sitter_c::language(),
            tree_sitter_c::HIGHLIGHTS_QUERY,
            "",
            ""
        );

        register_language!(
            languages,
            "cpp",
            "C++",
            "cpp",
            tree_sitter_cpp::language(),
            tree_sitter_cpp::HIGHLIGHTS_QUERY,
            "",
            ""
        );

        register_language!(
            languages,
            "json",
            "JSON",
            "json",
            tree_sitter_json::language(),
            tree_sitter_json::HIGHLIGHTS_QUERY,
            "",
            ""
        );

        register_language!(
            languages,
            "toml",
            "TOML",
            "toml",
            tree_sitter_toml::language(),
            tree_sitter_toml::HIGHLIGHTS_QUERY,
            "",
            ""
        );

        register_language!(
            languages,
            "lua",
            "Lua",
            "lua",
            tree_sitter_lua::language(),
            tree_sitter_lua::HIGHLIGHTS_QUERY,
            "",
            ""
        );

        Self { languages }
    }

    /// Return a list of supported language identifiers.
    pub fn supported_languages(&self) -> Vec<&str> {
        let mut langs: Vec<&str> = self.languages.keys().map(String::as_str).collect();
        langs.sort();
        langs
    }

    /// Parse tokens from source text for the given language.
    pub fn parse_tokens(
        &self,
        text: &str,
        language_id: &str,
        offset_encoding: OffsetEncoding,
    ) -> Result<Vec<TokenInfo>, ParseError> {
        let support = self
            .languages
            .get(language_id)
            .ok_or_else(|| ParseError::UnsupportedLanguage(language_id.to_owned()))?;
        support.parse(text, offset_encoding)
    }
}

impl Default for TokenParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_offset_to_position_single_line() {
        let text = "hello world";
        let pos = byte_offset_to_position(text, 5, OffsetEncoding::Utf8);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 5);
    }

    #[test]
    fn test_byte_offset_to_position_multiline() {
        let text = "line1\nline2\nline3";
        let pos = byte_offset_to_position(text, 6, OffsetEncoding::Utf8);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);

        let pos = byte_offset_to_position(text, 12, OffsetEncoding::Utf8);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_byte_offset_to_position_utf16() {
        let text = "hello\nworld";
        let pos = byte_offset_to_position(text, 6, OffsetEncoding::Utf16);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_byte_offset_to_position_end_of_text() {
        let text = "abc";
        let pos = byte_offset_to_position(text, 3, OffsetEncoding::Utf8);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 3);
    }

    #[test]
    fn test_byte_offset_to_position_empty_text() {
        let pos = byte_offset_to_position("", 0, OffsetEncoding::Utf8);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_byte_offset_to_position_safe_clamps() {
        let text = "abc";
        let pos = byte_offset_to_position_safe(text, 100, OffsetEncoding::Utf8);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 3);
    }

    #[test]
    fn test_scope_from_highlights_single() {
        let scope = scope_from_highlights(&["keyword"]);
        assert_eq!(scope, "keyword");
    }

    #[test]
    fn test_scope_from_highlights_multiple() {
        let scope = scope_from_highlights(&["keyword", "control"]);
        assert_eq!(scope, "keyword.control");
    }

    #[test]
    fn test_scope_from_highlights_empty() {
        let scope = scope_from_highlights(&[]);
        assert_eq!(scope, "");
    }

    #[test]
    fn test_scope_from_highlights_three_levels() {
        let scope = scope_from_highlights(&["entity", "name", "function"]);
        assert_eq!(scope, "entity.name.function");
    }

    #[test]
    fn test_token_parser_no_features_by_default() {
        let parser = TokenParser::new();
        assert!(parser.supported_languages().is_empty());
    }

    #[test]
    fn test_token_parser_unsupported_language() {
        let parser = TokenParser::new();
        let result = parser.parse_tokens("fn main() {}", "rust", OffsetEncoding::Utf8);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::UnsupportedLanguage(lang) => assert_eq!(lang, "rust"),
            other => panic!("expected UnsupportedLanguage, got {other}"),
        }
    }

    #[test]
    fn test_token_parser_empty_input() {
        let parser = TokenParser::new();
        let result = parser.parse_tokens("", "rust", OffsetEncoding::Utf8);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_parser_unknown_language_ids() {
        let parser = TokenParser::new();
        for id in &["brainfuck", "cobol", "fortran", "haskell", "zig"] {
            let result = parser.parse_tokens("", id, OffsetEncoding::Utf8);
            assert!(result.is_err());
        }
    }
}
