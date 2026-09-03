//! Workspace discovery.
//!
//! Answers questions such as:
//! * What is the workspace root for the current directory?
//! * What package owns the current directory?
//! * What scripts are available?
//!
//! This module owns filesystem traversal for workspace concerns and does not
//! depend on `detection` or `package_manager` — it only depends on the small
//! `crate::fs` leaf helper. That keeps the dependency direction
//! `cli -> workspace` clean and makes the logic easy to extend (e.g. new
//! workspace file types) without touching detection.

use serde_json::Value;
use std::path::{Path, PathBuf};

use crate::fs;

/// Find the workspace root by walking up from `cwd` and looking for
/// well-known workspace indicators.
///
/// Mirrors the previous `detection::find_workspace_root` but lives here as
/// the canonical implementation. Detection re-exports it for backwards
/// compatibility.
pub fn find_workspace_root(cwd: &Path) -> Option<PathBuf> {
    for dir in fs::ancestors(cwd) {
        if dir.join("pnpm-workspace.yaml").exists() {
            return Some(dir);
        }
        if dir.join("pnpm-workspace.yml").exists() {
            return Some(dir);
        }
        if dir.join(".yarnrc.yml").exists() {
            return Some(dir);
        }
        if dir.join("rush.json").exists() {
            return Some(dir);
        }
        if dir.join("deno.json").exists() || dir.join("deno.jsonc").exists() {
            return Some(dir);
        }
        let pkg_path = dir.join("package.json");
        if pkg_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg_path) {
                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                    if json.get("workspaces").is_some() {
                        return Some(dir);
                    }
                }
            }
        }
    }
    None
}

/// Find the closest `package.json` by walking up from `cwd`.
pub fn find_closest_package_json(cwd: &Path) -> Option<PathBuf> {
    for dir in fs::ancestors(cwd) {
        let pkg = dir.join("package.json");
        if pkg.exists() {
            return Some(pkg);
        }
    }
    None
}

/// Read and parse a `package.json` path.
pub fn read_package_json(path: &Path) -> Option<Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
}

/// Read scripts from the closest `package.json`.
///
/// Returns `Vec<(name, command, description)>`. The description is sourced
/// from `scripts-info` when present, otherwise falls back to the command
/// itself. Keys prefixed with `?` are ignored (ni-compatible convention).
pub fn read_package_scripts(cwd: &Path) -> Vec<(String, String, String)> {
    let pkg_path = find_closest_package_json(cwd).unwrap_or_else(|| cwd.join("package.json"));
    let content = std::fs::read_to_string(pkg_path).unwrap_or_default();
    let json: Value = serde_json::from_str(&content).unwrap_or(Value::Null);
    let scripts = json.get("scripts").and_then(|v| v.as_object());
    let scripts_info = json.get("scripts-info").and_then(|v| v.as_object());
    let mut result = Vec::new();
    if let Some(map) = scripts {
        for (k, v) in map {
            if k.starts_with('?') {
                continue;
            }
            let cmd = v.as_str().unwrap_or("").to_string();
            let desc = scripts_info
                .and_then(|info| info.get(k))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    let qkey = format!("?{k}");
                    json.get("scripts")
                        .and_then(|s| s.get(&qkey))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| cmd.clone());
            result.push((k.clone(), cmd, desc));
        }
    }
    result
}

/// Find all `package.json` files under `cwd`, excluding common ignored
/// directories (`node_modules`, `.git`, etc.). Used for monorepo package
/// selection UIs.
///
/// This is intentionally a simple DFS; it is only invoked from interactive
/// prompts so performance impact is negligible.
pub fn find_packages(cwd: &Path) -> Vec<PathBuf> {
    let mut packages = Vec::new();
    let ignore = [
        "node_modules",
        "dist",
        "public",
        "fixture",
        "fixtures",
        ".git",
        "target",
    ];
    fn walk(dir: &Path, out: &mut Vec<PathBuf>, ignore: &[&str]) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if ignore.contains(&name) {
                            continue;
                        }
                        if name.starts_with('.') {
                            continue;
                        }
                    }
                    walk(&path, out, ignore);
                } else if path.file_name().and_then(|n| n.to_str()) == Some("package.json") {
                    out.push(path);
                }
            }
        }
    }
    walk(cwd, &mut packages, &ignore);
    if packages.is_empty() {
        let pkg = cwd.join("package.json");
        if pkg.exists() {
            packages.push(pkg);
        }
    }
    packages
}
