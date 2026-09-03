//! npm command table.

use super::command::{LogicalCommand, dash_dash_arg};

pub(crate) fn resolve(command: LogicalCommand, args: &[String]) -> Option<Vec<String>> {
    match command {
        LogicalCommand::Agent => Some(vec!["npm".to_string()]),
        LogicalCommand::Run => Some(dash_dash_arg("npm", "run", &["-w", "--workspace"], args)),
        LogicalCommand::Install => {
            let mut v = vec!["npm".to_string(), "i".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Frozen => {
            let mut v = vec!["npm".to_string(), "ci".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Add => {
            let mut v = vec!["npm".to_string(), "i".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Global => {
            let mut v = vec!["npm".to_string(), "i".to_string(), "-g".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Uninstall => {
            let mut v = vec!["npm".to_string(), "uninstall".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::GlobalUninstall => {
            let mut v = vec!["npm".to_string(), "uninstall".to_string(), "-g".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Upgrade => {
            let mut v = vec!["npm".to_string(), "update".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::UpgradeInteractive => None,
        LogicalCommand::Dedupe => {
            let mut v = vec!["npm".to_string(), "dedupe".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::Execute => {
            let mut v = vec!["npx".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
        LogicalCommand::ExecuteLocal => {
            let mut v = vec!["npx".to_string()];
            v.extend_from_slice(args);
            Some(v)
        }
    }
}
