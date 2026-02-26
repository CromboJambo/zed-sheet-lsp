use zed_sheets_lsp::document::Grid;
use zed_sheets_lsp::sidecar::Sidecar;
use zed_sheets_lsp::dag::DependencyGraph;
use zed_sheets_lsp::diagnostics::Diagnostics;

#[tokio::main]
async fn main() {
    // Test Grid parsing
    let tsv_content = "product\tname\tquantity\tprice\tcost\nwidget\tA\t10\t5.0\t3.0\nwidget\tB\t20\t7.0\t4.0";
    let grid = Grid::parse_tsv(tsv_content);

    assert_eq!(grid.headers, vec!["product", "name", "quantity", "price", "cost"]);
    assert_eq!(grid.rows.len(), 2);

    // Test Sidecar loading
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

    let sidecar = Sidecar::load_from_json(sidecar_content).unwrap();
    assert_eq!(sidecar.version, 1);
    assert!(sidecar.columns.contains_key("product"));
    assert!(sidecar.is_derived_column("margin"));

    // Test DependencyGraph
    let mut dag = DependencyGraph::new();
    dag.add_edge("margin".to_string(), "price".to_string());
    dag.add_edge("margin".to_string(), "cost".to_string());

    assert_eq!(dag.edges.get("margin").unwrap().len(), 2);

    // Test diagnostics
    let mut diagnostics = Diagnostics::new();
    diagnostics.add_error("Test error".to_string(), tower_lsp::lsp_types::Range::default());
    assert!(diagnostics.has_errors());

    println!("All tests passed!");
}
