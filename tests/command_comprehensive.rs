#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
use zpm::package_manager::{Agent, CommandSpec, LogicalCommand, resolve_command};

fn cmd(agent: Agent, command: LogicalCommand, args: &[&str]) -> Option<CommandSpec> {
    let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    resolve_command(agent, command, &owned)
}

#[test]
fn command_spec_new_and_pretty_without_spaces() {
    let spec = CommandSpec::new("npm", vec!["install".to_string(), "--save".to_string()]);
    assert_eq!(spec.program, "npm");
    assert_eq!(spec.args, vec!["install", "--save"]);
    assert!(spec.cwd.is_none());
    assert_eq!(spec.to_string_pretty(), "npm install --save");
}

#[test]
fn command_spec_pretty_quotes_spaces() {
    let spec = CommandSpec::new(
        "pnpm",
        vec!["add".to_string(), "pkg with space".to_string()],
    );
    assert_eq!(spec.to_string_pretty(), "pnpm add \"pkg with space\"");
}

#[test]
fn command_spec_pretty_multiple_spaces() {
    let spec = CommandSpec::new(
        "yarn",
        vec![
            "run".to_string(),
            "hello world".to_string(),
            "foo bar".to_string(),
        ],
    );
    assert_eq!(
        spec.to_string_pretty(),
        "yarn run \"hello world\" \"foo bar\""
    );
}

#[test]
fn resolve_agent_identity() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::Agent, &[]).unwrap().program,
        "npm"
    );
    assert_eq!(
        cmd(Agent::Pnpm, LogicalCommand::Agent, &[])
            .unwrap()
            .program,
        "pnpm"
    );
    assert_eq!(
        cmd(Agent::PnpmRush, LogicalCommand::Agent, &[])
            .unwrap()
            .program,
        "rush-pnpm"
    );
    assert_eq!(
        cmd(Agent::Yarn, LogicalCommand::Agent, &[])
            .unwrap()
            .program,
        "yarn"
    );
    assert_eq!(
        cmd(Agent::YarnBerry, LogicalCommand::Agent, &[])
            .unwrap()
            .program,
        "yarn"
    );
    assert_eq!(
        cmd(Agent::Bun, LogicalCommand::Agent, &[]).unwrap().program,
        "bun"
    );
}

#[test]
fn construct_command_helper() {
    let spec = zpm::package_manager::construct_command(Some(vec![
        "npm".to_string(),
        "i".to_string(),
        "foo".to_string(),
    ]))
    .unwrap();
    assert_eq!(spec.program, "npm");
    assert_eq!(spec.args, vec!["i", "foo"]);
    assert!(zpm::package_manager::construct_command(None).is_none());
}

#[test]
fn all_managers_install_forms() {
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
    for agent in agents {
        let spec = cmd(agent, LogicalCommand::Install, &[]).expect("install should exist");
        assert!(!spec.program.is_empty());
    }
}

#[test]
fn all_managers_frozen_forms() {
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
    for agent in agents {
        let spec = cmd(agent, LogicalCommand::Frozen, &[]).expect("frozen should exist");
        assert!(!spec.program.is_empty());
        if agent == Agent::Npm {
            assert_eq!(spec.args, vec!["ci"]);
        }
        if agent == Agent::Yarn {
            assert_eq!(spec.args, vec!["install", "--frozen-lockfile"]);
        }
        if agent == Agent::YarnBerry {
            assert_eq!(spec.args, vec!["install", "--immutable"]);
        }
    }
}

#[test]
fn pnpm_variants_run_distinction() {
    // pnpm@6 should use dash_dash_arg for run, others not
    let spec_pnpm = cmd(Agent::Pnpm, LogicalCommand::Run, &["dev", "--port", "3000"]).unwrap();
    // standard pnpm run does NOT insert --
    assert_eq!(spec_pnpm.args, vec!["run", "dev", "--port", "3000"]);
    let spec_pnpm6 = cmd(
        Agent::PnpmAt6,
        LogicalCommand::Run,
        &["dev", "--port", "3000"],
    )
    .unwrap();
    // pnpm@6 uses dash handling which will insert --? Let's check: dash_dash_arg inserts -- between script and after
    // For pnpm@6, it should go through dash_dash_arg which inserts --
    // So "dev" script + after ["--port","3000"] -> ["pnpm","run","dev","--","--port","3000"]
    // But spec includes program "pnpm", args = ["run","dev","--","--port","3000"]
    assert_eq!(spec_pnpm6.args, vec!["run", "dev", "--", "--port", "3000"]);
}

#[test]
fn yarn_run_no_dashdash_even_with_args() {
    let spec = cmd(Agent::Yarn, LogicalCommand::Run, &["build", "--watch"]).unwrap();
    assert_eq!(spec.args, vec!["run", "build", "--watch"]);
    let spec_berry = cmd(Agent::YarnBerry, LogicalCommand::Run, &["build", "--watch"]).unwrap();
    assert_eq!(spec_berry.args, vec!["run", "build", "--watch"]);
}

#[test]
fn npm_run_inserts_dashdash_only_when_after_non_empty() {
    let spec_no_after = cmd(Agent::Npm, LogicalCommand::Run, &["dev"]).unwrap();
    assert_eq!(spec_no_after.args, vec!["run", "dev"]);
    let spec_with = cmd(Agent::Npm, LogicalCommand::Run, &["dev", "--port", "3000"]).unwrap();
    assert_eq!(spec_with.args, vec!["run", "dev", "--", "--port", "3000"]);
    // with value flag -w handling?
    let spec_with_w = cmd(
        Agent::Npm,
        LogicalCommand::Run,
        &["-w", "pkg", "dev", "--flag"],
    )
    .unwrap();
    // dash_dash_arg respects value flags: -w pkg is before, dev is script, --flag after -> inserts --
    assert!(spec_with_w.args.contains(&"--".to_string()));
}

#[test]
fn dedupe_unsupported() {
    assert!(cmd(Agent::Yarn, LogicalCommand::Dedupe, &[]).is_none());
    assert!(cmd(Agent::Bun, LogicalCommand::Dedupe, &[]).is_none());
    assert!(cmd(Agent::Deno, LogicalCommand::Dedupe, &[]).is_none());
    assert!(cmd(Agent::Npm, LogicalCommand::Dedupe, &[]).is_some());
    assert!(cmd(Agent::Pnpm, LogicalCommand::Dedupe, &[]).is_some());
}

#[test]
fn upgrade_interactive_unsupported_npm() {
    assert!(cmd(Agent::Npm, LogicalCommand::UpgradeInteractive, &[]).is_none());
    assert!(cmd(Agent::Yarn, LogicalCommand::UpgradeInteractive, &[]).is_some());
    assert!(cmd(Agent::Pnpm, LogicalCommand::UpgradeInteractive, &[]).is_some());
}

#[test]
fn execute_local_yarn_classic_inserts_dashdash() {
    let spec = cmd(
        Agent::Yarn,
        LogicalCommand::ExecuteLocal,
        &["esbuild", "--version"],
    )
    .unwrap();
    assert_eq!(spec.program, "yarn");
    assert_eq!(spec.args, vec!["exec", "esbuild", "--", "--version"]);
    let spec_simple = cmd(Agent::Yarn, LogicalCommand::ExecuteLocal, &["esbuild"]).unwrap();
    assert_eq!(spec_simple.args, vec!["exec", "esbuild"]);
}

#[test]
fn global_yarn_berry_delegates_to_npm() {
    let spec = cmd(Agent::YarnBerry, LogicalCommand::Global, &["eslint"]).unwrap();
    assert_eq!(spec.program, "npm");
    assert_eq!(spec.args, vec!["i", "-g", "eslint"]);
    let spec_uninstall = cmd(
        Agent::YarnBerry,
        LogicalCommand::GlobalUninstall,
        &["eslint"],
    )
    .unwrap();
    assert_eq!(spec_uninstall.program, "npm");
}

#[test]
fn resolve_command_preserves_args_literal() {
    // hostile values should remain literal, not shell interpolated
    let hostile = vec!["foo;echo hacked", "$(whoami)", "&&", "|", "\"hello world\""];
    for agent in [Agent::Npm, Agent::Pnpm, Agent::Yarn] {
        let spec = cmd(agent, LogicalCommand::Add, &hostile).unwrap();
        for h in &hostile {
            assert!(
                spec.args.contains(&h.to_string()),
                "agent {:?} missing hostile {}",
                agent,
                h
            );
        }
        // ensure program not containing hostile
        assert!(!spec.program.contains(';'));
    }
}

#[test]
fn command_spec_cwd_preserved() {
    let mut spec = CommandSpec::new("npm", vec!["i".to_string()]);
    spec.cwd = Some(std::path::PathBuf::from("/tmp/foo"));
    assert_eq!(spec.cwd.unwrap().to_str().unwrap(), "/tmp/foo");
}

#[test]
fn split_run_args_respects_value_flags() {
    // Use internal helpers via resolve_command behavior: test via Npm run with value flags
    // Already tested via npm run inserts dashdash, but explicit:
    let spec = cmd(
        Agent::Npm,
        LogicalCommand::Run,
        &["-w", "packages/foo", "dev", "--watch"],
    )
    .unwrap();
    // before = ["-w","packages/foo"], script="dev", after=["--watch"] -> args = run -w packages/foo dev -- --watch ?
    // Actually dash_dash_arg will treat "-w" as flag, packages/foo as value, so dev is script
    // Expect: npm run -w packages/foo dev -- --watch
    assert!(spec.args.contains(&"dev".to_string()));
    assert!(spec.args.contains(&"--".to_string()));
}
