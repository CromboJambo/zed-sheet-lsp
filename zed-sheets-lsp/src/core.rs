use std::path::PathBuf;
use tower_lsp::lsp_types::Url;

pub use crate::document::{
    DependencyGraph, Grid, SourceDocument, SourceFormat, TableBlock, TableCell,
};
pub use crate::model::{
    Cell, CellAddress, CellDiagnostic, CellKind, CellMeta, CellRef, CellValue, DiagnosticSeverity,
    FormulaLang, FormulaValue, Layer, LayerId, LayerKind, LinkTarget, SheetModel,
};
pub use crate::sidecar::{
    Sidecar, SidecarFormat, SidecarResolution, canonical_sidecar_path_for_source,
    canonical_stem_for_source, legacy_sidecar_path_for_source, resolve_sidecar_for_uri,
};

#[derive(Debug, Clone, Default)]
pub struct CoreSheetDocument {
    pub source: SourceDocument,
    pub sheet: SheetModel,
    pub sidecar: Option<Sidecar>,
}

impl CoreSheetDocument {
    pub fn from_text(text: &str) -> Self {
        let source = SourceDocument::parse(text);
        let sheet = source.to_sheet_model();

        Self {
            source,
            sheet,
            sidecar: None,
        }
    }

    pub async fn from_text_and_uri(uri: &Url, text: &str) -> Self {
        let mut doc = Self::from_text(text);
        doc.sidecar = crate::sidecar::load_sidecar_for_uri(uri);
        doc
    }

    pub fn with_sidecar(mut self, sidecar: Option<Sidecar>) -> Self {
        self.sidecar = sidecar;
        self
    }

    pub fn resolve_link_target(&self, source_uri: &Url, row: usize, col: usize) -> Option<PathBuf> {
        let cell = self.sheet.data_cell(row, col)?;
        resolve_cell_link_target(source_uri, cell)
    }
}

pub fn resolve_cell_link_target(source_uri: &Url, cell: &Cell) -> Option<PathBuf> {
    let CellValue::Link(link) = &cell.value else {
        return None;
    };

    let source_path = source_uri.to_file_path().ok()?;
    let base_dir = source_path.parent()?;
    Some(normalize_relative_path(base_dir.join(&link.path)))
}

fn normalize_relative_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tower_lsp::lsp_types::Url;

    use super::{CellValue, CoreSheetDocument, SourceFormat};

    #[test]
    fn builds_core_document_from_markdown_source() {
        let doc = CoreSheetDocument::from_text(
            "\
| Key | Spec |
|-----|------|
| user | [User](./user.md) |",
        );

        assert_eq!(doc.source.format, SourceFormat::MarkdownTable);
        assert_eq!(doc.sheet.rows.len(), 1);
        assert!(matches!(doc.sheet.rows[0][1].value, CellValue::Link(_)));
        assert!(doc.sidecar.is_none());
    }

    #[test]
    fn resolves_relative_markdown_link_targets() {
        let doc = CoreSheetDocument::from_text(
            "\
| Key | Spec |
|-----|------|
| user | [User](./docs/user.md) |",
        );
        let uri = Url::from_file_path(Path::new("/tmp/demo.sheet.md")).expect("file url");

        let target = doc
            .resolve_link_target(&uri, 1, 1)
            .expect("link target should resolve");

        assert_eq!(target, Path::new("/tmp/docs/user.md"));
    }
}
