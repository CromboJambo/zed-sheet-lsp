use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Range};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    pub errors: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics { errors: Vec::new() }
    }

    pub fn add_error(&mut self, message: String, range: Range) {
        self.errors.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("zed-sheets".to_string()),
            message,
            tags: None,
            related_information: None,
        });
    }

    pub fn add_warning(&mut self, message: String, range: Range) {
        self.errors.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::WARNING),
            code: None,
            code_description: None,
            source: Some("zed-sheets".to_string()),
            message,
            tags: None,
            related_information: None,
        });
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn clear(&mut self) {
        self.errors.clear();
    }
}
