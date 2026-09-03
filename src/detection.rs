//! Package-manager detection.
//!
//! The detection pipeline follows ni's precedence but is split into
//! distinct, testable stages:
//!
//! ```text
//! collect candidate dirs (ancestors)
//! → early deno check (target dir only, ni-compatible)
//! → per-dir: rush → lockfile (with packageManager override) → packageManager field → install metadata
//! → deno fallback (ancestors)
//! ```
//!
//! Signals are defined in submodules:
//! * [`package_json`] — `packageManager` / `devEngines` parsing
//! * [`signals`] — lockfile and install-metadata tables
//!
//! Precedence is explicit: within a directory the `LOCKS` order determines
//! which lockfile wins, and closer directories win over ancestors. This
//! keeps ambiguous-signal handling deterministic and mirrors
//! `package-manager-detector`.

use std::path::{Path, PathBuf};

use crate::fs;
use crate::package_manager::Agent;

pub(crate) mod package_json;
pub(crate) mod signals;

use package_json::parse_package_manager_field;
use signals::{INSTALL_METADATA, LOCKS, is_yarn_classic_metadata};

/// Result of a successful detection.
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub agent: Agent,
    pub name: String,
    pub version: Option<String>,
    pub path: PathBuf,
    pub strategy: String,
}

/// Source that produced a detection — useful for diagnostics and for making
/// precedence rules explicit in tests without forcing callers to parse the
/// `strategy` string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionSource {
    DenoJson,
    RushJson,
    Lockfile,
    PackageManagerField,
    InstallMetadata,
    Fallback,
}

/// Detect the package manager for `cwd` by walking its ancestors.
///
/// Returns `None` when no signal is found.
pub fn detect(cwd: &Path) -> Option<DetectionResult> {
    let dirs = fs::ancestors(cwd);

    if let Some(res) = early_deno_check(&dirs) {
        return Some(res);
    }

    for dir in &dirs {
        if let Some(res) = detect_at_dir(dir) {
            return Some(res);
        }
    }

    deno_fallback(&dirs)
}

fn early_deno_check(dirs: &[PathBuf]) -> Option<DetectionResult> {
    let first = dirs.first()?;
    if first.join("deno.json").exists() || first.join("deno.jsonc").exists() {
        return Some(DetectionResult {
            agent: Agent::Deno,
            name: "deno".to_string(),
            version: None,
            path: first.join("deno.json"),
            strategy: "deno.json".to_string(),
        });
    }
    None
}

fn deno_fallback(dirs: &[PathBuf]) -> Option<DetectionResult> {
    for dir in dirs {
        if dir.join("deno.json").exists() || dir.join("deno.jsonc").exists() {
            return Some(DetectionResult {
                agent: Agent::Deno,
                name: "deno".to_string(),
                version: None,
                path: dir.join("deno.json"),
                strategy: "deno.json".to_string(),
            });
        }
    }
    None
}

fn detect_at_dir(dir: &Path) -> Option<DetectionResult> {
    if let Some(res) = check_rush(dir) {
        return Some(res);
    }
    if let Some(res) = check_lockfile_with_pm_override(dir) {
        return Some(res);
    }
    if let Some(res) = check_package_manager_field(dir) {
        return Some(res);
    }
    if let Some(res) = check_install_metadata(dir) {
        return Some(res);
    }
    None
}

fn check_rush(dir: &Path) -> Option<DetectionResult> {
    if dir.join("rush.json").exists() {
        return Some(DetectionResult {
            agent: Agent::PnpmRush,
            name: "pnpm".to_string(),
            version: None,
            path: dir.join("rush.json"),
            strategy: "rush.json".to_string(),
        });
    }
    None
}

fn check_lockfile_with_pm_override(dir: &Path) -> Option<DetectionResult> {
    for (lock, name) in LOCKS {
        if dir.join(lock).exists() {
            // `packageManager` field overrides the lockfile mapping when present
            // in the same directory — this matches ni's disambiguation for
            // yarn berry vs classic and pnpm@6.
            let pkg_path = dir.join("package.json");
            if pkg_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&pkg_path) {
                    if let Some((detected_name, ver, agent)) = parse_package_manager_field(&content)
                    {
                        return Some(DetectionResult {
                            agent,
                            name: detected_name,
                            version: ver,
                            path: pkg_path,
                            strategy: "packageManager-field".to_string(),
                        });
                    }
                }
            }
            let agent = match *name {
                "aube" => Agent::Aube,
                "bun" => Agent::Bun,
                "deno" => Agent::Deno,
                "nub" => Agent::Nub,
                "pnpm" => Agent::Pnpm,
                "yarn" => Agent::Yarn,
                "npm" => Agent::Npm,
                _ => continue,
            };
            return Some(DetectionResult {
                agent,
                name: (*name).to_string(),
                version: None,
                path: dir.join(lock),
                strategy: "lockfile".to_string(),
            });
        }
    }
    None
}

fn check_package_manager_field(dir: &Path) -> Option<DetectionResult> {
    let pkg_path = dir.join("package.json");
    if !pkg_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&pkg_path).ok()?;
    let (name, ver, agent) = parse_package_manager_field(&content)?;
    Some(DetectionResult {
        agent,
        name,
        version: ver,
        path: pkg_path,
        strategy: "packageManager-field".to_string(),
    })
}

fn check_install_metadata(dir: &Path) -> Option<DetectionResult> {
    for (meta, name, is_dir) in INSTALL_METADATA {
        let p = dir.join(meta);
        let exists = if *is_dir { p.is_dir() } else { p.exists() };
        if !exists {
            continue;
        }
        let agent = match *name {
            "aube" => Agent::Aube,
            "deno" => Agent::Deno,
            "pnpm" => Agent::Pnpm,
            "yarn" => {
                if is_yarn_classic_metadata(&p) {
                    Agent::Yarn
                } else {
                    Agent::YarnBerry
                }
            }
            "npm" => Agent::Npm,
            "bun" => Agent::Bun,
            _ => continue,
        };
        return Some(DetectionResult {
            agent,
            name: (*name).to_string(),
            version: None,
            path: p,
            strategy: "install-metadata".to_string(),
        });
    }
    None
}

/// Detect with a fallback default agent when no signal is found.
pub fn detect_with_fallback(cwd: &Path, default_agent: Option<Agent>) -> Option<DetectionResult> {
    detect(cwd).or_else(|| {
        default_agent.map(|agent| DetectionResult {
            agent,
            name: agent.as_str().to_string(),
            version: None,
            path: cwd.to_path_buf(),
            strategy: "default".to_string(),
        })
    })
}

/// Whether any lockfile (or equivalent signal) exists in `cwd` or an ancestor.
///
/// Used by the CLI's `--frozen-if-present` logic and for deciding whether a
/// `has_lock` hint should be passed to command resolution.
pub fn has_lockfile(cwd: &Path) -> bool {
    for dir in fs::ancestors(cwd) {
        for (lock, _) in LOCKS {
            if dir.join(lock).exists() {
                return true;
            }
        }
        if dir.join("rush.json").exists() {
            return true;
        }
        let pkg = dir.join("package.json");
        if pkg.exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg) {
                if parse_package_manager_field(&content).is_some() {
                    return true;
                }
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Workspace helpers — canonical implementations live in `crate::workspace`.
// Re-exported here for backwards compatibility (tests and `cli` historically
// imported through `detection`). New code should prefer `crate::workspace`.
// ---------------------------------------------------------------------------

/// Find the workspace root. Delegates to `crate::workspace` as the canonical
/// owner of workspace semantics.
pub fn find_workspace_root(cwd: &Path) -> Option<PathBuf> {
    crate::workspace::find_workspace_root(cwd)
}

/// Find the closest `package.json`. Delegates to `crate::workspace`.
pub fn find_closest_package_json(cwd: &Path) -> Option<PathBuf> {
    crate::workspace::find_closest_package_json(cwd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn agent_str(cwd: &Path) -> Option<String> {
        detect(cwd).map(|r| r.agent.as_str().to_string())
    }

    #[test]
    fn detects_npm_via_lock() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("package-lock.json"), "{}").unwrap();
        assert_eq!(agent_str(tmp.path()).as_deref(), Some("npm"));
    }

    #[test]
    fn detects_pnpm_via_lock() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("pnpm-lock.yaml"), "").unwrap();
        assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm"));
    }

    #[test]
    fn detects_yarn_via_lock() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("yarn.lock"), "").unwrap();
        assert_eq!(agent_str(tmp.path()).as_deref(), Some("yarn"));
    }

    #[test]
    fn detects_bun_via_lock() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("bun.lockb"), "").unwrap();
        assert_eq!(agent_str(tmp.path()).as_deref(), Some("bun"));
    }

    #[test]
    fn detects_via_package_manager_field() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"packageManager":"pnpm@8.15.0"}"#,
        )
        .unwrap();
        assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm"));
    }

    #[test]
    fn detects_yarn_berry_via_package_manager() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"packageManager":"yarn@3.2.0"}"#,
        )
        .unwrap();
        assert_eq!(agent_str(tmp.path()).as_deref(), Some("yarn@berry"));
    }

    #[test]
    fn detects_pnpm6_via_package_manager() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"packageManager":"pnpm@6.32.0"}"#,
        )
        .unwrap();
        assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm@6"));
    }

    #[test]
    fn detects_deno_via_deno_json() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("deno.json"), "{}").unwrap();
        assert_eq!(agent_str(tmp.path()).as_deref(), Some("deno"));
    }

    #[test]
    fn detects_rush_pnpm() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("rush.json"), "{}").unwrap();
        assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm-rush"));
    }

    #[test]
    fn nested_detection() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("pnpm-lock.yaml"), "").unwrap();
        let nested = tmp.path().join("packages/app");
        fs::create_dir_all(&nested).unwrap();
        assert_eq!(agent_str(&nested).as_deref(), Some("pnpm"));
    }

    #[test]
    fn has_lockfile_check() {
        let tmp = TempDir::new().unwrap();
        assert!(!has_lockfile(tmp.path()));
        fs::write(tmp.path().join("package-lock.json"), "{}").unwrap();
        assert!(has_lockfile(tmp.path()));
    }
}
