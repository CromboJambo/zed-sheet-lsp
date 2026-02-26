use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_lsp::{
    lsp_types::{
        self as lsp, CompletionItem, CompletionList, Diagnostic, Hover, HoverContents, MarkupContent,
        Position, Range, TextDocumentIdentifier, TextDocumentPositionParams, Url,
    },
    Client, LanguageServer,
};
use crate::{sidecar::Sidecar, dag::DependencyGraph, diagnostics::Diagnostics, completions::Completions};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Grid {
    pub fn parse_tsv(content: &str) -> Self {
        let mut lines = content.lines();
        let headers = lines
            .next()
            .unwrap_or("")
            .split('\t')
            .map(String::from)
            .collect();
        let rows = lines
            .map(|l| l.split('\t').map(String::from).collect())
            .collect();
        Grid { headers, rows }
    }

    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.headers.iter().position(|h| h == name)
    }

    pub fn get_column(&self, index: usize) -> Option<&Vec<String>> {
        if index < self.rows.len() {
            Some(&self.rows[index])
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Document {
    client: Client,
    uri: Url,
    grid: Grid,
    sidecar: Sidecar,
    dag: DependencyGraph,
    diagnostics: Diagnostics,
    completions: Completions,
}

impl Document {
    pub fn new(client: Client) -> Self {
        Document {
            client,
            uri: Url::parse("file:///dev/null").unwrap(),
            grid: Grid {
                headers: vec![],
                rows: vec![],
            },
            sidecar: Sidecar {
                version: 1,
                columns: HashMap::new(),
                named_ranges: HashMap::new(),
            },
            dag: DependencyGraph::new(),
            diagnostics: Diagnostics::new(),
            completions: Completions::new(),
        }
    }

    pub fn set_uri(&mut self, uri: Url) {
        self.uri = uri;
    }

    pub fn load_content(&mut self, content: &str) {
        self.grid = Grid::parse_tsv(content);
    }

    pub fn load_sidecar(&mut self, sidecar_content: &str) -> Result<(), serde_json::Error> {
        self.sidecar = Sidecar::load_from_json(sidecar_content)?;
        self.update_dependency_graph();
        Ok(())
    }

    fn update_dependency_graph(&mut self) {
        // Clear existing edges
        self.dag.edges.clear();

        // Build dependency graph from sidecar expressions
        for (col_name, col_meta) in &self.sidecar.columns {
            if let Some(nu_expr) = &col_meta.nu_expr {
                // Simple parsing to extract column references - this would be more sophisticated in real implementation
                let references: Vec<String> = nu_expr
                    .split('$')
                    .filter(|s| s.starts_with("row."))
                    .map(|s| s[4..].to_string())
                    .collect();

                for reference in references {
                    if self.grid.column_index(&reference).is_some() {
                        self.dag.add_edge(col_name.clone(), reference);
                    }
                }
            }
        }
    }

    pub fn validate_diagnostics(&mut self) {
        self.diagnostics.clear();

        // Check for circular dependencies
        if self.dag.has_cycle() {
            let range = Range::new(Position::new(0, 0), Position::new(0, 0));
            self.diagnostics.add_error(
                "Circular dependency detected in derived columns".to_string(),
                range,
            );
        }

        // Check for missing columns referenced in expressions
        for (col_name, col_meta) in &self.sidecar.columns {
            if let Some(nu_expr) = &col_meta.nu_expr {
                let references: Vec<String> = nu_expr
                    .split('$')
                    .filter(|s| s.starts_with("row."))
                    .map(|s| s[4..].to_string())
                    .collect();

                for reference in references {
                    if self.grid.column_index(&reference).is_none() {
                        let range = Range::new(Position::new(0, 0), Position::new(0, 0));
                        self.diagnostics.add_warning(
                            format!("Referenced column '{}' not found in TSV", reference),
                            range,
                        );
                    }
                }
            }
        }
    }

    pub fn get_hover_info(&self, position: Position) -> Option<Hover> {
        // Get column name at position
        let line = position.line as usize;
        if line >= self.grid.rows.len() {
            return None;
        }

        let row = &self.grid.rows[line];
        let col_index = position.character as usize;
        if col_index >= row.len() {
            return None;
        }

        // Get column name from headers
        let header_name = if line == 0 {
            self.grid.headers.get(col_index).cloned()
        } else {
            Some(self.grid.headers.get(col_index).cloned().unwrap_or_default())
        };

        if let Some(header) = header_name {
            if let Some(col_meta) = self.sidecar.columns.get(&header) {
                let mut contents = vec![];

                // Add column type
                contents.push(format!("Type: {}", col_meta.type_));

                // Add unit if present
                if let Some(unit) = &col_meta.unit {
                    contents.push(format!("Unit: {}", unit));
                }

                // Add nu expression if derived
                if col_meta.type_ == "derived" {
                    if let Some(expr) = &col_meta.nu_expr {
                        contents.push(format!("Expression: {}", expr));
                    }
                }

                // Add dependents
                let dependents = self.dag.dependents_of(&header);
                if !dependents.is_empty() {
                    contents.push(format!("Dependents: {:?}", dependents));
                }

                let markup_content = MarkupContent {
                    kind: lsp::MarkupKind::Markdown,
                    value: contents.join("\n\n"),
                };

                return Some(Hover {
                    contents: HoverContents::Markup(markup_content),
                    range: Some(Range::new(position, position)),
                });
            }
        }

        None
    }

    pub fn get_completions(&self, position: Position) -> Option<CompletionList> {
        // Simple column name completions for now - would need more sophisticated parsing in real implementation
        let mut items = vec![];

        for col_name in &self.grid.headers {
            items.push(CompletionItem {
                label: col_name.clone(),
                kind: Some(lsp::CompletionItemKind::FIELD),
                detail: Some("Column reference".to_string()),
                documentation: None,
                deprecated: None,
                preselect: None,
                sort_text: None,
                filter_text: None,
                insert_text: Some(col_name.clone()),
                insert_text_format: None,
                text_edit: None,
                additional_text_edits: None,
                command: None,
                commit_characters: None,
                tags: None,
                data: None,
            });
        }

        // Add some common nu builtins for completions
        let nu_builtins = vec!["get", "math", "if", "where", "select", "group-by", "sort-by"];
        for builtin in nu_builtins {
            items.push(CompletionItem {
                label: builtin.to_string(),
                kind: Some(lsp::CompletionItemKind::FUNCTION),
                detail: Some("Nushell builtin".to_string()),
                documentation: None,
                deprecated: None,
                preselect: None,
                sort_text: None,
                filter_text: None,
                insert_text: Some(builtin.to_string()),
                insert_text_format: None,
                text_edit: None,
                additional_text_edits: None,
                command: None,
                commit_characters: None,
                tags: None,
                data: None,
            });
        }

        Some(CompletionList {
            is_incomplete: false,
            items,
        })
    }
}

impl LanguageServer for Document {
    async fn initialize(&self, _params: lsp::InitializeParams) -> Result<lsp::InitializeResult, tower_lsp::jsonrpc::Error> {
        Ok(lsp::InitializeResult {
            capabilities: lsp::ServerCapabilities {
                hover_provider: Some(lsp::HoverProviderCapability::Simple(true)),
                completion_provider: Some(lsp::CompletionOptions {
                    trigger_characters: Some(vec!["$".to_string()]),
                    ..Default::default()
                }),
                diagnostic_provider: Some(lsp::DiagnosticOptions {
                    identifier: Some("zed-sheets".to_string()),
                    ..Default::default()
                }),
                rename_provider: Some(lsp::RenameProviderCapability::Simple(true)),
                definition_provider: Some(lsp::DefinitionProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: lsp::InitializedParams) {
        // Nothing to do here
    }

    async fn shutdown(&self) -> Result<(), tower_lsp::jsonrpc::Error> {
        Ok(())
    }

    async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;

        self.set_uri(uri);
        self.load_content(&content);

        // Try to load sidecar if exists
        let sidecar_path = self.uri.to_file_path().unwrap_or_default();
        let sidecar_filename = sidecar_path.with_extension("zedsheets.json");
        // In a real implementation, we'd read the sidecar file here

        self.validate_diagnostics();

        // Send diagnostics to client
        if self.diagnostics.has_errors() {
            self.client.publish_diagnostics(self.uri.clone(), self.diagnostics.errors.clone(), None).await;
        }
    }

    async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
        let content = params.content_changes.first().unwrap().text.clone();
        self.load_content(&content);

        self.validate_diagnostics();

        // Send diagnostics to client
        if self.diagnostics.has_errors() {
            self.client.publish_diagnostics(self.uri.clone(), self.diagnostics.errors.clone(), None).await;
        }
    }

    async fn hover(&self, params: lsp::HoverParams) -> Result<Option<Hover>, tower_lsp::jsonrpc::Error> {
        let position = params.text_document_position_params.position;
        Ok(self.get_hover_info(position))
    }

    async fn completion(&self, params: lsp::CompletionParams) -> Result<Option<CompletionList>, tower_lsp::jsonrpc::Error> {
        let position = params.text_document_position_params.position;
        Ok(self.get_completions(position))
    }

    async fn diagnostic(&self, _params: lsp::DocumentDiagnosticParams) -> Result<lsp::DocumentDiagnosticReport, tower_lsp::jsonrpc::Error> {
        // This would be implemented for real diagnostics reporting
        Ok(lsp::DocumentDiagnosticReport::Full(lsp::DiagnosticReport {
            items: self.diagnostics.errors.clone(),
        }))
    }

    async fn rename(&self, _params: lsp::RenameParams) -> Result<Option<lsp::WorkspaceEdit>, tower_lsp::jsonrpc::Error> {
        // Rename functionality would be implemented here
        Ok(None)
    }

    async fn definition(&self, _params: lsp::DefinitionParams) -> Result<Option<lsp::LocationLink>, tower_lsp::jsonrpc::Error> {
        // Definition navigation would be implemented here
        Ok(None)
    }
}
