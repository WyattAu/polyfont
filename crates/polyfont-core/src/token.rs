use serde::{Deserialize, Serialize};

/// Zero-based line and column position in a text document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

/// Half-open range from [`start`](Range::start) (inclusive) to
/// [`end`](Range::end) (exclusive) in a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// A syntax token with its text, range, scope, and modifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub text: String,
    pub range: Range,
    pub scope: String,
    pub modifiers: Vec<String>,
}

/// Collection of tokens returned by a parser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCollection {
    pub uri: String,
    pub language: String,
    pub tokens: Vec<TokenInfo>,
}
