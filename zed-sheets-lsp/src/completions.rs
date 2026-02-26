use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Position};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completions {
    pub items: Vec<CompletionItem>,
}

impl Completions {
    pub fn new() -> Self {
        Completions { items: Vec::new() }
    }

    pub fn add_column_completion(&mut self, column_name: String, position: Position) {
        self.items.push(CompletionItem {
            label: column_name.clone(),
            kind: Some(CompletionItemKind::FIELD),
            detail: Some("Column reference".to_string()),
            documentation: None,
            deprecated: None,
            preselect: None,
            sort_text: None,
            filter_text: None,
            insert_text: Some(column_name),
            insert_text_format: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: None,
            tags: None,
            data: None,
        });
    }

    pub fn add_nu_builtin_completion(&mut self, builtin_name: String, position: Position) {
        self.items.push(CompletionItem {
            label: builtin_name.clone(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Nushell builtin".to_string()),
            documentation: None,
            deprecated: None,
            preselect: None,
            sort_text: None,
            filter_text: None,
            insert_text: Some(builtin_name),
            insert_text_format: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: None,
            tags: None,
            data: None,
        });
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}
