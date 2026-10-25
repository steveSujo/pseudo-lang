// Backend implementation for the LSP server

use tower_lsp::{Client, LanguageServer};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::tokens::{Token, TokenType};

/// Document state stored by the LSP server
#[derive(Debug, Clone)]
pub struct Document {
    pub uri: String,
    pub version: i32,
    pub text: String,
    pub tokens: Vec<Token>,
    pub parse_errors: Vec<String>,
}

/// The backend state for the LSP server
#[derive(Clone)]
pub struct Backend {
    pub client: Arc<Mutex<Option<Client>>>,
    pub documents: Arc<Mutex<HashMap<String, Document>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client: Arc::new(Mutex::new(Some(client))),
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Lex and parse a document, storing tokens and errors
    async fn analyze_document(&self, uri: &str, text: &str) -> (Vec<Token>, Vec<String>) {
        let mut lexer = Lexer::new(text.to_string());
        let tokens = match lexer.scan_tokens() {
            Ok(tokens) => tokens,
            Err(e) => {
                return (vec![], vec![format!("Lexer error: {}", e)]);
            }
        };

        let mut parser = Parser::new(tokens.clone());
        let parse_result = parser.parse();
        
        let mut parse_errors = Vec::new();
        if let Err(e) = parse_result {
            parse_errors.push(format!("Parser error: {}", e));
        }

        (tokens, parse_errors)
    }

    /// Update document and trigger diagnostics
    pub async fn update_document(&self, uri: String, version: i32, text: String) {
        let (tokens, parse_errors) = self.analyze_document(&uri, &text).await;
        
        let doc = Document {
            uri: uri.clone(),
            version,
            text,
            tokens,
            parse_errors,
        };
        
        self.documents.lock().await.insert(uri.clone(), doc);
        self.check_diagnostics(&uri).await;
    }

    /// Get document by URI
    pub async fn get_document(&self, uri: &str) -> Option<Document> {
        self.documents.lock().await.get(uri).cloned()
    }

    /// Check and publish diagnostics for a document
    pub async fn check_diagnostics(&self, uri: &str) {
        let doc = match self.get_document(uri).await {
            Some(doc) => doc,
            None => return,
        };

        let mut diagnostics = Vec::new();

        // Add lexer/parser errors
        for error in &doc.parse_errors {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 0 },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: error.clone(),
                ..Default::default()
            });
        }

        // TODO: Add semantic diagnostics (undefined variables, type errors, etc.)

        // Publish diagnostics
        let client_guard = self.client.lock().await;
        if let Some(client) = client_guard.as_ref() {
            let url = Url::parse(uri).unwrap_or_else(|_| Url::parse("file:///").unwrap());
            client.publish_diagnostics(url, diagnostics, Some(doc.version)).await;
        }
    }

    /// Find token at position
    fn token_at_position<'a>(&self, tokens: &'a [Token], line: u32, character: u32) -> Option<&'a Token> {
        // LSP uses 0-indexed positions, lexer uses 1-indexed
        let lsp_line = line + 1;
        let lsp_char = character + 1;
        for token in tokens {
            let token_line = token.span.start_line as u32;
            let token_start = token.span.start_col as u32;
            let token_end = token.span.end_col as u32;
            
            if token_line == lsp_line && lsp_char >= token_start && lsp_char <= token_end {
                return Some(token);
            }
        }
        None
    }
}

/// Type alias for the LSP service
pub type PsudoLanguageServer = Backend;

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), " ".to_string()]),
                    ..Default::default()
                }),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("psudo".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        ..Default::default()
                    }
                )),
                semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
                    SemanticTokensOptions {
                        legend: SemanticTokensLegend {
                            token_types: vec![
                                SemanticTokenType::KEYWORD,
                                SemanticTokenType::VARIABLE,
                                SemanticTokenType::FUNCTION,
                                SemanticTokenType::NUMBER,
                                SemanticTokenType::STRING,
                                SemanticTokenType::OPERATOR,
                            ],
                            token_modifiers: vec![],
                        },
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                        range: Some(false),
                        ..Default::default()
                    }
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "psudo".to_string(),
                version: Some("0.2.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        // Server is initialized
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.update_document(uri, params.text_document.version, params.text_document.text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let version = params.text_document.version;
        
        // For FULL sync, we take the last change as the full document
        let text = params.content_changes
            .last()
            .map(|c| c.text.clone())
            .unwrap_or_default();
        
        self.update_document(uri, version, text).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.documents.lock().await.remove(&uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;
        
        let doc = match self.get_document(&uri).await {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let token = match self.token_at_position(&doc.tokens, position.line, position.character) {
            Some(token) => token,
            None => return Ok(None),
        };

        // Generate hover content based on token type
        let content = match &token.token_type {
            TokenType::IDENTIFIER => {
                format!("**Variable**: `{}`\n\nType: inferred from assignment", token.text)
            }
            TokenType::NUMBER => {
                format!("**Number literal**: `{}`", token.text)
            }
            TokenType::STRING => {
                format!("**String literal**: `{}`", token.text)
            }
            TokenType::LET => "Keyword: `let` - declares a new variable".to_string(),
            TokenType::IF => "Keyword: `if` - conditional branch".to_string(),
            TokenType::ELSE => "Keyword: `else` - alternative branch".to_string(),
            TokenType::WHILE => "Keyword: `while` - loop while condition is true".to_string(),
            TokenType::FOR => "Keyword: `for` - loop with initializer, condition, increment".to_string(),
            TokenType::PRINT => "Keyword: `print` - output expression to console".to_string(),
            TokenType::TRUE => "Keyword: `true` - boolean true".to_string(),
            TokenType::FALSE => "Keyword: `false` - boolean false".to_string(),
            TokenType::AND => "Operator: `and` - logical AND".to_string(),
            TokenType::OR => "Operator: `or` - logical OR".to_string(),
            TokenType::BANG => "Operator: `!` - logical NOT".to_string(),
            TokenType::PLUS => "Operator: `+` - addition / string concatenation".to_string(),
            TokenType::MINUS => "Operator: `-` - subtraction".to_string(),
            TokenType::STAR => "Operator: `*` - multiplication".to_string(),
            TokenType::SLASH => "Operator: `/` - division".to_string(),
            TokenType::EQUAL => "Operator: `=` - assignment".to_string(),
            TokenType::EqualEqual => "Operator: `==` - equality comparison".to_string(),
            TokenType::BangEqual => "Operator: `!=` - inequality comparison".to_string(),
            TokenType::LESS => "Operator: `<` - less than".to_string(),
            TokenType::LessEqual => "Operator: `<=` - less than or equal".to_string(),
            TokenType::GREATER => "Operator: `>` - greater than".to_string(),
            TokenType::GreaterEqual => "Operator: `>=` - greater than or equal".to_string(),
            _ => format!("Token: {:?}", token.token_type),
        };

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: Some(Range {
                start: Position {
                    line: token.span.start_line as u32,
                    character: token.span.start_col as u32,
                },
                end: Position {
                    line: token.span.end_line as u32,
                    character: token.span.end_col as u32,
                },
            }),
        }))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;
        
        let doc = match self.get_document(&uri).await {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let mut items = Vec::new();

        // Keywords
        let keywords = vec![
            ("let", "Declare a variable"),
            ("if", "Conditional branch"),
            ("else", "Else branch"),
            ("while", "While loop"),
            ("for", "For loop"),
            ("print", "Print to console"),
            ("true", "Boolean true"),
            ("false", "Boolean false"),
            ("and", "Logical AND"),
            ("or", "Logical OR"),
            ("fn", "Function declaration"),
            ("return", "Return from function"),
            ("nil", "Null value"),
        ];

        for (label, detail) in keywords {
            items.push(CompletionItem {
                label: label.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(detail.to_string()),
                ..Default::default()
            });
        }

        // Built-in functions / variables from current scope
        // Find variable declarations in the document
        for token in &doc.tokens {
            if token.token_type == TokenType::IDENTIFIER {
                // Check if it's a variable declaration (preceded by 'let')
                items.push(CompletionItem {
                    label: token.text.clone(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    detail: Some("Local variable".to_string()),
                    ..Default::default()
                });
            }
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.to_string();
        let mut tokens_data = Vec::new();
        let mut prev_line = 0u32;
        let mut prev_char = 0u32;

        let documents = self.documents.lock().await;
        if let Some(doc) = documents.get(&uri) {
            for token in &doc.tokens {
                let line = token.span.start_line as u32;
                let start_char = token.span.start_col as u32;
                let length = (token.span.end_col - token.span.start_col) as u32;

                let token_type = match token.token_type {
                    TokenType::NUMBER => 0,
                    TokenType::STRING => 1,
                    TokenType::IDENTIFIER => 2,
                    TokenType::LET | TokenType::FN | TokenType::IF | TokenType::ELSE | TokenType::WHILE | TokenType::FOR | TokenType::PRINT | TokenType::RETURN | TokenType::TRUE | TokenType::FALSE | TokenType::NIL => 3,
                    TokenType::PLUS | TokenType::MINUS | TokenType::STAR | TokenType::SLASH | TokenType::EQUAL | TokenType::BangEqual | TokenType::EqualEqual | TokenType::GREATER | TokenType::GreaterEqual | TokenType::LESS | TokenType::LessEqual | TokenType::BANG | TokenType::AND | TokenType::OR => 4,
                    _ => 5,
                };

                let delta_line = line - prev_line;
                let delta_start = if delta_line == 0 { start_char - prev_char } else { start_char };

                tokens_data.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type,
                    token_modifiers_bitset: 0,
                });

                prev_line = line;
                prev_char = start_char;
            }
        }

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens_data,
        })))
    }
}