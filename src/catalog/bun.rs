//! Bun catalog detection (via `package.json` `catalog`/`catalogs` fields).

use std::path::Path;

use super::common::{CatalogConfig, CatalogInfo, CatalogProvider};

#[derive(Debug)]
pub struct BunCatalog;

impl CatalogProvider for BunCatalog {
    fn detect(&self, cwd: &Path) -> Option<CatalogConfig> {
        let mut dir = cwd.canonicalize().ok().unwrap_or_else(|| cwd.to_path_buf());
        loop {
            let file = dir.join("package.json");
            if file.exists() {
                if let Ok(content) = std::fs::read_to_string(&file) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        let workspaces = json.get("workspaces");
                        let has_catalog =
                            json.get("catalog").is_some() || json.get("catalogs").is_some();
                        let nested_catalog = workspaces.and_then(|w| w.get("catalog")).is_some()
                            || workspaces.and_then(|w| w.get("catalogs")).is_some();
                        if has_catalog || nested_catalog {
                            let has_default = json.get("catalog").is_some()
                                || workspaces.and_then(|w| w.get("catalog")).is_some();
                            let has_named = json.get("catalogs").is_some()
                                || workspaces.and_then(|w| w.get("catalogs")).is_some();
                            return Some(CatalogConfig {
                                file_path: file,
                                catalogs: vec![],
                                has_default_catalog: has_default,
                                has_named_catalogs: has_named,
                            });
                        }
                    }
                }
            }
            if let Some(parent) = dir.parent() {
                if parent == dir {
                    break;
                }
                dir = parent.to_path_buf();
            } else {
                break;
            }
        }
        None
    }

    fn find_package<'a>(&self, _config: &'a CatalogConfig, _pkg: &str) -> Option<&'a CatalogInfo> {
        None
    }
}
