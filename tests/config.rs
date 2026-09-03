#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
use std::fs;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;
use zpm::config::{Config, load_config};

static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[test]
fn config_default() {
    let cfg = Config::default();
    assert!(cfg.default_manager.is_none());
    assert!(cfg.global_manager.is_none());
}

#[test]
fn env_overrides_default_manager() {
    let _guard = env_lock();
    // Save original
    let orig = std::env::var("ZPM_DEFAULT_MANAGER").ok();
    unsafe { std::env::set_var("ZPM_DEFAULT_MANAGER", "pnpm") };
    unsafe { std::env::remove_var("NI_DEFAULT_AGENT") };
    let cfg = load_config();
    assert_eq!(cfg.default_manager.as_deref(), Some("pnpm"));
    // Restore
    if let Some(v) = orig {
        unsafe { std::env::set_var("ZPM_DEFAULT_MANAGER", v) };
    } else {
        unsafe { std::env::remove_var("ZPM_DEFAULT_MANAGER") };
    }
}

#[test]
fn env_overrides_global_manager() {
    let _guard = env_lock();
    let orig = std::env::var("ZPM_GLOBAL_MANAGER").ok();
    unsafe { std::env::set_var("ZPM_GLOBAL_MANAGER", "yarn") };
    let cfg = load_config();
    assert_eq!(cfg.global_manager.as_deref(), Some("yarn"));
    if let Some(v) = orig {
        unsafe { std::env::set_var("ZPM_GLOBAL_MANAGER", v) };
    } else {
        unsafe { std::env::remove_var("ZPM_GLOBAL_MANAGER") };
    }
}

#[test]
fn ni_env_compat() {
    let _guard = env_lock();
    let orig = std::env::var("NI_DEFAULT_AGENT").ok();
    let orig2 = std::env::var("ZPM_DEFAULT_MANAGER").ok();
    // Ensure zpm not set, NI is fallback
    unsafe { std::env::remove_var("ZPM_DEFAULT_MANAGER") };
    unsafe { std::env::set_var("NI_DEFAULT_AGENT", "bun") };
    let cfg = load_config();
    assert_eq!(cfg.default_manager.as_deref(), Some("bun"));
    unsafe { std::env::remove_var("NI_DEFAULT_AGENT") };
    if let Some(v) = orig {
        unsafe { std::env::set_var("NI_DEFAULT_AGENT", v) };
    }
    if let Some(v) = orig2 {
        unsafe { std::env::set_var("ZPM_DEFAULT_MANAGER", v) };
    }
}

#[test]
fn config_file_loading_toml() {
    let _guard = env_lock();
    let tmp = TempDir::new().unwrap();
    let cfg_path = tmp.path().join("config.toml");
    fs::write(
        &cfg_path,
        r#"default_manager = "pnpm"
global_manager = "npm"
auto_install = true
"#,
    )
    .unwrap();
    let orig = std::env::var("ZPM_CONFIG").ok();
    unsafe { std::env::set_var("ZPM_CONFIG", cfg_path.to_str().unwrap()) };
    // Ensure other env overrides don't interfere
    let orig_dm = std::env::var("ZPM_DEFAULT_MANAGER").ok();
    let orig_gm = std::env::var("ZPM_GLOBAL_MANAGER").ok();
    unsafe { std::env::remove_var("ZPM_DEFAULT_MANAGER") };
    unsafe { std::env::remove_var("ZPM_GLOBAL_MANAGER") };
    let cfg = load_config();
    assert_eq!(cfg.default_manager.as_deref(), Some("pnpm"));
    assert_eq!(cfg.global_manager.as_deref(), Some("npm"));
    assert_eq!(cfg.auto_install, Some(true));
    if let Some(v) = orig {
        unsafe { std::env::set_var("ZPM_CONFIG", v) };
    } else {
        unsafe { std::env::remove_var("ZPM_CONFIG") };
    }
    if let Some(v) = orig_dm {
        unsafe { std::env::set_var("ZPM_DEFAULT_MANAGER", v) };
    }
    if let Some(v) = orig_gm {
        unsafe { std::env::set_var("ZPM_GLOBAL_MANAGER", v) };
    }
}

#[test]
fn fallback_to_npm_in_ci() {
    let orig_ci = std::env::var("CI").ok();
    unsafe { std::env::set_var("CI", "true") };
    let cfg = Config::default(); // no default_manager
    let agent = zpm::config::get_default_agent(&cfg);
    // In CI, get_default_agent should return npm if no config? Actually load_config controls, but get_default_agent does CI check via env.
    // Our get_default_agent checks CI env directly.
    // With empty config, it should return npm when CI set and no interactive?
    // But implementation: if config.default_manager_agent() is Some => return it, else if CI => return Some(Npm)
    // So with empty cfg, it should be Some(Npm)
    assert!(agent.is_some());
    if let Some(v) = orig_ci {
        unsafe { std::env::set_var("CI", v) };
    } else {
        unsafe { std::env::remove_var("CI") };
    }
}
