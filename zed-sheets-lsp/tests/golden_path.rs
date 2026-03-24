use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::Url;
use zed_sheets_lsp::core::{CellKind, CellValue, CoreSheetDocument, SourceFormat};

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("golden")
        .join(relative)
}

#[test]
fn golden_path_loads_sheet_sidecar_and_linked_markdown() {
    let sheet_path = fixture_path("demo.sheet.md");
    let sheet_text = std::fs::read_to_string(&sheet_path).expect("fixture sheet");
    let sheet_uri = Url::from_file_path(&sheet_path).expect("sheet url");

    let doc = CoreSheetDocument::from_text_and_uri(&sheet_uri, &sheet_text);

    assert_eq!(doc.source.format, SourceFormat::MarkdownTable);
    assert!(doc.core_source_has_table_shape());
    assert!(doc.sidecar.is_some());

    let spec_cell = &doc.sheet.rows[0][1];
    let formula_cell = &doc.sheet.rows[0][2];
    let ref_cell = &doc.sheet.rows[1][2];
    let status_cell = &doc.sheet.rows[0][3];

    assert_eq!(spec_cell.kind, CellKind::Link);
    assert_eq!(formula_cell.kind, CellKind::Derived);
    assert_eq!(ref_cell.kind, CellKind::Reference);
    assert!(matches!(status_cell.value, CellValue::Bool(true)));

    let CellValue::Link(link) = &spec_cell.value else {
        panic!("expected spec cell to be a link");
    };
    assert_eq!(link.path, "./docs/user.md");

    let linked_path = doc
        .resolve_link_target(&sheet_uri, 1, 1)
        .expect("link target should resolve");
    assert_eq!(linked_path, fixture_path("docs/user.md"));
    assert!(linked_path.exists());

    let sidecar = doc.sidecar.as_ref().expect("canonical sidecar");
    assert_eq!(sidecar.version, 2);
    assert!(sidecar.columns.contains_key("Formula"));
}

trait CoreDocAsserts {
    fn core_source_has_table_shape(&self) -> bool;
}

impl CoreDocAsserts for CoreSheetDocument {
    fn core_source_has_table_shape(&self) -> bool {
        self.source.table_block.is_some()
            && self.sheet.headers.len() == 4
            && self.sheet.rows.len() == 2
    }
}
