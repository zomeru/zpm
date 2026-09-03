//! `package.json` parsing for package-manager detection.
//!
//! Extracts the `packageManager` and `devEngines.packageManager` signals that
//! take precedence over lockfile presence. Handles the `pnpm@<7 → pnpm@6` and
//! `yarn@>1 → yarn@berry` flavor mappings and version-range normalization
//! (e.g. `^8.15.0` → `8.15.0`).

use serde::Deserialize;

use crate::package_manager::Agent;

#[derive(Debug, Deserialize)]
pub(crate) struct PackageJsonManager {
    #[serde(rename = "packageManager")]
    pub package_manager: Option<String>,
    #[serde(rename = "devEngines")]
    pub dev_engines: Option<DevEngines>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DevEngines {
    #[serde(rename = "packageManager")]
    pub(crate) package_manager: Option<DevEnginePackageManager>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DevEnginePackageManager {
    pub(crate) name: Option<String>,
    pub(crate) version: Option<String>,
}

/// Parse `package.json` content and return `(name, version, agent)` when a
/// recognizable package-manager signal is present.
///
/// Returns `None` when the file is not valid JSON or contains no supported
/// manager. The `name` is the raw manager name (`pnpm`, `yarn`, …), `version`
/// is the normalized numeric version when available.
pub(crate) fn parse_package_manager_field(
    content: &str,
) -> Option<(String, Option<String>, Agent)> {
    let json: PackageJsonManager = serde_json::from_str(content).ok()?;
    if let Some(pm) = json.package_manager.as_deref() {
        if let Some(triple) = parse_pm_string(pm) {
            return Some(triple);
        }
    }
    if let Some(dev) = json.dev_engines {
        if let Some(pm) = dev.package_manager {
            if let Some(name) = pm.name {
                let ver = pm.version;
                let normalized = if let Some(v) = &ver {
                    format!("{name}@{v}")
                } else {
                    name.clone()
                };
                if let Some(triple) = parse_pm_string(&normalized) {
                    return Some(triple);
                }
                if let Ok(agent) = name.parse::<Agent>() {
                    return Some((name, ver, agent));
                }
            }
        }
    }
    None
}

/// Parse a `packageManager` string such as `pnpm@8.15.0` or `yarn@3.2.0`.
///
/// Handles leading `^`, version ranges, and maps version-specific flavors:
/// * `yarn@>1` → `yarn@berry`
/// * `pnpm@<7` → `pnpm@6`
pub(crate) fn parse_pm_string(pm: &str) -> Option<(String, Option<String>, Agent)> {
    let pm = pm.trim().trim_start_matches('^');
    if pm.is_empty() {
        return None;
    }
    let (name_part, ver_part) = if let Some(idx) = pm.rfind('@') {
        if idx == 0 {
            return None;
        }
        let name = &pm[..idx];
        let ver = &pm[idx + 1..];
        if ver.is_empty() {
            return None;
        }
        (name, Some(ver))
    } else {
        (pm, None)
    };

    let normalized_ver = ver_part.map(|v| {
        let re_start = v.chars().position(|c| c.is_ascii_digit());
        if let Some(start) = re_start {
            let trimmed = &v[start..];
            let end = trimmed
                .char_indices()
                .find(|(_, c)| !c.is_ascii_digit() && *c != '.')
                .map(|(i, _)| i)
                .unwrap_or(trimmed.len());
            let candidate = &trimmed[..end];
            if candidate.is_empty() {
                v.to_string()
            } else {
                candidate.to_string()
            }
        } else {
            v.to_string()
        }
    });

    let agent = match name_part {
        "npm" => Agent::Npm,
        "yarn" => {
            if let Some(ver) = normalized_ver.as_deref() {
                if let Some(major_str) = ver.split('.').next() {
                    if let Ok(major) = major_str.parse::<u64>() {
                        if major > 1 {
                            Agent::YarnBerry
                        } else {
                            Agent::Yarn
                        }
                    } else {
                        Agent::Yarn
                    }
                } else {
                    Agent::Yarn
                }
            } else {
                Agent::Yarn
            }
        }
        "pnpm" => {
            if let Some(ver) = normalized_ver.as_deref() {
                if let Some(major_str) = ver.split('.').next() {
                    if let Ok(major) = major_str.parse::<u64>() {
                        if major < 7 {
                            Agent::PnpmAt6
                        } else {
                            Agent::Pnpm
                        }
                    } else {
                        Agent::Pnpm
                    }
                } else {
                    Agent::Pnpm
                }
            } else {
                Agent::Pnpm
            }
        }
        "bun" => Agent::Bun,
        "deno" => Agent::Deno,
        "aube" => Agent::Aube,
        "nub" => Agent::Nub,
        _ => return None,
    };

    let name = name_part.to_string();
    Some((name, normalized_ver, agent))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pnpm_with_caret() {
        let (name, ver, agent) = parse_pm_string("pnpm@^8.15.0").unwrap();
        assert_eq!(name, "pnpm");
        assert_eq!(ver.as_deref(), Some("8.15.0"));
        assert_eq!(agent, crate::package_manager::Agent::Pnpm);
    }
}
