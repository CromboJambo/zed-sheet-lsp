use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tower_lsp::Client;
use tower_lsp::LanguageServer;
use tower_lsp::lsp_types::*;

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
        self.rows.get(index)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub type_: String,
    pub unit: Option<String>,
    pub nu_expr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedRange {
    pub rows: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sidecar {
    pub version: String,
    pub columns: HashMap<String, ColumnMetadata>,
    pub named_ranges: HashMap<String, NamedRange>,
}

impl Sidecar {
    pub fn load_from_json(content: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(content)
    }

    pub fn get_column(&self, name: &str) -> Option<&ColumnMetadata> {
        self.columns.get(name)
    }

    pub fn is_derived_column(&self, name: &str) -> bool {
        self.columns
            .get(name)
            .map_or(false, |col| col.nu_expr.is_some())
    }
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub edges: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        DependencyGraph {
            edges: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.edges
            .entry(from.to_string())
            .or_insert_with(HashSet::new)
            .insert(to.to_string());
    }

    pub fn has_cycle(&self) -> bool {
        // Simple cycle detection using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in self.edges.keys() {
            if !visited.contains(node) && self.dfs_cycle_check(node, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        false
    }

    fn dfs_cycle_check(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = self.edges.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) && self.dfs_cycle_check(neighbor, visited, rec_stack)
                {
                    return true;
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    pub fn dependents_of(&self, node: &str) -> Vec<String> {
        let mut result = Vec::new();
        for (from, tos) in &self.edges {
            if tos.contains(node) {
                result.push(from.clone());
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    client: Client,
    uri: Url,
    grid: Grid,
    sidecar: Sidecar,
    dag: DependencyGraph,
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
                version: "0.1".to_string(),
                columns: HashMap::new(),
                named_ranges: HashMap::new(),
            },
            dag: DependencyGraph::new(),
        }
    }

    pub fn set_uri(&mut self, uri: Url) {
        self.uri = uri;
    }

    pub fn load_content(&mut self, content: &str) {
        self.grid = Grid::parse_tsv(content);
    }

    pub fn load_sidecar(&mut self, content: &str) -> Result<(), serde_json::Error> {
        self.sidecar = Sidecar::load_from_json(content)?;
        Ok(())
    }

    fn update_dependency_graph(&mut self) {
        // Clear existing dependencies
        self.dag = DependencyGraph::new();

        // Add dependencies based on column expressions
        for (col_name, col_meta) in &self.sidecar.columns {
            if let Some(nu_expr) = &col_meta.nu_expr {
                let references: Vec<String> = nu_expr
                    .split('$')
                    .filter(|s| s.starts_with("row."))
                    .map(|s| s[4..].to_string())
                    .collect();

                for reference in references {
                    self.dag.add_edge(&reference, col_name);
                }
            }
        }
    }

    pub async fn validate_diagnostics(&mut self) {
        // Check for circular dependencies
        if self.dag.has_cycle() {
            let diagnostic = Diagnostic {
                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("zed-sheets".to_string()),
                message: "Circular dependency detected in derived columns".to_string(),
                related_information: None,
                tags: None,
                data: None,
            };

            self.client
                .publish_diagnostics(self.uri.clone(), vec![diagnostic], None)
                .await;
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
                        let diagnostic = Diagnostic {
                            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: None,
                            code_description: None,
                            source: Some("zed-sheets".to_string()),
                            message: format!("Referenced column '{}' not found in TSV", reference),
                            related_information: None,
                            tags: None,
                            data: None,
                        };

                        self.client
                            .publish_diagnostics(self.uri.clone(), vec![diagnostic], None)
                            .await;
                    }
                }
            }
        }
    }

    pub fn infer_type_from_value(&self, value: &str) -> String {
        // Default to string if empty or whitespace only
        if value.trim().is_empty() {
            return "null".to_string();
        }

        // Try to parse as integer first
        if value.parse::<i64>().is_ok() {
            return "integer".to_string();
        }

        // Try to parse as float
        if value.parse::<f64>().is_ok() {
            return "float".to_string();
        }

        // Check for date format (simple check)
        if self.is_date_format(value) {
            return "date".to_string();
        }

        // Default to string
        "string".to_string()
    }

    pub fn infer_column_type(&self, column_name: &str) -> String {
        let col_index = self.grid.column_index(column_name);
        if let Some(index) = col_index {
            let mut type_counts: HashMap<String, usize> = HashMap::new();
            let mut null_count = 0;

            // Count types in the column
            for row in &self.grid.rows {
                if index < row.len() {
                    let value = &row[index];
                    if value.trim().is_empty() {
                        null_count += 1;
                        type_counts.insert(
                            "null".to_string(),
                            type_counts.get("null").unwrap_or(&0) + 1,
                        );
                    } else {
                        let inferred_type = self.infer_type_from_value(value);
                        type_counts.insert(
                            inferred_type.clone(),
                            type_counts.get(&inferred_type).unwrap_or(&0) + 1,
                        );
                    }
                }
            }

            // Return the most common type, or string if there's no clear winner
            if !type_counts.is_empty() {
                let max_count = *type_counts.values().max().unwrap();
                for (type_name, count) in &type_counts {
                    if *count == max_count && *type_name != "null" {
                        return type_name.clone();
                    }
                }
            }

            // If all values are null or no data
            if null_count > 0
                && type_counts.get("null").unwrap_or(&0) >= (self.grid.rows.len() as usize / 2)
            {
                return "null".to_string();
            }

            // Default to string when we can't determine a better type
            "string".to_string()
        } else {
            "unknown".to_string()
        }
    }

    pub fn is_date_format(&self, value: &str) -> bool {
        // Simple date format checks - could be expanded
        let formats = ["%Y-%m-%d", "%m/%d/%Y", "%d/%m/%Y"];
        for format in formats.iter() {
            if chrono::NaiveDate::parse_from_str(value, format).is_ok() {
                return true;
            }
        }
        false
    }

    pub fn get_hover_info(&self, params: HoverParams) -> Option<Hover> {
        let position = params.text_document_position_params.position;
        let row_index = position.line as usize;
        let col_index = position.character as usize;

        // Check if we're hovering a header cell (row 0)
        if row_index == 0 && col_index < self.grid.headers.len() {
            let column_name = &self.grid.headers[col_index];
            let inferred_type = self.infer_column_type(column_name);

            // Calculate null count for this column
            let mut null_count = 0;
            for row in &self.grid.rows {
                if col_index < row.len() && row[col_index].trim().is_empty() {
                    null_count += 1;
                }
            }

            // Create markdown hover content
            let markdown_content = format!(
                "## Column: {}\n\n**Type:** {}\n**Null Count:** {}",
                column_name, inferred_type, null_count
            );

            Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(markdown_content)),
                range: Some(Range::new(position, position)),
            })
        } else if row_index < self.grid.rows.len() && col_index < self.grid.headers.len() {
            let column_name = &self.grid.headers[col_index];
            let value = &self.grid.rows[row_index][col_index];
            let inferred_type = self.infer_type_from_value(value);

            // Create markdown hover content for cell
            let markdown_content = format!(
                "## Cell: {}\n\n**Type:** {}\n**Row Index:** {}",
                column_name,
                inferred_type,
                row_index + 1
            );

            Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(markdown_content)),
                range: Some(Range::new(position, position)),
            })
        } else {
            None
        }
    }

    pub fn get_completions(&self, params: CompletionParams) -> Option<CompletionResponse> {
        // For now we'll return basic completions
        let trigger_character = params
            .context
            .as_ref()
            .and_then(|ctx| ctx.trigger_character.as_ref());

        if let Some(trigger) = trigger_character {
            if trigger == "." {
                let mut items = Vec::new();
                for header in &self.grid.headers {
                    items.push(CompletionItem {
                        label: header.clone(),
                        kind: Some(CompletionItemKind::FIELD),
                        detail: Some(String::from("Column")),
                        ..Default::default()
                    });
                }
                Some(CompletionResponse::Array(items))
            } else if trigger == "$" {
                let mut items = Vec::new();
                for header in &self.grid.headers {
                    items.push(CompletionItem {
                        label: format!("row.{}", header),
                        kind: Some(CompletionItemKind::FIELD),
                        detail: Some(String::from("Column reference")),
                        ..Default::default()
                    });
                }
                Some(CompletionResponse::Array(items))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Document {
    pub async fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        let content = params.text_document.text;
        self.set_uri(params.text_document.uri);
        self.load_content(&content);

        // Try to load sidecar if exists
        let sidecar_path = self
            .uri
            .to_file_path()
            .ok()
            .and_then(|path| {
                let sidecar_filename = path
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
                    .unwrap_or_default()
                    + ".zedsheets.json";
                let parent = path.parent()?;
                Some(parent.join(&sidecar_filename))
            })
            .unwrap_or_else(|| PathBuf::from("/dev/null"));

        if sidecar_path.exists() {
            let sidecar_content = std::fs::read_to_string(sidecar_path).unwrap_or_default();
            let _ = self.load_sidecar(&sidecar_content);
        }

        // Validate diagnostics
        self.validate_diagnostics().await;
    }

    pub async fn did_change(&mut self, params: DidChangeTextDocumentParams) {
        let content = params
            .content_changes
            .first()
            .map(|c| c.text.as_str())
            .unwrap_or("");
        self.load_content(content);

        // Validate diagnostics
        self.validate_diagnostics().await;
    }

    pub async fn hover(&self, params: HoverParams) -> Option<Hover> {
        self.get_hover_info(params)
    }

    pub async fn completion(&self, params: CompletionParams) -> Option<CompletionResponse> {
        self.get_completions(params)
    }

    pub async fn diagnostics(&self, _params: DocumentDiagnosticParams) -> DocumentDiagnosticReport {
        // For now we'll just return an empty report - actual diagnostics would be more complex
        DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
            related_documents: Some(HashMap::new()),
            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                result_id: None,
                items: Vec::new(),
            },
        })
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Document {
    async fn initialize(
        &self,
        _params: InitializeParams,
    ) -> Result<InitializeResult, tower_lsp::jsonrpc::Error> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Zed Sheets LSP".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), "$".to_string()]),
                    all_commit_characters: None,
                    resolve_provider: None,
                    completion_item: None,
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: None,
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                )),
                definition_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                })),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        // No initialization needed
    }

    async fn shutdown(&self) -> Result<(), tower_lsp::jsonrpc::Error> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = self.clone();
        doc.did_open(params).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let doc = self.clone();
        doc.did_change(params).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>, tower_lsp::jsonrpc::Error> {
        Ok(self.hover(params).await)
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>, tower_lsp::jsonrpc::Error> {
        Ok(self.completion(params).await)
    }
}
