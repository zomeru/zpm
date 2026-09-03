//! Deno command table.

use super::command::LogicalCommand;

pub(crate) fn resolve(command: LogicalCommand, args: &[String]) -> Option<Vec<String>> {
    match command {
        LogicalCommand::Agent => Some(vec!["deno".to_string()]),
        LogicalCommand::Run => {
            let mut v = vec!["deno".to_string(), "task".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Install => {
            let mut v = vec!["deno".to_string(), "install".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Frozen => {
            let mut v = vec![
                "deno".to_string(),
                "install".to_string(),
                "--frozen".to_string(),
            ];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Add => {
            let mut v = vec!["deno".to_string(), "add".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Global => {
            let mut v = vec!["deno".to_string(), "install".to_string(), "-g".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Uninstall => {
            let mut v = vec!["deno".to_string(), "remove".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::GlobalUninstall => {
            let mut v = vec![
                "deno".to_string(),
                "uninstall".to_string(),
                "-g".to_string(),
            ];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Upgrade => {
            let mut v = vec![
                "deno".to_string(),
                "outdated".to_string(),
                "--update".to_string(),
            ];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::UpgradeInteractive => {
            let mut v = vec![
                "deno".to_string(),
                "outdated".to_string(),
                "--update".to_string(),
            ];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Dedupe => None,
        LogicalCommand::Execute => {
            let mut v = vec!["deno".to_string(), "x".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::ExecuteLocal => {
            let mut v = vec!["deno".to_string(), "task".to_string(), "--eval".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
    }
}
