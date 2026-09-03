//! Application orchestration — bridges CLI intent and process execution.
//!
//! `main.rs` remains small and delegates here so the whole flow is testable
//! via the library crate (no process spawning required for dry-run paths).

use std::path::{Path, PathBuf};

use crate::{
    cli::{Cli, resolve_cli},
    config::Config,
    package_manager::CommandSpec,
    process, ui,
};

/// Result of the app pipeline: `None` means nothing to execute (e.g. `zpm agent`
/// already printed or interactive cancellation), otherwise a spec to run.
pub fn resolve_spec(cli: &Cli, config: &Config, cwd: &Path) -> anyhow::Result<Option<CommandSpec>> {
    resolve_cli(cli, config, cwd)
}

/// Execute the resolved spec, handling `dry-run`, `verbose`, and auto-install
/// prompting. Returns the process exit code.
pub fn execute(cli: &Cli, config: &Config, cwd: &Path, spec: CommandSpec) -> anyhow::Result<i32> {
    let dry_run = cli.dry_run
        || std::env::var("ZPM_DRY_RUN")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
    let verbose = cli.verbose
        || std::env::var("ZPM_VERBOSE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

    if verbose {
        eprintln!("{} {}", ui::dim("Detected:"), spec.program);
        eprintln!("{} {}", ui::dim("Command:"), spec.to_string_pretty());
    } else if dry_run {
        println!("Detected package manager: {}", spec.program);
        println!("Command: {}", spec.to_string_pretty());
        return Ok(0);
    }

    if dry_run {
        println!("Would run: {}", spec.to_string_pretty());
        return Ok(0);
    }

    if spec.program.is_empty() {
        eprintln!("No command resolved");
        return Ok(1);
    }

    // Auto-install handling
    let auto_install = config.auto_install.unwrap_or(false)
        || std::env::var("ZPM_AUTO_INSTALL")
            .map(|v| v == "true")
            .unwrap_or(false);

    let cwd_for_exec = spec.cwd.as_deref().unwrap_or(cwd);

    if !process::check_command_exists(&spec.program) {
        if auto_install {
            eprintln!("Auto-installing {}...", spec.program);
            let install_spec = CommandSpec::new(
                "npm",
                vec!["i".to_string(), "-g".to_string(), spec.program.clone()],
            );
            let code = process::execute_command(&install_spec, cwd_for_exec, false, verbose)?;
            if code != 0 {
                return Ok(code);
            }
        } else {
            if std::env::var("CI").is_ok() {
                eprintln!(
                    "{} {} is not installed and CI is set. Failing.",
                    ui::error("×"),
                    spec.program
                );
                return Ok(1);
            }
            if console::user_attended() && !cli.no_interactive {
                if ui::confirm_auto_install(&spec.program, None) {
                    eprintln!("Installing {} via npm...", spec.program);
                    let install_spec = CommandSpec::new(
                        "npm",
                        vec!["i".to_string(), "-g".to_string(), spec.program.clone()],
                    );
                    let code =
                        process::execute_command(&install_spec, cwd_for_exec, false, verbose)?;
                    if code != 0 {
                        return Ok(code);
                    }
                } else {
                    eprintln!("Aborted: {} is not installed.", spec.program);
                    return Ok(1);
                }
            } else {
                eprintln!(
                    "{} {} is not installed. Install it or run with --pm <manager>",
                    ui::error("×"),
                    spec.program
                );
                return Ok(1);
            }
        }
    }

    let cwd_for_exec = spec.cwd.as_deref().unwrap_or(cwd);
    let code = process::execute_command(&spec, cwd_for_exec, false, verbose)?;
    Ok(code)
}

/// High-level entry point used by `main.rs`: parse already done, just run.
pub fn run(cli: Cli, config: Config, cwd: PathBuf) -> anyhow::Result<i32> {
    // Handle color / interactivity env side-effects early (kept here for
    // testability — callers can pre-set env before `run`).
    if let Some(color) = &cli.color {
        match color.to_lowercase().as_str() {
            // SAFETY: single-threaded at startup before any threads are spawned
            "never" | "false" => unsafe { std::env::set_var("NO_COLOR", "1") },
            "always" | "true" => unsafe { std::env::remove_var("NO_COLOR") },
            _ => {}
        }
    }
    if cli.no_interactive {
        // SAFETY: single-threaded at startup
        unsafe { std::env::set_var("ZPM_NO_INTERACTIVE", "true") };
    }

    let spec_opt = resolve_spec(&cli, &config, &cwd)?;
    let spec = match spec_opt {
        Some(s) => s,
        None => return Ok(0),
    };
    execute(&cli, &config, &cwd, spec)
}
