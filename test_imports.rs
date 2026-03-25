//! Test to verify zed_sheets_lsp module imports work properly

use zed_sheets_lsp::core::{CoreSheetDocument, SourceFormat};
use zed_sheets_lsp::document::Document;

fn main() {
    // This test verifies that the imports work correctly
    println!("Imports are working correctly!");

    // We can't actually instantiate these without proper setup,
    // but we're just verifying the import paths work

    println!("Document module imported successfully");
    println!("Core module imported successfully");

    // The fact that this compiles means our imports are correct
    println!("Import structure is working properly!");
}
