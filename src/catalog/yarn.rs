//! Yarn Berry catalog detection (via `.yarnrc.yml`).

use std::path::Path;

use super::common::{CatalogConfig, CatalogInfo, CatalogProvider};

#[derive(Debug)]
pub struct YarnCatalog;

impl CatalogProvider for YarnCatalog {
    fn detect(&self, cwd: &Path) -> Option<CatalogConfig> {
        let mut dir = cwd.canonicalize().ok().unwrap_or_else(|| cwd.to_path_buf());
        loop {
            let file = dir.join(".yarnrc.yml");
            if file.exists() {
                if let Ok(content) = std::fs::read_to_string(&file) {
                    let has_default = content.contains("catalog:");
                    let has_named = content.contains("catalogs:");
                    if !has_default && !has_named {
                        return None;
                    }
                    return Some(CatalogConfig {
                        file_path: file,
                        catalogs: vec![],
                        has_default_catalog: has_default,
                        has_named_catalogs: has_named,
                    });
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
