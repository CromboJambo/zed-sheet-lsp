use tokio::io::{stdin, stdout};
use tower_lsp::{LspService, Server};
use zed_sheets_lsp::document::Document;

#[tokio::main]
async fn main() {
    let stdin = stdin();
    let stdout = stdout();

    // Create the LSP service with our document handler
    let (service, socket) = LspService::new(|client| Document::new(client));

    // Start the server
    Server::new(stdin, stdout, socket).serve(service).await;
}
