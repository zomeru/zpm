//! pnpm family command tables (pnpm, pnpm@6, pnpm-rush).

use super::Agent;
use super::command::{LogicalCommand, dash_dash_arg};

fn program(agent: Agent) -> &'static str {
    match agent {
        Agent::PnpmRush => "rush-pnpm",
        _ => "pnpm",
    }
}

pub(crate) fn resolve(
    agent: Agent,
    command: LogicalCommand,
    args: &[String],
) -> Option<Vec<String>> {
    match command {
        LogicalCommand::Agent => Some(vec![program(agent).to_string()]),
        LogicalCommand::Run => {
            if agent == Agent::PnpmAt6 {
                Some(dash_dash_arg("pnpm", "run", &["-F", "--filter"], args))
            } else {
                let mut v = vec![program(agent).to_string(), "run".to_string()];
                v.extend_from_slice(args);
                Some(v)
            }
        }
        LogicalCommand::Install => {
            let mut v = vec![program(agent).to_string(), "i".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Frozen => {
            let mut v = vec![
                program(agent).to_string(),
                "i".to_string(),
                "--frozen-lockfile".to_string(),
            ];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Add => {
            let mut v = vec![program(agent).to_string(), "add".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Global => {
            let mut v = vec![
                program(agent).to_string(),
                "add".to_string(),
                "-g".to_string(),
            ];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Uninstall => {
            let mut v = vec![program(agent).to_string(), "remove".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::GlobalUninstall => {
            let mut v = vec![
                program(agent).to_string(),
                "remove".to_string(),
                "--global".to_string(),
            ];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Upgrade => {
            let mut v = vec![program(agent).to_string(), "update".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::UpgradeInteractive => {
            let mut v = vec![
                program(agent).to_string(),
                "update".to_string(),
                "-i".to_string(),
            ];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Dedupe => {
            let mut v = vec![program(agent).to_string(), "dedupe".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Execute => {
            let mut v = vec![program(agent).to_string(), "dlx".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::ExecuteLocal => {
            let mut v = vec![program(agent).to_string(), "exec".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
    }
}
