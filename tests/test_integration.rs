use zed_sheets_lsp::document::Document;
use zed_sheets_lsp::sidecar::Sidecar;
use zed_sheets_lsp::dag::DependencyGraph;
use zed_sheets_lsp::diagnostics::Diagnostics;
use tower_lsp::lsp_types::{Position, Url};

#[tokio::main]
async fn main() {
    // Test basic Document creation
    let client = tower_lsp::Client::new(tokio::io::stdin(), tokio::io::stdout());
    let mut doc = Document::new(client);

    // Test loading content
    let tsv_content = "product\tname\tquantity\tprice\tcost\nwidget\tA\t10\t5.0\t3.0\nwidget\tB\t20\t7.0\t4.0";
    doc.load_content(tsv_content);

    assert_eq!(doc.grid.headers, vec!["product", "name", "quantity", "price", "cost"]);
    assert_eq!(doc.grid.rows.len(), 2);

    // Test loading sidecar
    let sidecar_content = r#"{
        "version": 1,
        "columns": {
            "product": { "type": "string" },
            "name": { "type": "string" },
            "quantity": { "type": "number" },
            "price": { "type": "number", "unit": "USD" },
            "cost": { "type": "number", "unit": "USD" },
            "margin": {
                "type": "derived",
                "nu_expr": "$row.price - $row.cost"
            }
        },
        "named_ranges": {}
    }"#;

    doc.load_sidecar(sidecar_content).unwrap();

    assert_eq!(doc.sidecar.version, 1);
    assert!(doc.sidecar.columns.contains_key("product"));
    assert!(doc.sidecar.is_derived_column("margin"));

    // Test dependency graph
    let dag = &doc.dag;
    assert!(!dag.edges.is_empty());

    // Test diagnostics
    doc.validate_diagnostics();
    println!("Diagnostics validation completed");

    // Test hover info
    let position = Position::new(0, 0);
    let hover = doc.get_hover_info(position);
    println!("Hover info test completed");

    // Test completions
    let completions = doc.get_completions(position);
    println!("Completions test completed");

    println!("All integration tests passed!");
}
