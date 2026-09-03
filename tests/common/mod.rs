//! Shared test helpers — isolated temp fixtures, env guards, fake executables.
#![allow(dead_code)]
#![allow(clippy::field_reassign_with_default)]
#![allow(unused)]
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use tempfile::TempDir;

/// Global mutex for tests that mutate process env (serializes those tests).
static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

pub fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Guard that saves original env values and restores on drop.
pub struct EnvGuard {
    original: HashMap<String, Option<String>>,
    _lock: Option<std::sync::MutexGuard<'static, ()>>,
}

impl EnvGuard {
    /// Take the global env lock and snapshot keys.
    pub fn new_with_lock(keys: &[&str]) -> Self {
        let lock = env_lock();
        let mut original = HashMap::new();
        for k in keys {
            original.insert(k.to_string(), std::env::var(k).ok());
        }
        Self {
            original,
            _lock: Some(lock),
        }
    }

    pub fn set(&self, key: &str, val: &str) {
        unsafe { std::env::set_var(key, val) };
    }

    pub fn remove(&self, key: &str) {
        unsafe { std::env::remove_var(key) };
    }

    pub fn set_path(&self, path: &Path) {
        unsafe { std::env::set_var("PATH", path) };
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (k, v) in &self.original {
            match v {
                Some(val) => unsafe { std::env::set_var(k, val) },
                None => unsafe { std::env::remove_var(k) },
            }
        }
        // lock released when _lock dropped
    }
}

/// Simple temp project builder.
pub struct TempProject {
    pub dir: TempDir,
}

impl TempProject {
    pub fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
        }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn join(&self, rel: &str) -> PathBuf {
        self.path().join(rel)
    }

    pub fn mkdir(&self, rel: &str) -> PathBuf {
        let p = self.join(rel);
        fs::create_dir_all(&p).unwrap();
        p
    }

    pub fn mkdir_p(&self, rel: &str) -> PathBuf {
        let p = self.join(rel);
        fs::create_dir_all(&p).unwrap();
        p
    }

    pub fn write(&self, rel: &str, content: &str) -> PathBuf {
        let p = self.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&p, content).unwrap();
        p
    }

    pub fn write_json(&self, rel: &str, json: &str) -> PathBuf {
        self.write(rel, json)
    }

    pub fn touch(&self, rel: &str) -> PathBuf {
        self.write(rel, "")
    }

    pub fn symlink(&self, original: &Path, link: &str) {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(original, self.join(link)).unwrap();
        }
        #[cfg(windows)]
        {
            if original.is_dir() {
                std::os::windows::fs::symlink_dir(original, self.join(link)).unwrap();
            } else {
                std::os::windows::fs::symlink_file(original, self.join(link)).unwrap();
            }
        }
    }

    pub fn create_fake_executable(&self, name: &str, exit_code: i32) -> PathBuf {
        let bin_dir = self.mkdir("bin");
        let exe_path = bin_dir.join(name);
        #[cfg(unix)]
        {
            let script = format!("#!/bin/sh\nexit {}\n", exit_code);
            fs::write(&exe_path, script).unwrap();
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&exe_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&exe_path, perms).unwrap();
        }
        #[cfg(windows)]
        {
            // On Windows, create a .exe via creating a .bat that will be found via `which`?
            // `which` on Windows looks for PATHEXT; we create a simple .exe placeholder by
            // creating a .bat and also a file without extension that contains batch.
            // Simpler: create exe file with echo and exit code using cmd.
            let script = format!("@echo off\nexit /b {}\n", exit_code);
            let bat_path = bin_dir.join(format!("{}.bat", name));
            fs::write(&bat_path, script).unwrap();
            // also create bare file as shim via copying bat
            fs::write(&exe_path, format!("@echo off\nexit /b {}\n", exit_code)).unwrap();
        }
        exe_path
    }

    /// Create a fake executable that logs its args to a file.
    pub fn create_logging_executable(&self, name: &str, log_path: &Path) -> PathBuf {
        let bin_dir = self.mkdir("bin");
        let exe_path = bin_dir.join(name);
        #[cfg(unix)]
        {
            let script = format!(
                "#!/bin/sh\necho \"$@\" > \"{}\"\n# also echo each arg on new line for detailed check\nprintf \"%s\\n\" \"$@\" >> \"{}\"\nexit 0\n",
                log_path.display(),
                log_path.display()
            );
            fs::write(&exe_path, script).unwrap();
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&exe_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&exe_path, perms).unwrap();
        }
        #[cfg(windows)]
        {
            let script = format!(
                "@echo off\necho %* > \"{}\"\nexit /b 0\n",
                log_path.display()
            );
            fs::write(&exe_path, script).unwrap();
            let bat_path = bin_dir.join(format!("{}.bat", name));
            fs::write(&bat_path, script).unwrap();
        }
        exe_path
    }

    pub fn bin_dir(&self) -> PathBuf {
        self.path().join("bin")
    }

    pub fn add_to_path_env(&self, guard: &EnvGuard) {
        let bin = self.bin_dir();
        let current_path = std::env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", bin.display(), current_path);
        guard.set("PATH", &new_path);
        // On Windows separator is ;
        #[cfg(windows)]
        {
            let new_path = format!("{};{}", bin.display(), current_path);
            guard.set("PATH", &new_path);
        }
    }
}

/// Helper to run `cargo run` style but via library resolve.
/// Create a Cli via parsing strings.
use clap::Parser;
use zpm::cli::Cli;

pub fn parse_cli(args: &[&str]) -> Cli {
    Cli::try_parse_from(args).expect("cli parse failed")
}

pub fn parse_cli_fails(args: &[&str]) -> bool {
    Cli::try_parse_from(args).is_err()
}

/// Create a minimal package.json with given content snippets.
pub fn pkg_json_with(workspaces: Option<&str>, scripts: Option<&[(&str, &str)]>) -> String {
    let mut map = serde_json::Map::new();
    if let Some(ws) = workspaces {
        // parse workspaces as json array or string
        if ws.starts_with('[') {
            let v: serde_json::Value = serde_json::from_str(ws).unwrap();
            map.insert("workspaces".to_string(), v);
        } else {
            map.insert(
                "workspaces".to_string(),
                serde_json::Value::Array(vec![serde_json::Value::String(ws.to_string())]),
            );
        }
    }
    if let Some(scripts) = scripts {
        let mut s = serde_json::Map::new();
        for (k, v) in scripts {
            s.insert(k.to_string(), serde_json::Value::String(v.to_string()));
        }
        map.insert("scripts".to_string(), serde_json::Value::Object(s));
    }
    serde_json::Value::Object(map).to_string()
}
