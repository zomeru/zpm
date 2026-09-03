//! Aube command table.

use super::command::LogicalCommand;

pub(crate) fn resolve(command: LogicalCommand, args: &[String]) -> Option<Vec<String>> {
    match command {
        LogicalCommand::Agent => Some(vec!["aube".to_string()]),
        LogicalCommand::Run => {
            let mut v = vec!["aube".to_string(), "run".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Install => {
            let mut v = vec!["aube".to_string(), "install".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Frozen => {
            let mut v = vec![
                "aube".to_string(),
                "install".to_string(),
                "--frozen-lockfile".to_string(),
            ];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Add => {
            let mut v = vec!["aube".to_string(), "add".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Global => {
            let mut v = vec!["aube".to_string(), "add".to_string(), "-g".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Uninstall => {
            let mut v = vec!["aube".to_string(), "remove".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::GlobalUninstall => {
            let mut v = vec!["aube".to_string(), "remove".to_string(), "-g".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Upgrade => {
            let mut v = vec!["aube".to_string(), "update".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::UpgradeInteractive => {
            let mut v = vec!["aube".to_string(), "update".to_string(), "-i".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Dedupe => {
            let mut v = vec!["aube".to_string(), "dedupe".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Execute => {
            let mut v = vec!["aube".to_string(), "dlx".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::ExecuteLocal => {
            let mut v = vec!["aube".to_string(), "exec".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
    }
}
