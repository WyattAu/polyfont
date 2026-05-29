#![allow(clippy::multiple_crate_versions)]
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

use polyfont_config::{ConfigLoader, PolyfontConfig};
use polyfont_core::{PolyfontEngine, ScopeMatchEngine, TokenInfo};
use polyfont_parse::{OffsetEncoding, TokenParser};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::{
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams,
    ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use tower_lsp::{Client, ClientSocket, LanguageServer, LspService};
use tracing::{info, warn};

/// LSP notification payload sent when font assignments change for a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontAssignmentNotification {
    pub uri: String,
    pub assignments: Vec<FontAssignmentEntry>,
}

/// A single scope-to-font assignment within a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontAssignmentEntry {
    pub scope: String,
    pub range: LspRange,
    pub font: FontInfo,
}

/// LSP-compatible text range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

/// LSP-compatible position (zero-based line and character).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

/// Serializable font specification for LSP communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub family: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallbacks: Vec<String>,
    #[serde(default = "default_weight", skip_serializing_if = "is_default_weight")]
    pub weight: String,
    #[serde(default = "default_style", skip_serializing_if = "is_default_style")]
    pub style: String,
}

fn default_weight() -> String {
    "regular".to_string()
}

fn is_default_weight(s: &str) -> bool {
    s == "regular"
}

fn default_style() -> String {
    "normal".to_string()
}

fn is_default_style(s: &str) -> bool {
    s == "normal"
}

impl From<&polyfont_core::FontSpec> for FontInfo {
    fn from(spec: &polyfont_core::FontSpec) -> Self {
        Self {
            family: spec.family.clone(),
            fallbacks: spec.fallbacks.clone(),
            weight: spec.weight.to_string(),
            style: spec.style.to_string(),
        }
    }
}

#[derive(Debug)]
struct DocumentState {
    version: i32,
    text: String,
    /// Cached tokenization result to enable incremental re-parsing.
    cached_tokens: Vec<TokenInfo>,
}

struct PolyfontFontAssignments;

impl tower_lsp::lsp_types::notification::Notification for PolyfontFontAssignments {
    type Params = FontAssignmentNotification;
    const METHOD: &'static str = "polyfont/fontAssignments";
}

/// LSP server providing per-token font assignment for polyfont-aware editors.
pub struct PolyfontLanguageServer {
    client: Client,
    state: Arc<RwLock<ServerState>>,
}

struct ServerState {
    engine: Option<ScopeMatchEngine>,
    config: Option<PolyfontConfig>,
    workspace_root: Option<PathBuf>,
    documents: HashMap<String, DocumentState>,
}

impl ServerState {
    fn new() -> Self {
        Self {
            engine: None,
            config: None,
            workspace_root: None,
            documents: HashMap::new(),
        }
    }
}

fn build_assignment_entries(
    engine: &ScopeMatchEngine,
    tokens: &[TokenInfo],
) -> Vec<FontAssignmentEntry> {
    let assignments = engine.resolve_all(tokens);
    tokens
        .iter()
        .zip(assignments)
        .filter_map(|(token, assignment)| {
            let assignment = assignment?;
            let font_info = FontInfo::from(&assignment.font);
            let range = LspRange {
                start: LspPosition {
                    line: token.range.start.line,
                    character: token.range.start.column,
                },
                end: LspPosition {
                    line: token.range.end.line,
                    character: token.range.end.column,
                },
            };
            Some(FontAssignmentEntry {
                scope: assignment.scope,
                range,
                font: font_info,
            })
        })
        .collect()
}

impl PolyfontLanguageServer {
    /// Create a new language server instance with the given LSP client.
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: Arc::new(RwLock::new(ServerState::new())),
        }
    }

    /// Build the LSP service with custom polyfont request handlers.
    pub fn build_service() -> (LspService<Self>, ClientSocket) {
        LspService::build(Self::new)
            .custom_method(
                "polyfont/requestFontAssignments",
                Self::serve_request_font_assignments,
            )
            .custom_method("polyfont/suggestFonts", Self::serve_suggest_fonts)
            .finish()
    }

    async fn load_config(&self, root: &std::path::Path) {
        info!(
            "loading polyfont config from workspace root: {}",
            root.display()
        );
        match ConfigLoader::load_from_dir(root) {
            Ok(config) => {
                info!("loaded config with {} rules", config.rules.len());
                let rules = config.to_rules();
                let engine = ScopeMatchEngine::from_rules(rules);
                let mut state = self.state.write().await;
                state.config = Some(config);
                state.engine = Some(engine);
                state.workspace_root = Some(root.to_path_buf());
            }
            Err(e) => {
                warn!("failed to load config: {e}");
            }
        }
    }

    async fn publish_assignments(&self, uri: &str) {
        let entries = {
            let state = self.state.read().await;
            let Some(engine) = &state.engine else {
                return;
            };
            let Some(doc) = state.documents.get(uri) else {
                return;
            };
            // Use cached tokens if available, otherwise re-tokenize.
            let tokens = if doc.cached_tokens.is_empty() {
                tokenize_document(&doc.text, uri)
            } else {
                doc.cached_tokens.clone()
            };
            let entries = build_assignment_entries(engine, &tokens);
            drop(state);
            entries
        };

        if entries.is_empty() {
            return;
        }

        let notification = FontAssignmentNotification {
            uri: uri.to_string(),
            assignments: entries,
        };

        self.client
            .send_notification::<PolyfontFontAssignments>(notification)
            .await;
    }

    async fn serve_request_font_assignments(
        &self,
        params: FontAssignmentsRequestParams,
    ) -> LspResult<Option<FontAssignmentNotification>> {
        let entries = {
            let state = self.state.read().await;
            let Some(engine) = &state.engine else {
                return Ok(None);
            };
            let Some(doc) = state.documents.get(&params.uri) else {
                return Ok(None);
            };
            let tokens = tokenize_document(&doc.text, &params.uri);
            let entries = build_assignment_entries(engine, &tokens);
            drop(state);
            entries
        };

        if entries.is_empty() {
            return Ok(None);
        }

        Ok(Some(FontAssignmentNotification {
            uri: params.uri,
            assignments: entries,
        }))
    }

    async fn serve_suggest_fonts(
        &self,
        params: SuggestFontsParams,
    ) -> LspResult<Option<FontSuggestionsResponse>> {
        let _language = params.language;
        let suggestions: Vec<FontSuggestion> = FONT_PAIRINGS
            .iter()
            .map(|(scope, family, reason, category)| FontSuggestion {
                scope: (*scope).to_string(),
                recommended_family: (*family).to_string(),
                reason: (*reason).to_string(),
                category: (*category).to_string(),
            })
            .collect();

        Ok(Some(FontSuggestionsResponse { suggestions }))
    }
}

#[derive(Debug, Deserialize)]
struct FontAssignmentsRequestParams {
    uri: String,
}

#[derive(Debug, Deserialize)]
struct SuggestFontsParams {
    /// Optional language ID to tailor suggestions.
    language: Option<String>,
}

#[derive(Debug, Serialize)]
struct FontSuggestion {
    scope: String,
    recommended_family: String,
    reason: String,
    category: String,
}

#[derive(Debug, Serialize)]
struct FontSuggestionsResponse {
    suggestions: Vec<FontSuggestion>,
}

/// Curated font pairing database indexed by scope category.
static FONT_PAIRINGS: &[(&str, &str, &str, &str)] = &[
    // (scope_prefix, family, reason, category)
    (
        "keyword",
        "Maple Mono",
        "Clear geometric mono with heavy weight for keywords",
        "geometric",
    ),
    (
        "keyword.control",
        "Fira Code",
        "Ligature support for control flow operators",
        "ligature",
    ),
    (
        "comment",
        "IBM Plex Mono",
        "Humanist design improves readability for prose comments",
        "humanist",
    ),
    (
        "comment.doc",
        "Source Serif Pro",
        "Serif face signals documentation distinct from code",
        "serif",
    ),
    (
        "string",
        "Source Code Pro",
        "Light weight creates visual contrast for string literals",
        "light",
    ),
    (
        "string.regexp",
        "JetBrains Mono",
        "Dense information density suits regex patterns",
        "dense",
    ),
    (
        "entity.name.function",
        "Monaspace Argon",
        "Distinctive x-height for function identification",
        "variable",
    ),
    (
        "entity.name.type",
        "Monaspace Neon",
        "Wide stance for type names at a glance",
        "variable",
    ),
    (
        "variable",
        "JetBrains Mono",
        "Balanced weight for the most common token type",
        "balanced",
    ),
    (
        "variable.parameter",
        "MonoLisa",
        "Italic-friendly for parameter distinction",
        "humanist",
    ),
    (
        "constant",
        "Monaspace Radon",
        "Heavy weight emphasizes constant values",
        "variable",
    ),
    (
        "constant.numeric",
        "Input Mono",
        "Tabular figures for numeric alignment",
        "tabular",
    ),
    (
        "support.function",
        "Monaspace Krypton",
        "Medium weight for built-in function calls",
        "variable",
    ),
    (
        "punctuation",
        "Fira Code",
        "Ligature support for bracket pairs and arrows",
        "ligature",
    ),
    (
        "operator",
        "Operator Mono",
        "Italic-style operators for visual separation",
        "stylish",
    ),
    (
        "storage.type",
        "Maple Mono",
        "Bold weight for type annotations",
        "geometric",
    ),
];

static TOKEN_PARSER: LazyLock<TokenParser> = LazyLock::new(TokenParser::new);

fn language_id_from_uri(uri: &str) -> &str {
    let path = uri.split('/').next_back().unwrap_or(uri);
    match path.split('.').next_back().unwrap_or("") {
        "rs" => "rust",
        "ts" => "typescript",
        "tsx" => "typescript",
        "js" => "javascript",
        "jsx" => "javascript",
        "py" => "python",
        "go" => "go",
        "c" => "c",
        "cpp" | "cc" | "cxx" | "h" | "hpp" => "cpp",
        "json" => "json",
        "toml" => "toml",
        "lua" => "lua",
        _ => "unknown",
    }
}

fn tokenize_document(text: &str, uri: &str) -> Vec<polyfont_core::TokenInfo> {
    let lang = language_id_from_uri(uri);

    match TOKEN_PARSER.parse_tokens(text, lang, OffsetEncoding::Utf16) {
        Ok(tokens) if !tokens.is_empty() => {
            info!(
                language = lang,
                method = "tree-sitter",
                "tokenized document"
            );
            tokens
        }
        Ok(_) => {
            info!(
                language = lang,
                method = "naive",
                reason = "tree-sitter returned no tokens",
                "tokenized document"
            );
            tokenize_document_naive(text)
        }
        Err(e) => {
            info!(
                language = lang,
                method = "naive",
                reason = %e,
                "tokenized document"
            );
            tokenize_document_naive(text)
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn tokenize_document_naive(text: &str) -> Vec<polyfont_core::TokenInfo> {
    let mut tokens = Vec::new();
    for (line_idx, line) in text.lines().enumerate() {
        let leading = line.len() - line.trim_start().len();
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let scope = classify_line(trimmed);
        let end_char = (leading + trimmed.len()) as u32;

        tokens.push(polyfont_core::TokenInfo {
            text: trimmed.to_string(),
            range: polyfont_core::Range {
                start: polyfont_core::Position {
                    line: line_idx as u32,
                    column: leading as u32,
                },
                end: polyfont_core::Position {
                    line: line_idx as u32,
                    column: end_char,
                },
            },
            scope,
            modifiers: Vec::new(),
        });
    }
    tokens
}

fn classify_line(line: &str) -> String {
    let trimmed = line.trim();

    if trimmed.starts_with("///") || trimmed.starts_with("//") || trimmed.starts_with('#') {
        return "comment".to_string();
    }
    if trimmed.starts_with('"') || trimmed.starts_with('\'') || trimmed.starts_with('`') {
        return "string".to_string();
    }
    if trimmed.starts_with("fn ")
        || trimmed.starts_with("function ")
        || trimmed.starts_with("def ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("async fn ")
    {
        return "entity.name.function".to_string();
    }
    if trimmed.starts_with("let ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("var ")
        || trimmed.starts_with("mut ")
        || trimmed.starts_with("let mut ")
    {
        return "variable".to_string();
    }
    if trimmed.starts_with("struct ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("interface ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("impl ")
        || trimmed.starts_with("trait ")
    {
        return "entity.name.type".to_string();
    }
    if trimmed.starts_with("use ")
        || trimmed.starts_with("import ")
        || trimmed.starts_with("from ")
        || trimmed.starts_with("mod ")
    {
        return "keyword".to_string();
    }
    if trimmed.starts_with("if ")
        || trimmed.starts_with("else")
        || trimmed.starts_with("for ")
        || trimmed.starts_with("while ")
        || trimmed.starts_with("loop ")
        || trimmed.starts_with("match ")
        || trimmed.starts_with("switch ")
        || trimmed.starts_with("return")
        || trimmed.starts_with("break")
        || trimmed.starts_with("continue")
    {
        return "keyword.control".to_string();
    }

    "source".to_string()
}

#[tower_lsp::async_trait]
impl LanguageServer for PolyfontLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        info!("initializing polyfont LSP server");

        let workspace_root = params.root_uri.and_then(|uri| uri.to_file_path().ok());

        if let Some(ref root) = workspace_root {
            let mut state = self.state.write().await;
            state.workspace_root = Some(root.clone());
        }

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            ..Default::default()
        };

        Ok(InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: "polyfont-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        info!("polyfont LSP server initialized");

        let workspace_root = {
            let state = self.state.read().await;
            state.workspace_root.clone()
        };

        if let Some(root) = workspace_root {
            self.load_config(&root).await;

            let uris: Vec<String> = {
                let state = self.state.read().await;
                state.documents.keys().cloned().collect()
            };
            for uri in uris {
                self.publish_assignments(&uri).await;
            }
        }
    }

    async fn shutdown(&self) -> LspResult<()> {
        info!("shutting down polyfont LSP server");
        let mut state = self.state.write().await;
        state.engine = None;
        state.config = None;
        state.documents.clear();
        drop(state);
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        info!("document opened: {uri}");

        let text = params.text_document.text.clone();
        let tokens = tokenize_document(&text, &uri);

        {
            let mut state = self.state.write().await;
            state.documents.insert(
                uri.clone(),
                DocumentState {
                    version: params.text_document.version,
                    text,
                    cached_tokens: tokens,
                },
            );
        }

        self.publish_assignments(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();

        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text.clone();
            let tokens = tokenize_document(&text, &uri);
            let mut state = self.state.write().await;
            if let Some(doc) = state.documents.get_mut(&uri) {
                doc.text = text;
                doc.version = params.text_document.version;
                doc.cached_tokens = tokens;
            }
        }

        self.publish_assignments(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        info!("document closed: {uri}");
        let mut state = self.state.write().await;
        state.documents.remove(&uri);
    }

    async fn did_change_configuration(&self, _params: DidChangeConfigurationParams) {
        info!("configuration changed, reloading");

        let workspace_root = {
            let state = self.state.read().await;
            state.workspace_root.clone()
        };

        if let Some(root) = workspace_root {
            self.load_config(&root).await;

            let uris: Vec<String> = {
                let state = self.state.read().await;
                state.documents.keys().cloned().collect()
            };
            for uri in uris {
                self.publish_assignments(&uri).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polyfont_core::{FontRule, FontSpec, Position, Range, ScopeMatchEngine, TokenInfo};
    use polyfont_core::{FontStyle, FontWeight};

    #[test]
    fn test_language_id_rust() {
        assert_eq!(
            language_id_from_uri("file:///home/user/src/main.rs"),
            "rust"
        );
        assert_eq!(language_id_from_uri("main.rs"), "rust");
    }

    #[test]
    fn test_language_id_typescript() {
        assert_eq!(language_id_from_uri("app.tsx"), "typescript");
        assert_eq!(language_id_from_uri("app.ts"), "typescript");
    }

    #[test]
    fn test_language_id_javascript() {
        assert_eq!(language_id_from_uri("app.jsx"), "javascript");
        assert_eq!(language_id_from_uri("app.js"), "javascript");
    }

    #[test]
    fn test_language_id_python() {
        assert_eq!(language_id_from_uri("script.py"), "python");
    }

    #[test]
    fn test_language_id_go() {
        assert_eq!(language_id_from_uri("main.go"), "go");
    }

    #[test]
    fn test_language_id_c_cpp() {
        assert_eq!(language_id_from_uri("main.c"), "c");
        assert_eq!(language_id_from_uri("main.cpp"), "cpp");
        assert_eq!(language_id_from_uri("header.hpp"), "cpp");
        assert_eq!(language_id_from_uri("header.h"), "cpp");
    }

    #[test]
    fn test_language_id_config() {
        assert_eq!(language_id_from_uri("Cargo.toml"), "toml");
        assert_eq!(language_id_from_uri("data.json"), "json");
    }

    #[test]
    fn test_language_id_lua() {
        assert_eq!(language_id_from_uri("init.lua"), "lua");
    }

    #[test]
    fn test_language_id_unknown() {
        assert_eq!(language_id_from_uri("readme"), "unknown");
        assert_eq!(language_id_from_uri("file.xyz"), "unknown");
    }

    #[test]
    fn test_classify_comment() {
        assert_eq!(classify_line("// hello"), "comment");
        assert_eq!(classify_line("/// doc comment"), "comment");
        assert_eq!(classify_line("# comment"), "comment");
    }

    #[test]
    fn test_classify_string() {
        assert_eq!(classify_line("\"hello\""), "string");
        assert_eq!(classify_line("'c'"), "string");
        assert_eq!(classify_line("`template`"), "string");
    }

    #[test]
    fn test_classify_function() {
        assert_eq!(classify_line("fn main()"), "entity.name.function");
        assert_eq!(classify_line("function foo()"), "entity.name.function");
        assert_eq!(classify_line("def bar()"), "entity.name.function");
        assert_eq!(classify_line("pub fn baz()"), "entity.name.function");
        assert_eq!(classify_line("async fn qux()"), "entity.name.function");
    }

    #[test]
    fn test_classify_variable() {
        assert_eq!(classify_line("let x = 1"), "variable");
        assert_eq!(classify_line("const Y = 2"), "variable");
        assert_eq!(classify_line("var z = 3"), "variable");
        assert_eq!(classify_line("let mut a = 4"), "variable");
    }

    #[test]
    fn test_classify_type() {
        assert_eq!(classify_line("struct Foo"), "entity.name.type");
        assert_eq!(classify_line("enum Bar"), "entity.name.type");
        assert_eq!(classify_line("class Baz"), "entity.name.type");
        assert_eq!(classify_line("interface Qux"), "entity.name.type");
        assert_eq!(classify_line("type Alias = i32"), "entity.name.type");
        assert_eq!(classify_line("impl Display"), "entity.name.type");
        assert_eq!(classify_line("trait Clone"), "entity.name.type");
    }

    #[test]
    fn test_classify_keyword() {
        assert_eq!(classify_line("use std::io"), "keyword");
        assert_eq!(classify_line("import foo"), "keyword");
        assert_eq!(classify_line("from bar import baz"), "keyword");
        assert_eq!(classify_line("mod tests"), "keyword");
    }

    #[test]
    fn test_classify_control_flow() {
        assert_eq!(classify_line("if x > 0"), "keyword.control");
        assert_eq!(classify_line("else"), "keyword.control");
        assert_eq!(classify_line("for i in 0..10"), "keyword.control");
        assert_eq!(classify_line("while true"), "keyword.control");
        assert_eq!(classify_line("return"), "keyword.control");
        assert_eq!(classify_line("break"), "keyword.control");
        assert_eq!(classify_line("continue"), "keyword.control");
    }

    #[test]
    fn test_classify_source_fallback() {
        assert_eq!(classify_line("x + y"), "source");
        assert_eq!(classify_line("println!(\"hello\")"), "source");
    }

    #[test]
    fn test_tokenize_naive_empty() {
        let tokens = tokenize_document_naive("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_naive_blank_lines() {
        let tokens = tokenize_document_naive("\n\n  \n");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_naive_single_line() {
        let tokens = tokenize_document_naive("fn main()");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].scope, "entity.name.function");
        assert_eq!(tokens[0].range.start.line, 0);
        assert_eq!(tokens[0].range.start.column, 0);
    }

    #[test]
    fn test_tokenize_naive_multiline() {
        let code = "fn foo()\nlet x = 1\n// comment\n";
        let tokens = tokenize_document_naive(code);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].scope, "entity.name.function");
        assert_eq!(tokens[1].scope, "variable");
        assert_eq!(tokens[2].scope, "comment");
        assert_eq!(tokens[0].range.start.line, 0);
        assert_eq!(tokens[1].range.start.line, 1);
        assert_eq!(tokens[2].range.start.line, 2);
    }

    #[test]
    fn test_tokenize_naive_indentation_preserved() {
        let tokens = tokenize_document_naive("    let x = 1");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].range.start.column, 4);
        assert_eq!(tokens[0].text, "let x = 1");
    }

    #[test]
    fn test_font_info_from_spec() {
        let spec = FontSpec {
            family: "Fira Code".to_string(),
            fallbacks: vec!["monospace".to_string()],
            weight: FontWeight::Bold,
            style: FontStyle::Italic,
            size: None,
            axes: vec![],
        };
        let info = FontInfo::from(&spec);
        assert_eq!(info.family, "Fira Code");
        assert_eq!(info.fallbacks, vec!["monospace"]);
        assert_eq!(info.weight, "bold");
        assert_eq!(info.style, "italic");
    }

    #[test]
    fn test_server_state_new() {
        let state = ServerState::new();
        assert!(state.engine.is_none());
        assert!(state.config.is_none());
        assert!(state.workspace_root.is_none());
        assert!(state.documents.is_empty());
    }

    #[test]
    fn test_build_assignment_entries_matching() {
        let engine = ScopeMatchEngine::from_rules(vec![FontRule {
            scope: "keyword".to_string(),
            font: FontSpec::default_font("Maple Mono"),
        }]);
        let tokens = vec![TokenInfo {
            text: "fn".to_string(),
            range: Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 2 },
            },
            scope: "keyword".to_string(),
            modifiers: vec![],
        }];
        let entries = build_assignment_entries(&engine, &tokens);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].font.family, "Maple Mono");
        assert_eq!(entries[0].scope, "keyword");
        assert_eq!(entries[0].range.start.line, 0);
        assert_eq!(entries[0].range.start.character, 0);
    }

    #[test]
    fn test_build_assignment_entries_no_match() {
        let engine = ScopeMatchEngine::from_rules(vec![]);
        let tokens = vec![TokenInfo {
            text: "x".to_string(),
            range: Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 1 },
            },
            scope: "variable".to_string(),
            modifiers: vec![],
        }];
        let entries = build_assignment_entries(&engine, &tokens);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_font_pairings_not_empty() {
        assert!(!FONT_PAIRINGS.is_empty());
        for (scope, family, reason, category) in FONT_PAIRINGS {
            assert!(!scope.is_empty(), "scope should not be empty");
            assert!(!family.is_empty(), "family should not be empty");
            assert!(!reason.is_empty(), "reason should not be empty");
            assert!(!category.is_empty(), "category should not be empty");
        }
    }
}
