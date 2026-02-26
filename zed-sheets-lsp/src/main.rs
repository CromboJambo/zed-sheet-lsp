use tower_lsp::{LspService, Server};
use zed_sheets_lsp::document::Document;
use zed_sheets_lsp::sidecar::Sidecar;
use zed_sheets_lsp::dag::DependencyGraph;
use zed_sheets_lsp::diagnostics::Diagnostics;
use zed_sheets_lsp::completions::Completions;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| {
        Document::new(client)
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
