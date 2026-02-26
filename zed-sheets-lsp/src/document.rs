use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Grid {
    pub fn parse_tsv(content: &str) -> Self {
        let mut lines = content.lines();
        let headers = lines
            .next()
            .unwrap_or("")
            .split('\t')
            .map(String::from)
            .collect();
        let rows = lines
            .map(|l| l.split('\t').map(String::from).collect())
            .collect();
        Grid { headers, rows }
    }

    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.headers.iter().position(|h| h == name)
    }

    pub fn get_column(&self, index: usize) -> Option<&Vec<String>> {
        if index < self.rows.len() {
            Some(&self.rows[index])
        } else {
            None
        }
    }
}
