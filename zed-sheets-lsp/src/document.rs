use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Grid {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub raw_lines: Vec<String>,
}

impl Grid {
    pub fn parse_tsv(content: &str) -> Self {
        let raw_lines: Vec<String> = content.lines().map(str::to_string).collect();
        let mut lines = content.lines();
        let headers = lines
            .next()
            .unwrap_or("")
            .split('\t')
            .map(String::from)
            .collect();
        let rows = lines
            .map(|line| line.split('\t').map(String::from).collect())
            .collect();

        Self {
            headers,
            rows,
            raw_lines,
        }
    }

    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.headers.iter().position(|header| header == name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    #[serde(rename = "type")]
    pub type_: String,
    pub unit: Option<String>,
    pub nu_expr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamedRange {
    pub rows: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sidecar {
    pub version: u32,
    pub columns: HashMap<String, ColumnMetadata>,
    #[serde(default)]
    pub named_ranges: HashMap<String, NamedRange>,
}

impl Sidecar {
    pub fn load_from_json(content: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(content)
    }
}

#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    pub edges: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn add_edge(&mut self, derived_col: &str, source_col: &str) {
        self.edges
            .entry(derived_col.to_string())
            .or_default()
            .insert(source_col.to_string());
    }

    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();

        self.edges
            .keys()
            .any(|node| !visited.contains(node) && self.visit(node, &mut visited, &mut in_stack))
    }

    fn visit(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        in_stack.insert(node.to_string());

        if let Some(neighbors) = self.edges.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.visit(neighbor, visited, in_stack) {
                        return true;
                    }
                } else if in_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        in_stack.remove(node);
        false
    }
}

#[derive(Debug, Clone, Default)]
struct SheetDocument {
    grid: Grid,
    sidecar: Option<Sidecar>,
}

#[derive(Debug)]
pub struct Document {
    client: Client,
    docs: RwLock<HashMap<Url, SheetDocument>>,
}

impl Document {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            docs: RwLock::new(HashMap::new()),
        }
    }

    fn load_sidecar_for_uri(uri: &Url) -> Option<Sidecar> {
        let sidecar_path = uri
            .to_file_path()
            .ok()
            .and_then(|path| {
                let filename = format!("{}.zedsheets.json", path.file_stem()?.to_string_lossy());
                Some(path.parent()?.join(filename))
            })
            .unwrap_or_else(|| PathBuf::from("/dev/null"));

        if !sidecar_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(sidecar_path).ok()?;
        Sidecar::load_from_json(&content).ok()
    }

    fn extract_column_refs(expr: &str) -> Vec<String> {
        let mut refs = Vec::new();
        for part in expr.split("$row.").skip(1) {
            let mut col = String::new();
            for ch in part.chars() {
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    col.push(ch);
                } else {
                    break;
                }
            }
            if !col.is_empty() {
                refs.push(col);
            }
        }
        refs
    }

    fn build_dependency_graph(sidecar: &Sidecar) -> DependencyGraph {
        let mut dag = DependencyGraph::default();
        for (col_name, meta) in &sidecar.columns {
            if let Some(expr) = &meta.nu_expr {
                for source in Self::extract_column_refs(expr) {
                    dag.add_edge(col_name, &source);
                }
            }
        }
        dag
    }

    fn diagnostics_for(doc: &SheetDocument) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let Some(sidecar) = &doc.sidecar else {
            return diagnostics;
        };

        let dag = Self::build_dependency_graph(sidecar);
        if dag.has_cycle() {
            diagnostics.push(Diagnostic {
                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("zed-sheets".to_string()),
                message: "Circular dependency detected in derived columns".to_string(),
                ..Default::default()
            });
        }

        for meta in sidecar.columns.values() {
            if let Some(expr) = &meta.nu_expr {
                for reference in Self::extract_column_refs(expr) {
                    if doc.grid.column_index(&reference).is_none() {
                        diagnostics.push(Diagnostic {
                            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                            severity: Some(DiagnosticSeverity::WARNING),
                            source: Some("zed-sheets".to_string()),
                            message: format!("Referenced column '{}' not found in TSV", reference),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        diagnostics
    }

    fn column_from_character(line: &str, character: u32) -> usize {
        line.chars()
            .take(character as usize)
            .filter(|ch| *ch == '\t')
            .count()
    }

    async fn publish_diagnostics(&self, uri: &Url) {
        let docs = self.docs.read().await;
        if let Some(doc) = docs.get(uri) {
            let diagnostics = Self::diagnostics_for(doc);
            self.client
                .publish_diagnostics(uri.clone(), diagnostics, None)
                .await;
        }
    }

    async fn hover_for(&self, params: HoverParams) -> Option<Hover> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let docs = self.docs.read().await;
        let doc = docs.get(&uri)?;

        let line = doc.grid.raw_lines.get(pos.line as usize)?;
        let col = Self::column_from_character(line, pos.character);
        let header = doc.grid.headers.get(col)?;

        if pos.line == 0 {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**Column:** `{}`", header),
                }),
                range: None,
            });
        }

        let row_index = pos.line as usize - 1;
        let value = doc
            .grid
            .rows
            .get(row_index)
            .and_then(|row| row.get(col))
            .cloned()
            .unwrap_or_default();

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**Column:** `{}`\n\n**Value:** `{}`", header, value),
            }),
            range: None,
        })
    }

    async fn completions_for(&self, params: CompletionParams) -> Option<CompletionResponse> {
        let uri = params.text_document_position.text_document.uri;
        let docs = self.docs.read().await;
        let doc = docs.get(&uri)?;

        let trigger = params
            .context
            .as_ref()
            .and_then(|ctx| ctx.trigger_character.as_deref());

        let items: Vec<CompletionItem> = doc
            .grid
            .headers
            .iter()
            .map(|header| {
                let label = match trigger {
                    Some("$") => format!("row.{}", header),
                    _ => header.clone(),
                };
                CompletionItem {
                    label,
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some("TSV column".to_string()),
                    ..Default::default()
                }
            })
            .collect();

        Some(CompletionResponse::Array(items))
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
                    trigger_characters: Some(vec!["$".to_string(), ".".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _params: InitializedParams) {}

    async fn shutdown(&self) -> Result<(), tower_lsp::jsonrpc::Error> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let grid = Grid::parse_tsv(&params.text_document.text);
        let sidecar = Self::load_sidecar_for_uri(&uri);

        {
            let mut docs = self.docs.write().await;
            docs.insert(uri.clone(), SheetDocument { grid, sidecar });
        }

        self.publish_diagnostics(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params
            .content_changes
            .first()
            .map(|change| change.text.as_str())
            .unwrap_or("");
        let grid = Grid::parse_tsv(text);

        {
            let mut docs = self.docs.write().await;
            let sidecar = docs.get(&uri).and_then(|doc| doc.sidecar.clone());
            docs.insert(uri.clone(), SheetDocument { grid, sidecar });
        }

        self.publish_diagnostics(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        {
            let mut docs = self.docs.write().await;
            docs.remove(&uri);
        }
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>, tower_lsp::jsonrpc::Error> {
        Ok(self.hover_for(params).await)
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>, tower_lsp::jsonrpc::Error> {
        Ok(self.completions_for(params).await)
    }
}
