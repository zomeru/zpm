//! Filesystem traversal helpers.
//!
//! Centralizes ancestor walking and upward file discovery so that `detection`,
//! `workspace`, and `catalog` do not duplicate `lookup_dirs` logic or create
//! circular dependencies.

use std::path::{Path, PathBuf};

/// Return ancestor directories from `cwd` up to the filesystem root.
///
/// Mirrors the previous `lookup_dirs` helper: canonicalizes `cwd` when
/// possible, then walks `parent()` until it stabilises. The returned vector
/// is ordered closest-first, which is the natural precedence order for both
/// detection and workspace discovery.
pub(crate) fn ancestors(cwd: &Path) -> Vec<PathBuf> {
    let cwd = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    let mut dirs = Vec::new();
    let mut current = Some(cwd);
    while let Some(dir) = current {
        dirs.push(dir.clone());
        let parent = dir.parent().map(|p| p.to_path_buf());
        if parent.as_deref() == Some(dir.as_path()) {
            break;
        }
        current = parent;
    }
    dirs
}
