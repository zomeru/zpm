//! Package-manager abstraction.
//!
//! Centralizes all manager-specific command translation so that adding a new
//! manager requires touching only the files under `package_manager/`:
//!
//! * [`agent`] — identity, parsing, display
//! * [`command`] — `CommandSpec` / `LogicalCommand` and shared helpers
//! * `npm`, `pnpm`, `yarn`, … — per-manager command tables
//!
//! The public entry point is [`resolve_command`], which returns a structured
//! [`command::CommandSpec`] instead of a shell string.

pub mod agent;
pub mod command;

mod aube;
mod bun;
mod deno;
mod npm;
mod nub;
mod pnpm;
mod yarn;

pub use agent::Agent;
pub use command::{CommandSpec, LogicalCommand};

/// Resolve a logical command for `agent` into a concrete `CommandSpec`.
///
/// Returns `None` when the manager does not support the command (e.g.
/// `bun dedupe`).
pub fn resolve_command(
    agent: Agent,
    command: LogicalCommand,
    args: &[String],
) -> Option<CommandSpec> {
    let raw: Option<Vec<String>> = match agent {
        Agent::Npm => npm::resolve(command, args),
        Agent::Pnpm | Agent::PnpmAt6 | Agent::PnpmRush => pnpm::resolve(agent, command, args),
        Agent::Yarn | Agent::YarnBerry => yarn::resolve(agent, command, args),
        Agent::Bun => bun::resolve(command, args),
        Agent::Deno => deno::resolve(command, args),
        Agent::Aube => aube::resolve(command, args),
        Agent::Nub => nub::resolve(command, args),
    };

    raw.map(|v| {
        if v.is_empty() {
            CommandSpec::new("", vec![])
        } else {
            let program = v[0].clone();
            let args = v[1..].to_vec();
            CommandSpec::new(program, args)
        }
    })
}

/// Legacy helper retained for backwards compatibility — wraps an optional
/// `Vec<String>` where the first element is the program.
pub fn construct_command(value: Option<Vec<String>>) -> Option<CommandSpec> {
    value.map(|v| {
        let program = v[0].clone();
        let args = v[1..].to_vec();
        CommandSpec::new(program, args)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(agent: Agent, command: LogicalCommand, args: &[&str]) -> Option<String> {
        let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        resolve_command(agent, command, &args_owned).map(|c| {
            if c.args.is_empty() {
                c.program
            } else {
                format!("{} {}", c.program, c.args.join(" "))
            }
        })
    }

    #[test]
    fn npm_run_with_args_uses_double_dash() {
        let c = cmd(Agent::Npm, LogicalCommand::Run, &["dev", "--port", "3000"]);
        assert_eq!(c.unwrap(), "npm run dev -- --port 3000");
    }

    #[test]
    fn yarn_run_no_double_dash() {
        let c = cmd(Agent::Yarn, LogicalCommand::Run, &["dev", "--port", "3000"]);
        assert_eq!(c.unwrap(), "yarn run dev --port 3000");
    }
}
