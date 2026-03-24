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

    pub fn from_text_and_uri(uri: &Url, text: &str) -> Self {
        let mut doc = Self::from_text(text);
        doc.sidecar = crate::sidecar::load_sidecar_for_uri(uri);
        doc
    }

    pub fn with_sidecar(mut self, sidecar: Option<Sidecar>) -> Self {
        self.sidecar = sidecar;
        self
    }
}

#[cfg(test)]
mod tests {
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
}
