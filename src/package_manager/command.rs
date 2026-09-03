//! Structured command representation.
//!
//! `CommandSpec` is the domain-level output of `resolve_command` — a
//! program plus an argument vector, with an optional working directory.
//! Keeping the command structured (vs. a shell string) preserves argument
//! boundaries and avoids quoting issues.

/// Concrete, executable command produced by the resolver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<std::path::PathBuf>,
}

impl CommandSpec {
    pub fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
            cwd: None,
        }
    }

    /// Human-readable rendering, quoting arguments that contain spaces.
    pub fn to_string_pretty(&self) -> String {
        let mut parts = vec![self.program.clone()];
        parts.extend(self.args.clone());
        parts
            .into_iter()
            .map(|a| {
                if a.contains(' ') {
                    format!("\"{a}\"")
                } else {
                    a
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Logical intent that must be translated to a per-manager command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalCommand {
    Agent,
    Install,
    Frozen,
    Add,
    Upgrade,
    UpgradeInteractive,
    Uninstall,
    Global,
    GlobalUninstall,
    Execute,
    ExecuteLocal,
    Dedupe,
    Run,
}

// ---------------------------------------------------------------------------
// Shared helpers for `run` dash handling (ported from original mod.rs)
// ---------------------------------------------------------------------------

/// Split `args` into `(before_script, script, after_script)`.
///
/// The split respects value flags (e.g. `-w packages/foo`) so that the first
/// non-flag that is *not* a flag value is treated as the script name.
pub(crate) fn split_run_args(
    args: &[String],
    value_flags: &[&str],
) -> (Vec<String>, Option<String>, Vec<String>) {
    for i in 0..args.len() {
        if args[i].starts_with('-') {
            continue;
        }
        if i > 0 && value_flags.contains(&args[i - 1].as_str()) {
            continue;
        }
        let before = args[..i].to_vec();
        let script = args[i].clone();
        let after = args[i + 1..].to_vec();
        return (before, Some(script), after);
    }
    (args.to_vec(), None, vec![])
}

/// Build the `agent agent_command` invocation, inserting `--` between the
/// script and its forwarded arguments when needed (npm semantics).
pub(crate) fn dash_dash_arg(
    agent: &str,
    agent_command: &str,
    value_flags: &[&str],
    args: &[String],
) -> Vec<String> {
    let (before, script, after) = split_run_args(args, value_flags);
    match script {
        None => {
            let mut v = vec![agent.to_string(), agent_command.to_string()];
            v.extend(before);
            v
        }
        Some(s) => {
            if !after.is_empty() {
                let mut v = vec![agent.to_string(), agent_command.to_string()];
                v.extend(before);
                v.push(s);
                v.push("--".to_string());
                v.extend(after);
                v
            } else {
                let mut v = vec![agent.to_string(), agent_command.to_string()];
                v.extend(before);
                v.push(s);
                v
            }
        }
    }
}
