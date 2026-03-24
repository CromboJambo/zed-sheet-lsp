use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "unit")]
    pub unit: Option<String>,
    #[serde(rename = "nu_expr")]
    pub nu_expr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedRange {
    pub rows: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sidecar {
    pub version: u32,
    pub columns: HashMap<String, ColumnMetadata>,
    #[serde(default)]
    pub named_ranges: HashMap<String, NamedRange>,
}

impl Sidecar {
    pub fn load_from_json(content: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(content)
    }

    pub fn get_column(&self, name: &str) -> Option<&ColumnMetadata> {
        self.columns.get(name)
    }

    pub fn is_derived_column(&self, name: &str) -> bool {
        if let Some(col) = self.columns.get(name) {
            col.type_ == "derived"
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidecarFormat {
    CanonicalNustage,
    LegacyZedSheets,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidecarResolution {
    pub path: PathBuf,
    pub format: SidecarFormat,
}

pub fn canonical_stem_for_source(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_string_lossy();
    Some(stem.strip_suffix(".sheet").unwrap_or(&stem).to_string())
}

pub fn canonical_sidecar_path_for_source(path: &Path) -> Option<PathBuf> {
    let stem = canonical_stem_for_source(path)?;
    Some(path.parent()?.join(format!("{stem}.nustage.json")))
}

pub fn legacy_sidecar_path_for_source(path: &Path) -> Option<PathBuf> {
    let stem = canonical_stem_for_source(path)?;
    Some(path.parent()?.join(format!("{stem}.zedsheets.json")))
}

pub fn resolve_sidecar_for_uri(uri: &Url) -> Option<SidecarResolution> {
    let path = uri.to_file_path().ok()?;

    let canonical = canonical_sidecar_path_for_source(&path)?;
    if canonical.exists() {
        return Some(SidecarResolution {
            path: canonical,
            format: SidecarFormat::CanonicalNustage,
        });
    }

    let legacy = legacy_sidecar_path_for_source(&path)?;
    if legacy.exists() {
        return Some(SidecarResolution {
            path: legacy,
            format: SidecarFormat::LegacyZedSheets,
        });
    }

    None
}

pub fn load_sidecar_for_uri(uri: &Url) -> Option<Sidecar> {
    let resolution = resolve_sidecar_for_uri(uri)?;
    let content = std::fs::read_to_string(resolution.path).ok()?;
    Sidecar::load_from_json(&content).ok()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tower_lsp::lsp_types::Url;

    use super::{
        canonical_sidecar_path_for_source, legacy_sidecar_path_for_source, load_sidecar_for_uri,
    };

    #[test]
    fn resolves_sheet_markdown_to_canonical_sidecar_name() {
        let path = Path::new("/tmp/example.sheet.md");

        let canonical = canonical_sidecar_path_for_source(path).expect("canonical path");
        let legacy = legacy_sidecar_path_for_source(path).expect("legacy path");

        assert_eq!(canonical, Path::new("/tmp/example.nustage.json"));
        assert_eq!(legacy, Path::new("/tmp/example.zedsheets.json"));
    }

    #[test]
    fn loads_canonical_sidecar_before_legacy_name() {
        let temp_dir =
            std::env::temp_dir().join(format!("zed-sheets-sidecar-{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).expect("temp dir");

        let source = temp_dir.join("demo.sheet.md");
        let canonical = temp_dir.join("demo.nustage.json");
        let legacy = temp_dir.join("demo.zedsheets.json");

        std::fs::write(&source, "| Key |\n|-----|\n| A   |\n").expect("source");
        std::fs::write(
            &canonical,
            r#"{"version":2,"columns":{"Key":{"type":"string"}},"named_ranges":{}}"#,
        )
        .expect("canonical sidecar");
        std::fs::write(
            &legacy,
            r#"{"version":1,"columns":{"Old":{"type":"string"}},"named_ranges":{}}"#,
        )
        .expect("legacy sidecar");

        let uri = Url::from_file_path(&source).expect("file url");
        let sidecar = load_sidecar_for_uri(&uri).expect("resolved sidecar");

        assert_eq!(sidecar.version, 2);
        assert!(sidecar.columns.contains_key("Key"));

        let _ = std::fs::remove_file(&source);
        let _ = std::fs::remove_file(&canonical);
        let _ = std::fs::remove_file(&legacy);
        let _ = std::fs::remove_dir(&temp_dir);
    }
}
