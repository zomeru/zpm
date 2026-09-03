//! Process execution — the smallest leaf in the dependency graph.
//!
//! Responsibilities:
//! * take a resolved `CommandSpec` + working directory
//! * optionally preview (verbose / dry-run) without side effects
//! * spawn the process with inherited stdio and return the exit code
//! * never contain package-manager detection, CLI parsing, or catalog logic

use std::path::Path;
use std::process::Command;

use crate::package_manager::CommandSpec;

/// Execute `spec` in `cwd` and return the process exit code.
///
/// When `dry_run` is true the command is printed and not executed. When
/// `verbose` is true the command and working directory are echoed to stderr
/// before execution.
pub fn execute_command(
    spec: &CommandSpec,
    cwd: &Path,
    dry_run: bool,
    verbose: bool,
) -> anyhow::Result<i32> {
    if verbose || dry_run {
        eprintln!("Detected command: {} {}", spec.program, spec.args.join(" "));
        if let Some(cwd) = &spec.cwd {
            eprintln!("Working dir: {}", cwd.display());
        } else {
            eprintln!("Working dir: {}", cwd.display());
        }
    }
    if dry_run {
        println!("Would run: {} {}", spec.program, spec.args.join(" "));
        return Ok(0);
    }

    let actual_cwd = spec.cwd.as_deref().unwrap_or(cwd);

    if which::which(&spec.program).is_err() {
        eprintln!(
            "× {} was detected but is not installed or not in PATH.\n  Install {} or run with --pm <manager>",
            spec.program, spec.program
        );
        return Ok(127);
    }

    let mut cmd = Command::new(&spec.program);
    cmd.args(&spec.args);
    cmd.current_dir(actual_cwd);
    cmd.stdin(std::process::Stdio::inherit());
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());

    let status = cmd.status()?;
    Ok(status.code().unwrap_or(1))
}

/// Whether `program` is available on `PATH` (via `which`).
pub fn check_command_exists(program: &str) -> bool {
    which::which(program).is_ok()
}
