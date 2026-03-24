use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::core::CoreSheetDocument;
use crate::model::Cell;
use crate::sidecar::Sidecar;

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

    pub fn from_markdown_table(table: &TableBlock) -> Self {
        Self {
            headers: table.headers.iter().map(|cell| cell.text.clone()).collect(),
            rows: table
                .rows
                .iter()
                .map(|row| row.iter().map(|cell| cell.text.clone()).collect())
                .collect(),
            raw_lines: table.raw_lines.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TableCell {
    pub text: String,
    pub range: Range,
}

#[derive(Debug, Clone, Default)]
pub struct TableBlock {
    pub headers: Vec<TableCell>,
    pub rows: Vec<Vec<TableCell>>,
    pub raw_lines: Vec<String>,
}

impl TableBlock {
    pub fn parse(content: &str) -> Option<Self> {
        let lines: Vec<&str> = content.lines().collect();

        for start in 0..lines.len() {
            let header_line = lines[start];
            let Some(header_cells) = Self::parse_row(header_line, start as u32) else {
                continue;
            };

            let Some(delimiter_line) = lines.get(start + 1) else {
                continue;
            };

            if !Self::is_delimiter_row(delimiter_line) {
                continue;
            }

            let mut rows = Vec::new();
            let mut raw_lines = vec![header_line.to_string(), (*delimiter_line).to_string()];
            let mut line_index = start + 2;

            while let Some(line) = lines.get(line_index) {
                if line.trim().is_empty() {
                    break;
                }

                let Some(row_cells) = Self::parse_row(line, line_index as u32) else {
                    break;
                };

                if row_cells.len() != header_cells.len() {
                    break;
                }

                raw_lines.push((*line).to_string());
                rows.push(row_cells);
                line_index += 1;
            }

            return Some(Self {
                headers: header_cells,
                rows,
                raw_lines,
            });
        }

        None
    }

    fn is_delimiter_row(line: &str) -> bool {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            return false;
        }

        let cells: Vec<&str> = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();

        !cells.is_empty()
            && cells.iter().all(|cell| {
                let body = cell.trim_matches(':');
                !body.is_empty() && body.chars().all(|ch| ch == '-')
            })
    }

    fn parse_row(line: &str, line_index: u32) -> Option<Vec<TableCell>> {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            return None;
        }

        let mut cells = Vec::new();
        let mut current = String::new();
        let mut cell_start: Option<u32> = None;
        let mut in_cell = false;

        for (offset, ch) in line.char_indices() {
            if ch == '|' {
                if in_cell {
                    let start = cell_start.unwrap_or(offset as u32);
                    cells.push(TableCell {
                        text: current.trim().to_string(),
                        range: Range::new(
                            Position::new(line_index, start),
                            Position::new(line_index, offset as u32),
                        ),
                    });
                    current.clear();
                    cell_start = None;
                } else {
                    in_cell = true;
                }
                continue;
            }

            if in_cell {
                if cell_start.is_none() {
                    cell_start = Some(offset as u32);
                }
                current.push(ch);
            }
        }

        Some(cells)
    }

    fn cell_at_position(&self, pos: Position) -> Option<(usize, usize, &TableCell)> {
        for (col_index, cell) in self.headers.iter().enumerate() {
            if Self::position_in_range(pos, cell.range) {
                return Some((0, col_index, cell));
            }
        }

        for (row_index, row) in self.rows.iter().enumerate() {
            for (col_index, cell) in row.iter().enumerate() {
                if Self::position_in_range(pos, cell.range) {
                    return Some((row_index + 1, col_index, cell));
                }
            }
        }

        None
    }

    fn position_in_range(pos: Position, range: Range) -> bool {
        pos.line == range.start.line
            && pos.character >= range.start.character
            && pos.character <= range.end.character
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    MarkdownTable,
    Tsv,
}

#[derive(Debug, Clone)]
pub struct SourceDocument {
    pub format: SourceFormat,
    pub grid: Grid,
    pub table_block: Option<TableBlock>,
}

impl SourceDocument {
    pub fn parse(content: &str) -> Self {
        if let Some(table_block) = TableBlock::parse(content) {
            return Self {
                format: SourceFormat::MarkdownTable,
                grid: Grid::from_markdown_table(&table_block),
                table_block: Some(table_block),
            };
        }

        Self {
            format: SourceFormat::Tsv,
            grid: Grid::parse_tsv(content),
            table_block: None,
        }
    }
}

impl Default for SourceDocument {
    fn default() -> Self {
        Self {
            format: SourceFormat::Tsv,
            grid: Grid::default(),
            table_block: None,
        }
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
    core: CoreSheetDocument,
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

        let Some(sidecar) = &doc.core.sidecar else {
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
                    if doc.core.sheet.column_index(&reference).is_none() {
                        diagnostics.push(Diagnostic {
                            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                            severity: Some(DiagnosticSeverity::WARNING),
                            source: Some("zed-sheets".to_string()),
                            message: format!(
                                "Referenced column '{}' not found in source table",
                                reference
                            ),
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

        if let Some(table) = &doc.core.source.table_block {
            let (row_index, col, cell) = table.cell_at_position(pos)?;
            let header_cell = doc.core.sheet.header_cell(col)?;

            if row_index == 0 {
                return Some(Self::hover_for_cell(
                    header_cell,
                    header_cell.display.as_str(),
                    Some(cell.range),
                ));
            }

            let data_cell = doc.core.sheet.data_cell(row_index, col)?;
            return Some(Self::hover_for_cell(
                data_cell,
                header_cell.display.as_str(),
                Some(cell.range),
            ));
        }

        let line = doc.core.source.grid.raw_lines.get(pos.line as usize)?;
        let col = Self::column_from_character(line, pos.character);
        let header_cell = doc.core.sheet.header_cell(col)?;

        if pos.line == 0 {
            return Some(Self::hover_for_cell(
                header_cell,
                header_cell.display.as_str(),
                None,
            ));
        }

        let row_index = pos.line as usize;
        let cell = doc.core.sheet.data_cell(row_index, col)?;
        Some(Self::hover_for_cell(
            cell,
            header_cell.display.as_str(),
            None,
        ))
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
            .core
            .sheet
            .column_names()
            .into_iter()
            .map(|header| {
                let label = match trigger {
                    Some("$") => format!("row.{}", header),
                    _ => header.to_string(),
                };
                CompletionItem {
                    label,
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some(match doc.core.source.format {
                        SourceFormat::MarkdownTable => "Markdown table column".to_string(),
                        SourceFormat::Tsv => "TSV column".to_string(),
                    }),
                    ..Default::default()
                }
            })
            .collect();

        Some(CompletionResponse::Array(items))
    }

    fn hover_for_cell(cell: &Cell, header_label: &str, range: Option<Range>) -> Hover {
        let title = match cell.kind {
            crate::model::CellKind::Header => format!("**Column:** `{}`", header_label),
            _ => format!(
                "**Column:** `{}`\n\n**Value:** `{}`",
                header_label, cell.display
            ),
        };

        let detail = match &cell.value {
            crate::model::CellValue::Link(link) => format!("\n\n**Link:** `{}`", link.path),
            crate::model::CellValue::Formula(formula) => {
                format!("\n\n**Formula:** `{}`", formula.expr)
            }
            crate::model::CellValue::Ref(cell_ref) => {
                let target = cell_ref
                    .column_name
                    .clone()
                    .or_else(|| cell_ref.path.clone())
                    .unwrap_or_default();
                format!("\n\n**Ref:** `{}`", target)
            }
            _ => String::new(),
        };

        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("{title}{detail}"),
            }),
            range,
        }
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
        let core = CoreSheetDocument::from_text_and_uri(&uri, &params.text_document.text);

        {
            let mut docs = self.docs.write().await;
            docs.insert(uri.clone(), SheetDocument { core });
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

        {
            let mut docs = self.docs.write().await;
            let sidecar = docs.get(&uri).and_then(|doc| doc.core.sidecar.clone());
            let core = CoreSheetDocument::from_text(text).with_sidecar(sidecar);
            docs.insert(uri.clone(), SheetDocument { core });
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

#[cfg(test)]
mod tests {
    use super::{Grid, SourceDocument, SourceFormat, TableBlock};
    use tower_lsp::lsp_types::Position;

    #[test]
    fn parses_markdown_pipe_table_into_grid() {
        let content = "\
| Key | Var 1 | Var 2 |
|-----|-------|-------|
| A   | foo   | bar   |
| B   | baz   | qux   |";

        let table = TableBlock::parse(content).expect("expected markdown table");
        let grid = Grid::from_markdown_table(&table);

        assert_eq!(grid.headers, vec!["Key", "Var 1", "Var 2"]);
        assert_eq!(grid.rows[0], vec!["A", "foo", "bar"]);
        assert_eq!(grid.rows[1], vec!["B", "baz", "qux"]);
    }

    #[test]
    fn maps_cursor_position_to_markdown_cell() {
        let content = "\
| Key | Var 1 | Var 2 |
|-----|-------|-------|
| A   | foo   | bar   |";

        let table = TableBlock::parse(content).expect("expected markdown table");
        let (row_index, col_index, cell) = table
            .cell_at_position(Position::new(2, 9))
            .expect("expected markdown cell");

        assert_eq!(row_index, 1);
        assert_eq!(col_index, 1);
        assert_eq!(cell.text, "foo");
    }

    #[test]
    fn prefers_markdown_table_as_primary_source_format() {
        let content = "\
| Key | Value |
|-----|-------|
| A   | 1     |";

        let source = SourceDocument::parse(content);

        assert_eq!(source.format, SourceFormat::MarkdownTable);
        assert!(source.table_block.is_some());
        assert_eq!(source.grid.headers, vec!["Key", "Value"]);
    }
}
