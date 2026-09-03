//! Yarn family command tables (classic and berry).

use super::Agent;
use super::command::{LogicalCommand, dash_dash_arg};

pub(crate) fn resolve(
    agent: Agent,
    command: LogicalCommand,
    args: &[String],
) -> Option<Vec<String>> {
    match (agent, command) {
        // agent identity
        (Agent::Yarn, LogicalCommand::Agent) => Some(vec!["yarn".to_string()]),
        (Agent::YarnBerry, LogicalCommand::Agent) => Some(vec!["yarn".to_string()]),

        // run
        (Agent::Yarn, LogicalCommand::Run) => {
            let mut v = vec!["yarn".to_string(), "run".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        (Agent::YarnBerry, LogicalCommand::Run) => {
            let mut v = vec!["yarn".to_string(), "run".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }

        // install
        (Agent::Yarn, LogicalCommand::Install) => {
            let mut v = vec!["yarn".to_string(), "install".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        (Agent::YarnBerry, LogicalCommand::Install) => {
            let mut v = vec!["yarn".to_string(), "install".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }

        // frozen
        (Agent::Yarn, LogicalCommand::Frozen) => {
            let mut v = vec![
                "yarn".to_string(),
                "install".to_string(),
                "--frozen-lockfile".to_string(),
            ];
            v.extend_from_slice(args);
            Some(v)
        }
        (Agent::YarnBerry, LogicalCommand::Frozen) => {
            let mut v = vec![
                "yarn".to_string(),
                "install".to_string(),
                "--immutable".to_string(),
            ];
            v.extend_from_slice(args);
            Some(v)
        }

        // add
        (Agent::Yarn, LogicalCommand::Add) => {
            let mut v = vec!["yarn".to_string(), "add".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        (Agent::YarnBerry, LogicalCommand::Add) => {
            let mut v = vec!["yarn".to_string(), "add".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }

        // global
        (Agent::Yarn, LogicalCommand::Global) => {
            let mut v = vec!["yarn".to_string(), "global".to_string(), "add".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        (Agent::YarnBerry, LogicalCommand::Global) => {
            // Berry has no global, ni delegates to npm
            let mut v = vec!["npm".to_string(), "i".to_string(), "-g".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }

        // uninstall
        (Agent::Yarn, LogicalCommand::Uninstall) => {
            let mut v = vec!["yarn".to_string(), "remove".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        (Agent::YarnBerry, LogicalCommand::Uninstall) => {
            let mut v = vec!["yarn".to_string(), "remove".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }

        // global uninstall
        (Agent::Yarn, LogicalCommand::GlobalUninstall) => {
            let mut v = vec![
                "yarn".to_string(),
                "global".to_string(),
                "remove".to_string(),
            ];
            v.extend_from_slice(args);
            Some(v)
        }
        (Agent::YarnBerry, LogicalCommand::GlobalUninstall) => {
            let mut v = vec!["npm".to_string(), "uninstall".to_string(), "-g".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }

        // upgrade
        (Agent::Yarn, LogicalCommand::Upgrade) => {
            let mut v = vec!["yarn".to_string(), "upgrade".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        (Agent::YarnBerry, LogicalCommand::Upgrade) => {
            let mut v = vec!["yarn".to_string(), "up".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }

        // upgrade-interactive
        (Agent::Yarn, LogicalCommand::UpgradeInteractive) => {
            let mut v = vec!["yarn".to_string(), "upgrade-interactive".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        (Agent::YarnBerry, LogicalCommand::UpgradeInteractive) => {
            let mut v = vec!["yarn".to_string(), "up".to_string(), "-i".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }

        // dedupe
        (Agent::Yarn, LogicalCommand::Dedupe) => None,
        (Agent::YarnBerry, LogicalCommand::Dedupe) => {
            let mut v = vec!["yarn".to_string(), "dedupe".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }

        // execute
        (Agent::Yarn, LogicalCommand::Execute) => {
            let mut v = vec!["npx".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        (Agent::YarnBerry, LogicalCommand::Execute) => {
            let mut v = vec!["yarn".to_string(), "dlx".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }

        // executeLocal
        (Agent::Yarn, LogicalCommand::ExecuteLocal) => {
            Some(dash_dash_arg("yarn", "exec", &[], args))
        }
        (Agent::YarnBerry, LogicalCommand::ExecuteLocal) => {
            let mut v = vec!["yarn".to_string(), "exec".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }

        _ => None,
    }
}
