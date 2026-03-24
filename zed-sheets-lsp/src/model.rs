use crate::document::{SourceDocument, SourceFormat};
use tower_lsp::lsp_types::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellAddress {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkTarget {
    pub path: String,
    pub label: Option<String>,
    pub anchor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRef {
    pub address: Option<CellAddress>,
    pub column_name: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaLang {
    MarkdownExpr,
    Nushell,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormulaValue {
    pub expr: String,
    pub lang: FormulaLang,
    pub depends_on: Vec<CellRef>,
    pub cached_result: Option<Box<CellValue>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Date(String),
    Link(LinkTarget),
    Formula(FormulaValue),
    Ref(CellRef),
    Empty,
    RichInline(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellKind {
    Literal,
    Derived,
    Link,
    Reference,
    ComputedPreview,
    Header,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellMeta {
    pub locked: bool,
    pub frozen: bool,
    pub hidden: bool,
    pub editable_layer: LayerId,
    pub source_span: Option<Range>,
    pub format_hint: Option<String>,
    pub validation: Vec<CellDiagnostic>,
    pub tags: Vec<String>,
}

impl Default for CellMeta {
    fn default() -> Self {
        Self {
            locked: false,
            frozen: false,
            hidden: false,
            editable_layer: LayerId("values".to_string()),
            source_span: None,
            format_hint: None,
            validation: Vec::new(),
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub address: CellAddress,
    pub raw: String,
    pub display: String,
    pub value: CellValue,
    pub kind: CellKind,
    pub meta: CellMeta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerKind {
    Values,
    Formulas,
    Links,
    Preview,
    Annotations,
    Schema,
    Pipeline,
    Diff,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub kind: LayerKind,
    pub visible: bool,
    pub locked: bool,
    pub frozen: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SheetModel {
    pub format: SourceFormat,
    pub headers: Vec<Cell>,
    pub rows: Vec<Vec<Cell>>,
    pub layers: Vec<Layer>,
}

impl SheetModel {
    pub fn from_source(source: &SourceDocument) -> Self {
        let headers = match &source.table_block {
            Some(block) => block
                .headers
                .iter()
                .enumerate()
                .map(|(col, cell)| {
                    Self::build_cell(
                        0,
                        col,
                        cell.text.clone(),
                        Some(cell.range),
                        CellKind::Header,
                    )
                })
                .collect(),
            None => source
                .grid
                .headers
                .iter()
                .enumerate()
                .map(|(col, text)| Self::build_cell(0, col, text.clone(), None, CellKind::Header))
                .collect(),
        };

        let rows = match &source.table_block {
            Some(block) => block
                .rows
                .iter()
                .enumerate()
                .map(|(row_idx, row)| {
                    row.iter()
                        .enumerate()
                        .map(|(col_idx, cell)| {
                            Self::build_cell(
                                row_idx + 1,
                                col_idx,
                                cell.text.clone(),
                                Some(cell.range),
                                Self::cell_kind_for(&cell.text),
                            )
                        })
                        .collect()
                })
                .collect(),
            None => source
                .grid
                .rows
                .iter()
                .enumerate()
                .map(|(row_idx, row)| {
                    row.iter()
                        .enumerate()
                        .map(|(col_idx, raw)| {
                            Self::build_cell(
                                row_idx + 1,
                                col_idx,
                                raw.clone(),
                                None,
                                Self::cell_kind_for(raw),
                            )
                        })
                        .collect()
                })
                .collect(),
        };

        Self {
            format: source.format,
            headers,
            rows,
            layers: default_layers(),
        }
    }

    fn build_cell(
        row: usize,
        col: usize,
        raw: String,
        source_span: Option<Range>,
        kind: CellKind,
    ) -> Cell {
        let value = Self::cell_value_for(&raw, &kind);
        let display = match &value {
            CellValue::Link(link) => link.label.clone().unwrap_or_else(|| link.path.clone()),
            CellValue::Formula(formula) => formula.expr.clone(),
            CellValue::Ref(cell_ref) => cell_ref
                .column_name
                .clone()
                .or_else(|| cell_ref.path.clone())
                .unwrap_or_else(|| raw.clone()),
            CellValue::Text(text) => text.clone(),
            CellValue::Number(number) => number.to_string(),
            CellValue::Bool(value) => value.to_string(),
            CellValue::Date(text) => text.clone(),
            CellValue::Empty => String::new(),
            CellValue::RichInline(text) => text.clone(),
        };

        let mut meta = CellMeta {
            source_span,
            ..Default::default()
        };
        meta.editable_layer = match kind {
            CellKind::Derived => LayerId("formulas".to_string()),
            CellKind::Link => LayerId("links".to_string()),
            _ => LayerId("values".to_string()),
        };

        Cell {
            address: CellAddress { row, col },
            raw,
            display,
            value,
            kind,
            meta,
        }
    }

    fn cell_kind_for(raw: &str) -> CellKind {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return CellKind::Literal;
        }
        if looks_like_markdown_link(trimmed).is_some() {
            return CellKind::Link;
        }
        if trimmed.starts_with('=') {
            return CellKind::Derived;
        }
        if trimmed.starts_with('@') {
            return CellKind::Reference;
        }
        CellKind::Literal
    }

    fn cell_value_for(raw: &str, kind: &CellKind) -> CellValue {
        let trimmed = raw.trim();
        match kind {
            CellKind::Header => CellValue::Text(trimmed.to_string()),
            CellKind::Link => looks_like_markdown_link(trimmed)
                .map(CellValue::Link)
                .unwrap_or_else(|| CellValue::Text(trimmed.to_string())),
            CellKind::Derived => CellValue::Formula(FormulaValue {
                expr: trimmed.trim_start_matches('=').trim().to_string(),
                lang: FormulaLang::MarkdownExpr,
                depends_on: Vec::new(),
                cached_result: None,
            }),
            CellKind::Reference => CellValue::Ref(CellRef {
                address: None,
                column_name: Some(trimmed.trim_start_matches('@').to_string()),
                path: None,
            }),
            CellKind::ComputedPreview => CellValue::RichInline(trimmed.to_string()),
            CellKind::Literal => {
                if trimmed.is_empty() {
                    CellValue::Empty
                } else if let Ok(number) = trimmed.parse::<f64>() {
                    CellValue::Number(number)
                } else if trimmed.eq_ignore_ascii_case("true") {
                    CellValue::Bool(true)
                } else if trimmed.eq_ignore_ascii_case("false") {
                    CellValue::Bool(false)
                } else {
                    CellValue::Text(trimmed.to_string())
                }
            }
        }
    }

    pub fn header_cell(&self, col: usize) -> Option<&Cell> {
        self.headers.get(col)
    }

    pub fn data_cell(&self, row: usize, col: usize) -> Option<&Cell> {
        if row == 0 {
            return self.header_cell(col);
        }

        self.rows
            .get(row.checked_sub(1)?)
            .and_then(|cells| cells.get(col))
    }

    pub fn column_names(&self) -> Vec<&str> {
        self.headers.iter().map(|cell| cell.raw.trim()).collect()
    }

    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.headers
            .iter()
            .position(|cell| cell.raw.trim() == name || cell.display == name)
    }
}

impl Default for SheetModel {
    fn default() -> Self {
        Self {
            format: SourceFormat::Tsv,
            headers: Vec::new(),
            rows: Vec::new(),
            layers: default_layers(),
        }
    }
}

impl SourceDocument {
    pub fn to_sheet_model(&self) -> SheetModel {
        SheetModel::from_source(self)
    }
}

fn default_layers() -> Vec<Layer> {
    vec![
        Layer {
            id: LayerId("values".to_string()),
            name: "Values".to_string(),
            kind: LayerKind::Values,
            visible: true,
            locked: false,
            frozen: false,
        },
        Layer {
            id: LayerId("formulas".to_string()),
            name: "Formulas".to_string(),
            kind: LayerKind::Formulas,
            visible: true,
            locked: false,
            frozen: false,
        },
        Layer {
            id: LayerId("links".to_string()),
            name: "Links".to_string(),
            kind: LayerKind::Links,
            visible: true,
            locked: false,
            frozen: false,
        },
        Layer {
            id: LayerId("preview".to_string()),
            name: "Preview".to_string(),
            kind: LayerKind::Preview,
            visible: true,
            locked: false,
            frozen: true,
        },
    ]
}

fn looks_like_markdown_link(raw: &str) -> Option<LinkTarget> {
    if !(raw.starts_with('[') && raw.contains("](") && raw.ends_with(')')) {
        return None;
    }

    let split_index = raw.find("](")?;
    let label = &raw[1..split_index];
    let target = &raw[split_index + 2..raw.len() - 1];
    let (path, anchor) = target
        .split_once('#')
        .map(|(path, anchor)| (path.to_string(), Some(anchor.to_string())))
        .unwrap_or_else(|| (target.to_string(), None));

    Some(LinkTarget {
        path,
        label: Some(label.to_string()),
        anchor,
    })
}

#[cfg(test)]
mod tests {
    use crate::document::SourceDocument;

    use super::{CellKind, CellValue, SheetModel};

    #[test]
    fn builds_rich_cells_from_markdown_table_source() {
        let source = SourceDocument::parse(
            "\
| Key | Spec | Total |
|-----|------|-------|
| user | [User](./user.md) | =price * qty |",
        );

        let sheet = SheetModel::from_source(&source);

        assert_eq!(sheet.headers[0].kind, CellKind::Header);
        assert_eq!(sheet.rows[0][1].kind, CellKind::Link);
        assert_eq!(sheet.rows[0][2].kind, CellKind::Derived);
        assert!(matches!(sheet.rows[0][1].value, CellValue::Link(_)));
    }

    #[test]
    fn source_document_exposes_sheet_model_conversion() {
        let source = SourceDocument::parse(
            "\
| Key | Value |
|-----|-------|
| a   | 42    |",
        );

        let sheet = source.to_sheet_model();

        assert_eq!(sheet.rows.len(), 1);
        assert!(matches!(sheet.rows[0][1].value, CellValue::Number(42.0)));
        assert_eq!(sheet.layers.len(), 4);
    }
}
