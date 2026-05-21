#![allow(clippy::multiple_crate_versions)]
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use polyfont_config::{ConfigLoader, PolyfontConfig};
use polyfont_core::{PolyfontEngine, ScopeMatchEngine, TokenInfo};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontAssignmentNotification {
    pub uri: String,
    pub assignments: Vec<FontAssignmentEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontAssignmentEntry {
    pub scope: String,
    pub range: LspRange,
    pub font: FontInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

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
}

struct PolyfontFontAssignments;

impl tower_lsp::lsp_types::notification::Notification for PolyfontFontAssignments {
    type Params = FontAssignmentNotification;
    const METHOD: &'static str = "polyfont/fontAssignments";
}

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
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: Arc::new(RwLock::new(ServerState::new())),
        }
    }

    pub fn build_service() -> (LspService<Self>, ClientSocket) {
        LspService::build(Self::new)
            .custom_method(
                "polyfont/requestFontAssignments",
                Self::serve_request_font_assignments,
            )
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
            let tokens = tokenize_document(&doc.text, uri);
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
}

#[derive(Debug, Deserialize)]
struct FontAssignmentsRequestParams {
    uri: String,
}

#[allow(clippy::cast_possible_truncation)]
fn tokenize_document(text: &str, _uri: &str) -> Vec<polyfont_core::TokenInfo> {
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

        {
            let mut state = self.state.write().await;
            state.documents.insert(
                uri.clone(),
                DocumentState {
                    version: params.text_document.version,
                    text: params.text_document.text,
                },
            );
        }

        self.publish_assignments(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();

        if let Some(change) = params.content_changes.into_iter().last() {
            let mut state = self.state.write().await;
            if let Some(doc) = state.documents.get_mut(&uri) {
                doc.text = change.text;
                doc.version = params.text_document.version;
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
