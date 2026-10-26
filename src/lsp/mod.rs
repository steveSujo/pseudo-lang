// LSP Server implementation for pseudo-lang
// Uses tower-lsp for the Language Server Protocol implementation

pub mod backend;

pub use backend::Backend;
pub use backend::PsudoLanguageServer;

/// Start the LSP server
pub async fn start_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = tower_lsp::LspService::new(Backend::new);
    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}
