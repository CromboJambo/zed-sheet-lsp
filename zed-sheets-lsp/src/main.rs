use tower_lsp::{LspService, Server};
use tokio::io::{stdin, stdout};

mod document;
use document::Document;

#[tokio::main]
async fn main() {
    let stdin = stdin();
    let stdout = stdout();

    let (service, socket) = LspService::new(|client| {
        Document::new(client)
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
