//! Catalog support — pnpm / Yarn Berry / Bun dependency catalogs.
//!
//! The provider model isolates per-manager detection so that YAML/JSON
//! parsing quirks and file locations remain local. Full read-modify-write
//! (with serialization preservation) is scaffolded for the next milestone;
//! the current implementation provides defensive detection and `catalog:`
//! reference generation without brittle string replacement.

pub mod bun;
pub mod common;
pub mod pnpm;
pub mod yarn;

pub use bun::BunCatalog;
pub use common::{CatalogConfig, CatalogInfo, CatalogProvider};
pub use pnpm::PnpmCatalog;
pub use yarn::YarnCatalog;

pub fn get_catalog_provider(
    agent: crate::package_manager::Agent,
) -> Option<Box<dyn CatalogProvider>> {
    match agent {
        crate::package_manager::Agent::Pnpm
        | crate::package_manager::Agent::PnpmAt6
        | crate::package_manager::Agent::PnpmRush => Some(Box::new(PnpmCatalog)),
        crate::package_manager::Agent::YarnBerry => Some(Box::new(YarnCatalog)),
        crate::package_manager::Agent::Bun => Some(Box::new(BunCatalog)),
        _ => None,
    }
}

pub fn get_catalog_ref(catalog_name: &str) -> String {
    if catalog_name == "default" {
        "catalog:".to_string()
    } else {
        format!("catalog:{catalog_name}")
    }
}
