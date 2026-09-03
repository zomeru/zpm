//! Configuration — file, env, and defaults.
//!
//! Precedence: CLI flags > env > config file > defaults.
//!
//! Config file handling preserves backwards compatibility with `~/.nirc` (ini)
//! and the `NI_*` env namespace, while preferring `~/.config/zpm/config.toml`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::package_manager::Agent;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub default_manager: Option<String>,
    #[serde(default)]
    pub global_manager: Option<String>,
    #[serde(default)]
    pub interactive: Option<bool>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub auto_install: Option<bool>,
    #[serde(default)]
    pub catalog: Option<bool>,
}

impl Config {
    pub fn default_manager_agent(&self) -> Option<Agent> {
        self.default_manager
            .as_deref()
            .and_then(|s| s.parse::<Agent>().ok())
    }

    pub fn global_manager_agent(&self) -> Option<Agent> {
        self.global_manager
            .as_deref()
            .and_then(|s| s.parse::<Agent>().ok())
    }
}

pub fn config_file_path() -> PathBuf {
    if let Ok(custom) = std::env::var("ZPM_CONFIG") {
        if !custom.is_empty() && custom != "false" {
            return PathBuf::from(custom);
        }
    }
    if let Ok(custom) = std::env::var("NI_CONFIG_FILE") {
        if !custom.is_empty() && custom != "false" {
            return PathBuf::from(custom);
        }
    }

    if let Some(dir) = dirs::config_dir() {
        dir.join("zpm").join("config.toml")
    } else if let Some(home) = dirs::home_dir() {
        home.join(".config").join("zpm").join("config.toml")
    } else {
        PathBuf::from(".config/zpm/config.toml")
    }
}

/// Load config from file (TOML or `.nirc` ini) then apply env overrides.
///
/// Env handling is split into `apply_env_overrides` for clarity and testability.
pub fn load_config() -> Config {
    let mut config = Config::default();

    let path = config_file_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(parsed) = toml::from_str::<Config>(&content) {
                    config = parsed;
                }
            } else if let Ok(parsed) = parse_ini_config(&content) {
                config = merge_config(config, parsed);
            }
        }
    }

    // Back-compat: fall back to ~/.nirc when zpm config is absent.
    if !path.exists() {
        if let Some(home) = dirs::home_dir() {
            let nirc = home.join(".nirc");
            if nirc.exists() {
                if let Ok(content) = std::fs::read_to_string(&nirc) {
                    if let Ok(parsed) = parse_ini_config(&content) {
                        config = merge_config(config, parsed);
                    }
                }
            }
        }
    }

    apply_env_overrides(config)
}

fn apply_env_overrides(mut config: Config) -> Config {
    if let Ok(val) = std::env::var("ZPM_DEFAULT_MANAGER") {
        if !val.is_empty() {
            config.default_manager = Some(val);
        }
    }
    if let Ok(val) = std::env::var("NI_DEFAULT_AGENT") {
        if config.default_manager.is_none() && !val.is_empty() {
            config.default_manager = Some(val);
        }
    }

    if let Ok(val) = std::env::var("ZPM_GLOBAL_MANAGER") {
        if !val.is_empty() {
            config.global_manager = Some(val);
        }
    }
    if let Ok(val) = std::env::var("NI_GLOBAL_AGENT") {
        if config.global_manager.is_none() && !val.is_empty() {
            config.global_manager = Some(val);
        }
    }

    if let Ok(val) = std::env::var("ZPM_AUTO_INSTALL") {
        config.auto_install = Some(val == "true" || val == "1");
    }
    if let Ok(val) = std::env::var("NI_AUTO_INSTALL") {
        if config.auto_install.is_none() {
            config.auto_install = Some(val == "true" || val == "1");
        }
    }

    if let Ok(val) = std::env::var("ZPM_CATALOG") {
        config.catalog = Some(val != "false" && val != "0");
    }
    if let Ok(val) = std::env::var("NI_CATALOG") {
        if config.catalog.is_none() {
            config.catalog = Some(val != "false" && val != "0");
        }
    }

    if let Ok(val) = std::env::var("ZPM_NO_INTERACTIVE") {
        config.interactive = Some(val != "true" && val != "1");
    }

    config
}

fn merge_config(mut base: Config, other: Config) -> Config {
    if other.default_manager.is_some() {
        base.default_manager = other.default_manager;
    }
    if other.global_manager.is_some() {
        base.global_manager = other.global_manager;
    }
    if other.interactive.is_some() {
        base.interactive = other.interactive;
    }
    if other.color.is_some() {
        base.color = other.color;
    }
    if other.auto_install.is_some() {
        base.auto_install = other.auto_install;
    }
    if other.catalog.is_some() {
        base.catalog = other.catalog;
    }
    base
}

fn parse_ini_config(content: &str) -> Result<Config, String> {
    let mut cfg = Config::default();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with(';')
            || line.starts_with('[')
        {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim();
            let v = v.trim().trim_matches('"').trim_matches('\'').to_string();
            match k {
                "defaultAgent" | "default_manager" | "defaultManager" => {
                    cfg.default_manager = Some(v)
                }
                "globalAgent" | "global_manager" | "globalManager" => cfg.global_manager = Some(v),
                "autoInstall" | "auto_install" => cfg.auto_install = Some(v == "true" || v == "1"),
                "catalog" => cfg.catalog = Some(v != "false" && v != "0"),
                "interactive" => cfg.interactive = Some(v == "true" || v == "1"),
                "color" => cfg.color = Some(v),
                _ => {}
            }
        }
    }
    Ok(cfg)
}

pub fn get_default_agent(config: &Config) -> Option<Agent> {
    if let Some(agent) = config.default_manager_agent() {
        return Some(agent);
    }
    if std::env::var("CI").is_ok() {
        return Some(Agent::Npm);
    }
    None
}

pub fn get_global_agent(config: &Config) -> Agent {
    config.global_manager_agent().unwrap_or(Agent::Npm)
}

pub fn is_interactive(config: &Config) -> bool {
    if std::env::var("ZPM_NO_INTERACTIVE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        return false;
    }
    if std::env::var("NI_NO_INTERACTIVE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        return false;
    }
    if let Some(v) = config.interactive {
        return v;
    }
    console::user_attended()
}

pub fn should_use_color(config: &Config) -> bool {
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    if let Some(c) = &config.color {
        match c.to_lowercase().as_str() {
            "never" | "false" | "0" => return false,
            "always" | "true" | "1" => return true,
            _ => {}
        }
    }
    console::user_attended()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ini_basic() {
        let ini = r#"
defaultAgent = pnpm
globalAgent = npm
"#;
        let cfg = parse_ini_config(ini).unwrap();
        assert_eq!(cfg.default_manager.as_deref(), Some("pnpm"));
        assert_eq!(cfg.global_manager.as_deref(), Some("npm"));
    }

    #[test]
    fn env_override() {
        let mut cfg = Config {
            default_manager: Some("npm".to_string()),
            ..Default::default()
        };
        cfg.default_manager = Some("pnpm".to_string());
        assert_eq!(cfg.default_manager_agent(), Some(Agent::Pnpm));
    }
}
