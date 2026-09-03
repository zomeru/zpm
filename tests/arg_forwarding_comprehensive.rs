#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
use zpm::package_manager::{Agent, LogicalCommand, resolve_command};

fn cmd(agent: Agent, cmd: LogicalCommand, args: &[&str]) -> Option<Vec<String>> {
    let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    resolve_command(agent, cmd, &owned).map(|c| c.args)
}

#[test]
fn zero_and_many_args() {
    assert_eq!(
        cmd(Agent::Npm, LogicalCommand::Install, &[]).unwrap(),
        vec!["i"]
    );
    assert_eq!(
        cmd(
            Agent::Npm,
            LogicalCommand::Install,
            &["--foo", "bar", "baz"]
        )
        .unwrap(),
        vec!["i", "--foo", "bar", "baz"]
    );
}

#[test]
fn args_with_dash() {
    let args = cmd(Agent::Npm, LogicalCommand::Run, &["dev", "--port", "3000"]).unwrap();
    assert_eq!(args, vec!["run", "dev", "--", "--port", "3000"]);
    // yarn no dash
    let args = cmd(Agent::Yarn, LogicalCommand::Run, &["dev", "--port", "3000"]).unwrap();
    assert_eq!(args, vec!["run", "dev", "--port", "3000"]);
}

#[test]
fn double_dash_forwarding() {
    // npm run with double dash already present? The dash_dash logic will insert extra --, but our resolve for Run via resolve_command does dash handling separately.
    // For npm, if args already contain "--", what happens? Let's test via CLI resolve path
    use clap::Parser;
    use zpm::cli::Cli;
    use zpm::config::Config;
    let proj = tempfile::TempDir::new().unwrap();
    std::fs::write(
        proj.path().join("package.json"),
        r#"{"scripts":{"dev":"vite"}}"#,
    )
    .unwrap();
    let cli =
        Cli::try_parse_from(["zpm", "--pm", "npm", "run", "dev", "--", "--port", "3000"]).unwrap();
    let cfg = Config::default();
    let spec = zpm::cli::resolve_cli(&cli, &cfg, proj.path())
        .unwrap()
        .unwrap();
    // Should have dev and forwarded --port
    assert!(spec.args.contains(&"dev".to_string()));
    assert!(spec.args.contains(&"--port".to_string()));
}

#[test]
fn quoted_values_and_spaces() {
    let spec = zpm::package_manager::CommandSpec::new(
        "npm",
        vec![
            "run".to_string(),
            "dev".to_string(),
            "arg with spaces".to_string(),
        ],
    );
    assert_eq!(spec.to_string_pretty(), "npm run dev \"arg with spaces\"");
    // actual resolve should preserve literal
    let args = cmd(Agent::Npm, LogicalCommand::Add, &["pkg with space"]).unwrap();
    assert_eq!(args, vec!["i", "pkg with space"]);
}

#[test]
fn unicode_and_empty_strings() {
    let spec = resolve_command(
        Agent::Pnpm,
        LogicalCommand::Add,
        &["🦀".to_string(), "".to_string(), " ".to_string()],
    )
    .unwrap();
    assert!(spec.args.contains(&"🦀".to_string()));
    assert!(spec.args.contains(&"".to_string()));
    assert!(spec.args.contains(&" ".to_string()));
}

#[test]
fn platform_osstring() {
    // Test with OsString-like values via String that contains non-UTF8? In Rust Strings are UTF8, but we test with unicode and special chars
    let vals = [
        "a", "b-c", "c_d", "e.f", "g@h", "i:j", "k/l", "m\\n", "o\tp",
    ];
    for v in vals {
        let spec = resolve_command(Agent::Yarn, LogicalCommand::Add, &[v.to_string()]).unwrap();
        assert!(spec.args.contains(&v.to_string()));
    }
}

#[test]
fn workspace_flag_merging() {
    use clap::Parser;
    use zpm::cli::Cli;
    use zpm::config::Config;
    let proj = tempfile::TempDir::new().unwrap();
    std::fs::write(
        proj.path().join("package.json"),
        r#"{"scripts":{"dev":"vite"}}"#,
    )
    .unwrap();
    // pnpm run with workspace flag
    let cli =
        Cli::try_parse_from(["zpm", "--pm", "pnpm", "run", "dev", "-w", "packages/foo"]).unwrap();
    let cfg = Config::default();
    let spec = zpm::cli::resolve_cli(&cli, &cfg, proj.path())
        .unwrap()
        .unwrap();
    assert!(spec.args.iter().any(|a| a.contains("packages/foo")));
    // with --workspace
    let cli = Cli::try_parse_from([
        "zpm",
        "--pm",
        "pnpm",
        "run",
        "dev",
        "--workspace",
        "packages/foo",
    ])
    .unwrap();
    let spec = zpm::cli::resolve_cli(&cli, &cfg, proj.path())
        .unwrap()
        .unwrap();
    assert!(spec.args.iter().any(|a| a.contains("packages/foo")));
    // value flag without value shouldn't merge
    let cli = Cli::try_parse_from(["zpm", "--pm", "npm", "run", "dev", "-w"]).unwrap();
    let spec = zpm::cli::resolve_cli(&cli, &cfg, proj.path())
        .unwrap()
        .unwrap();
    assert!(spec.args.contains(&"-w".to_string()));
}
