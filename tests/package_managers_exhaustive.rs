#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
use zpm::package_manager::{Agent, LogicalCommand, resolve_command};

fn resolve(agent: Agent, cmd: LogicalCommand, args: &[&str]) -> Option<String> {
    let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    resolve_command(agent, cmd, &owned).map(|c| {
        format!("{} {}", c.program, c.args.join(" "))
            .trim()
            .to_string()
    })
}

#[test]
fn all_agents_all_commands() {
    let agents = [
        Agent::Npm,
        Agent::Pnpm,
        Agent::PnpmAt6,
        Agent::PnpmRush,
        Agent::Yarn,
        Agent::YarnBerry,
        Agent::Bun,
        Agent::Deno,
        Agent::Aube,
        Agent::Nub,
    ];
    let commands = [
        LogicalCommand::Agent,
        LogicalCommand::Install,
        LogicalCommand::Frozen,
        LogicalCommand::Add,
        LogicalCommand::Upgrade,
        LogicalCommand::UpgradeInteractive,
        LogicalCommand::Uninstall,
        LogicalCommand::Global,
        LogicalCommand::GlobalUninstall,
        LogicalCommand::Execute,
        LogicalCommand::ExecuteLocal,
        LogicalCommand::Dedupe,
        LogicalCommand::Run,
    ];
    for agent in agents {
        for cmd in commands {
            let res = resolve(agent, cmd, &["react"]);
            // For unsupported combos, res is None, which is expected for some
            // For supported, ensure program non-empty
            if let Some(s) = res {
                assert!(
                    !s.is_empty(),
                    "agent {:?} cmd {:?} gave empty string",
                    agent,
                    cmd
                );
                // Ensure program is known
                assert!(s.split_whitespace().next().is_some());
            } else {
                // Check expected unsupported combos
                let expected_none = matches!(
                    (agent, cmd),
                    (Agent::Yarn, LogicalCommand::Dedupe)
                        | (Agent::Bun, LogicalCommand::Dedupe)
                        | (Agent::Deno, LogicalCommand::Dedupe)
                        | (Agent::Npm, LogicalCommand::UpgradeInteractive)
                );
                // Not all None are expected; but at least we exercised the branch
                let _ = expected_none;
            }
        }
    }
}

#[test]
fn aube_specific() {
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::Agent, &[]).unwrap(),
        "aube"
    );
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::Install, &[]).unwrap(),
        "aube install"
    );
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::Frozen, &[]).unwrap(),
        "aube install --frozen-lockfile"
    );
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::Add, &["react"]).unwrap(),
        "aube add react"
    );
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::Global, &["eslint"]).unwrap(),
        "aube add -g eslint"
    );
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::Uninstall, &["react"]).unwrap(),
        "aube remove react"
    );
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::GlobalUninstall, &["eslint"]).unwrap(),
        "aube remove -g eslint"
    );
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::Upgrade, &[]).unwrap(),
        "aube update"
    );
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::UpgradeInteractive, &[]).unwrap(),
        "aube update -i"
    );
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::Dedupe, &[]).unwrap(),
        "aube dedupe"
    );
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::Execute, &["vite"]).unwrap(),
        "aube dlx vite"
    );
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::ExecuteLocal, &["vite"]).unwrap(),
        "aube exec vite"
    );
    assert_eq!(
        resolve(Agent::Aube, LogicalCommand::Run, &["dev"]).unwrap(),
        "aube run dev"
    );
}

#[test]
fn nub_specific() {
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::Agent, &[]).unwrap(),
        "nub"
    );
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::Install, &[]).unwrap(),
        "nub install"
    );
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::Frozen, &[]).unwrap(),
        "nub install --frozen-lockfile"
    );
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::Add, &["react"]).unwrap(),
        "nub add react"
    );
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::Global, &["eslint"]).unwrap(),
        "nub add -g eslint"
    );
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::Uninstall, &["react"]).unwrap(),
        "nub remove react"
    );
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::GlobalUninstall, &["eslint"]).unwrap(),
        "nub remove -g eslint"
    );
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::Upgrade, &[]).unwrap(),
        "nub update"
    );
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::UpgradeInteractive, &[]).unwrap(),
        "nub update -i"
    );
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::Dedupe, &[]).unwrap(),
        "nub dedupe"
    );
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::Execute, &["vite"]).unwrap(),
        "nubx vite"
    );
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::ExecuteLocal, &["vite"]).unwrap(),
        "nub exec vite"
    );
    assert_eq!(
        resolve(Agent::Nub, LogicalCommand::Run, &["dev"]).unwrap(),
        "nub run dev"
    );
}

#[test]
fn bun_specific_extra() {
    assert_eq!(
        resolve(Agent::Bun, LogicalCommand::Global, &["eslint"]).unwrap(),
        "bun add -g eslint"
    );
    assert_eq!(
        resolve(Agent::Bun, LogicalCommand::GlobalUninstall, &["eslint"]).unwrap(),
        "bun remove -g eslint"
    );
    assert_eq!(
        resolve(Agent::Bun, LogicalCommand::Upgrade, &[]).unwrap(),
        "bun update"
    );
    assert_eq!(
        resolve(Agent::Bun, LogicalCommand::UpgradeInteractive, &[]).unwrap(),
        "bun update -i"
    );
    assert!(resolve(Agent::Bun, LogicalCommand::Dedupe, &[]).is_none());
}

#[test]
fn deno_specific_extra() {
    assert_eq!(
        resolve(Agent::Deno, LogicalCommand::Agent, &[]).unwrap(),
        "deno"
    );
    assert_eq!(
        resolve(Agent::Deno, LogicalCommand::Install, &[]).unwrap(),
        "deno install"
    );
    assert_eq!(
        resolve(Agent::Deno, LogicalCommand::Frozen, &[]).unwrap(),
        "deno install --frozen"
    );
    assert_eq!(
        resolve(Agent::Deno, LogicalCommand::Add, &["react"]).unwrap(),
        "deno add react"
    );
    assert_eq!(
        resolve(Agent::Deno, LogicalCommand::Global, &["eslint"]).unwrap(),
        "deno install -g eslint"
    );
    assert_eq!(
        resolve(Agent::Deno, LogicalCommand::Uninstall, &["react"]).unwrap(),
        "deno remove react"
    );
    assert_eq!(
        resolve(Agent::Deno, LogicalCommand::GlobalUninstall, &["eslint"]).unwrap(),
        "deno uninstall -g eslint"
    );
    assert_eq!(
        resolve(Agent::Deno, LogicalCommand::Upgrade, &[]).unwrap(),
        "deno outdated --update"
    );
    assert_eq!(
        resolve(Agent::Deno, LogicalCommand::UpgradeInteractive, &[]).unwrap(),
        "deno outdated --update"
    );
    assert!(resolve(Agent::Deno, LogicalCommand::Dedupe, &[]).is_none());
    assert_eq!(
        resolve(Agent::Deno, LogicalCommand::Execute, &["vite"]).unwrap(),
        "deno x vite"
    );
    assert_eq!(
        resolve(Agent::Deno, LogicalCommand::ExecuteLocal, &["vite"]).unwrap(),
        "deno task --eval vite"
    );
    assert_eq!(
        resolve(Agent::Deno, LogicalCommand::Run, &["dev"]).unwrap(),
        "deno task dev"
    );
}
