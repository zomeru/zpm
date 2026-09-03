//! Shared catalog types.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CatalogInfo {
    pub name: String,
    pub packages: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CatalogConfig {
    pub file_path: PathBuf,
    pub catalogs: Vec<CatalogInfo>,
    pub has_default_catalog: bool,
    pub has_named_catalogs: bool,
}

pub trait CatalogProvider {
    fn detect(&self, cwd: &Path) -> Option<CatalogConfig>;
    fn find_package<'a>(&self, config: &'a CatalogConfig, pkg: &str) -> Option<&'a CatalogInfo>;
}
