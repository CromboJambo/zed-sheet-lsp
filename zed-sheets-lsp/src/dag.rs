use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// derived_col → set of source cols it references
    pub edges: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        DependencyGraph {
            edges: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, derived_col: String, source_col: String) {
        self.edges
            .entry(derived_col)
            .or_insert_with(HashSet::new)
            .insert(source_col);
    }

    pub fn has_cycle(&self) -> bool {
        // DFS cycle detection
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in self.edges.keys() {
            if !visited.contains(node) && self.dfs_cycle_check(node, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        false
    }

    fn dfs_cycle_check(
        &self,
        node: &String,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());

        if let Some(dependencies) = self.edges.get(node) {
            for dep in dependencies {
                if !visited.contains(dep) && self.dfs_cycle_check(dep, visited, rec_stack) {
                    return true;
                } else if rec_stack.contains(dep) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    pub fn dependents_of(&self, col: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter_map(|(k, deps)| {
                if deps.contains(col) {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}
