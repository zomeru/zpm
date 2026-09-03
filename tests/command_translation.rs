#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
use zpm::package_manager::{Agent, LogicalCommand, resolve_command};

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
fn install_empty() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::Install, &[]).as_deref(),
        Some("npm i")
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::Install, &[]).as_deref(),
        Some("pnpm i")
    );
    assert_eq!(
        cmd(Agent::Yarn, LogicalCommand::Install, &[]).as_deref(),
        Some("yarn install")
    );
    assert_eq!(
        cmd(Agent::YarnBerry, LogicalCommand::Install, &[]).as_deref(),
        Some("yarn install")
    );
    assert_eq!(
        cmd(Agent::Bun, LogicalCommand::Install, &[]).as_deref(),
        Some("bun install")
    );
    assert_eq!(
        cmd(Agent::Deno, LogicalCommand::Install, &[]).as_deref(),
        Some("deno install")
    );
    assert_eq!(
        cmd(Agent::Aube, LogicalCommand::Install, &[]).as_deref(),
        Some("aube install")
    );
    assert_eq!(
        cmd(Agent::Nub, LogicalCommand::Install, &[]).as_deref(),
        Some("nub install")
    );
    assert_eq!(
        cmd(Agent::PnpmAt6, LogicalCommand::Install, &[]).as_deref(),
        Some("pnpm i")
    );
    assert_eq!(
        cmd(Agent::PnpmRush, LogicalCommand::Install, &[]).as_deref(),
        Some("rush-pnpm i")
    );
}

#[test]
fn frozen() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::Frozen, &[]).as_deref(),
        Some("npm ci")
    );
    assert_eq!(
        cmd(Agent::Yarn, LogicalCommand::Frozen, &[]).as_deref(),
        Some("yarn install --frozen-lockfile")
    );
    assert_eq!(
        cmd(Agent::YarnBerry, LogicalCommand::Frozen, &[]).as_deref(),
        Some("yarn install --immutable")
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::Frozen, &[]).as_deref(),
        Some("pnpm i --frozen-lockfile")
    );
    assert_eq!(
        cmd(Agent::Bun, LogicalCommand::Frozen, &[]).as_deref(),
        Some("bun install --frozen-lockfile")
    );
    assert_eq!(
        cmd(Agent::Deno, LogicalCommand::Frozen, &[]).as_deref(),
        Some("deno install --frozen")
    );
    assert_eq!(
        cmd(Agent::Aube, LogicalCommand::Frozen, &[]).as_deref(),
        Some("aube install --frozen-lockfile")
    );
    assert_eq!(
        cmd(Agent::Nub, LogicalCommand::Frozen, &[]).as_deref(),
        Some("nub install --frozen-lockfile")
    );
}

#[test]
fn add_single() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::Add, &["axios"]).as_deref(),
        Some("npm i axios")
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::Add, &["axios"]).as_deref(),
        Some("pnpm add axios")
    );
    assert_eq!(
        cmd(Agent::Yarn, LogicalCommand::Add, &["axios"]).as_deref(),
        Some("yarn add axios")
    );
    assert_eq!(
        cmd(Agent::Bun, LogicalCommand::Add, &["axios"]).as_deref(),
        Some("bun add axios")
    );
    assert_eq!(
        cmd(Agent::Deno, LogicalCommand::Add, &["axios"]).as_deref(),
        Some("deno add axios")
    );
    assert_eq!(
        cmd(Agent::Aube, LogicalCommand::Add, &["axios"]).as_deref(),
        Some("aube add axios")
    );
}

#[test]
fn global_add() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::Global, &["eslint"]).as_deref(),
        Some("npm i -g eslint")
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::Global, &["eslint"]).as_deref(),
        Some("pnpm add -g eslint")
    );
    assert_eq!(
        cmd(Agent::Yarn, LogicalCommand::Global, &["eslint"]).as_deref(),
        Some("yarn global add eslint")
    );
    assert_eq!(
        cmd(Agent::YarnBerry, LogicalCommand::Global, &["eslint"]).as_deref(),
        Some("npm i -g eslint")
    );
    assert_eq!(
        cmd(Agent::Bun, LogicalCommand::Global, &["eslint"]).as_deref(),
        Some("bun add -g eslint")
    );
    assert_eq!(
        cmd(Agent::Deno, LogicalCommand::Global, &["eslint"]).as_deref(),
        Some("deno install -g eslint")
    );
}

#[test]
fn run_simple() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::Run, &["dev"]).as_deref(),
        Some("npm run dev")
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::Run, &["dev"]).as_deref(),
        Some("pnpm run dev")
    );
    assert_eq!(
        cmd(Agent::Yarn, LogicalCommand::Run, &["dev"]).as_deref(),
        Some("yarn run dev")
    );
    assert_eq!(
        cmd(Agent::Bun, LogicalCommand::Run, &["dev"]).as_deref(),
        Some("bun run dev")
    );
    assert_eq!(
        cmd(Agent::Deno, LogicalCommand::Run, &["dev"]).as_deref(),
        Some("deno task dev")
    );
    assert_eq!(
        cmd(Agent::Aube, LogicalCommand::Run, &["dev"]).as_deref(),
        Some("aube run dev")
    );
}

#[test]
fn run_with_args_npm_inserts_dashdash() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::Run, &["build", "--watch", "-o"]).as_deref(),
        Some("npm run build -- --watch -o")
    );
    assert_eq!(
        cmd(
            Agent::Pnpm,
            LogicalCommand::Run,
            &["build", "--watch", "-o"]
        )
        .as_deref(),
        Some("pnpm run build --watch -o")
    );
    assert_eq!(
        cmd(
            Agent::Yarn,
            LogicalCommand::Run,
            &["build", "--watch", "-o"]
        )
        .as_deref(),
        Some("yarn run build --watch -o")
    );
    assert_eq!(
        cmd(Agent::Bun, LogicalCommand::Run, &["build", "--watch", "-o"]).as_deref(),
        Some("bun run build --watch -o")
    );
    assert_eq!(
        cmd(
            Agent::Deno,
            LogicalCommand::Run,
            &["build", "--watch", "-o"]
        )
        .as_deref(),
        Some("deno task build --watch -o")
    );
}

#[test]
fn execute() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::Execute, &["esbuild"]).as_deref(),
        Some("npx esbuild")
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::Execute, &["esbuild"]).as_deref(),
        Some("pnpm dlx esbuild")
    );
    assert_eq!(
        cmd(Agent::Yarn, LogicalCommand::Execute, &["esbuild"]).as_deref(),
        Some("npx esbuild")
    );
    assert_eq!(
        cmd(Agent::YarnBerry, LogicalCommand::Execute, &["esbuild"]).as_deref(),
        Some("yarn dlx esbuild")
    );
    assert_eq!(
        cmd(Agent::Bun, LogicalCommand::Execute, &["esbuild"]).as_deref(),
        Some("bun x esbuild")
    );
    assert_eq!(
        cmd(Agent::Deno, LogicalCommand::Execute, &["esbuild"]).as_deref(),
        Some("deno x esbuild")
    );
    assert_eq!(
        cmd(Agent::Aube, LogicalCommand::Execute, &["esbuild"]).as_deref(),
        Some("aube dlx esbuild")
    );
    assert_eq!(
        cmd(Agent::Nub, LogicalCommand::Execute, &["esbuild"]).as_deref(),
        Some("nubx esbuild")
    );
}

#[test]
fn execute_local() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::ExecuteLocal, &["esbuild"]).as_deref(),
        Some("npx esbuild")
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::ExecuteLocal, &["esbuild"]).as_deref(),
        Some("pnpm exec esbuild")
    );
    assert_eq!(
        cmd(Agent::YarnBerry, LogicalCommand::ExecuteLocal, &["esbuild"]).as_deref(),
        Some("yarn exec esbuild")
    );
    assert_eq!(
        cmd(Agent::Bun, LogicalCommand::ExecuteLocal, &["esbuild"]).as_deref(),
        Some("bun x esbuild")
    );
    assert_eq!(
        cmd(Agent::Deno, LogicalCommand::ExecuteLocal, &["esbuild"]).as_deref(),
        Some("deno task --eval esbuild")
    );
    // yarn classic inserts --
    assert_eq!(
        cmd(
            Agent::Yarn,
            LogicalCommand::ExecuteLocal,
            &["esbuild", "--version"]
        )
        .as_deref(),
        Some("yarn exec esbuild -- --version")
    );
}

#[test]
fn uninstall() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::Uninstall, &["axios"]).as_deref(),
        Some("npm uninstall axios")
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::Uninstall, &["axios"]).as_deref(),
        Some("pnpm remove axios")
    );
    assert_eq!(
        cmd(Agent::Yarn, LogicalCommand::Uninstall, &["axios"]).as_deref(),
        Some("yarn remove axios")
    );
    assert_eq!(
        cmd(Agent::Bun, LogicalCommand::Uninstall, &["axios"]).as_deref(),
        Some("bun remove axios")
    );
    assert_eq!(
        cmd(Agent::Deno, LogicalCommand::Uninstall, &["axios"]).as_deref(),
        Some("deno remove axios")
    );
}

#[test]
fn global_uninstall() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::GlobalUninstall, &["eslint"]).as_deref(),
        Some("npm uninstall -g eslint")
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::GlobalUninstall, &["eslint"]).as_deref(),
        Some("pnpm remove --global eslint")
    );
    assert_eq!(
        cmd(Agent::Yarn, LogicalCommand::GlobalUninstall, &["eslint"]).as_deref(),
        Some("yarn global remove eslint")
    );
    assert_eq!(
        cmd(
            Agent::YarnBerry,
            LogicalCommand::GlobalUninstall,
            &["eslint"]
        )
        .as_deref(),
        Some("npm uninstall -g eslint")
    );
}

#[test]
fn upgrade() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::Upgrade, &[]).as_deref(),
        Some("npm update")
    );
    assert_eq!(
        cmd(Agent::Yarn, LogicalCommand::Upgrade, &[]).as_deref(),
        Some("yarn upgrade")
    );
    assert_eq!(
        cmd(Agent::YarnBerry, LogicalCommand::Upgrade, &[]).as_deref(),
        Some("yarn up")
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::Upgrade, &[]).as_deref(),
        Some("pnpm update")
    );
    assert_eq!(
        cmd(Agent::Bun, LogicalCommand::Upgrade, &[]).as_deref(),
        Some("bun update")
    );
    assert_eq!(
        cmd(Agent::Deno, LogicalCommand::Upgrade, &[]).as_deref(),
        Some("deno outdated --update")
    );
}

#[test]
fn upgrade_interactive() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::UpgradeInteractive, &[]),
        None
    );
    assert_eq!(
        cmd(Agent::Yarn, LogicalCommand::UpgradeInteractive, &[]).as_deref(),
        Some("yarn upgrade-interactive")
    );
    assert_eq!(
        cmd(Agent::YarnBerry, LogicalCommand::UpgradeInteractive, &[]).as_deref(),
        Some("yarn up -i")
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::UpgradeInteractive, &[]).as_deref(),
        Some("pnpm update -i")
    );
    assert_eq!(
        cmd(Agent::Bun, LogicalCommand::UpgradeInteractive, &[]).as_deref(),
        Some("bun update -i")
    );
}

#[test]
fn dedupe() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::Dedupe, &[]).as_deref(),
        Some("npm dedupe")
    );
    assert_eq!(cmd(Agent::Yarn, LogicalCommand::Dedupe, &[]), None);
    assert_eq!(
        cmd(Agent::YarnBerry, LogicalCommand::Dedupe, &[]).as_deref(),
        Some("yarn dedupe")
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::Dedupe, &[]).as_deref(),
        Some("pnpm dedupe")
    );
    assert_eq!(cmd(Agent::Bun, LogicalCommand::Dedupe, &[]), None);
    assert_eq!(cmd(Agent::Deno, LogicalCommand::Dedupe, &[]), None);
    assert_eq!(
        cmd(Agent::Aube, LogicalCommand::Dedupe, &[]).as_deref(),
        Some("aube dedupe")
    );
}

#[test]
fn unsupported_returns_none() {
    // bun dedupe unsupported
    assert!(cmd(Agent::Bun, LogicalCommand::Dedupe, &[]).is_none());
    assert!(cmd(Agent::Npm, LogicalCommand::UpgradeInteractive, &[]).is_none());
    assert!(cmd(Agent::Yarn, LogicalCommand::Dedupe, &[]).is_none());
}

#[test]
fn command_spec_quotes_spaces() {
    let spec = zpm::package_manager::CommandSpec::new(
        "pnpm",
        vec!["add".to_string(), "pkg with space".to_string()],
    );
    assert_eq!(spec.to_string_pretty(), "pnpm add \"pkg with space\"");
}
