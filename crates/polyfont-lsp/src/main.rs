#![allow(clippy::multiple_crate_versions)]
use polyfont_lsp::PolyfontLanguageServer;
use tower_lsp::Server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let (service, socket) = PolyfontLanguageServer::build_service();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    Server::new(stdin, stdout, socket).serve(service).await;
}
