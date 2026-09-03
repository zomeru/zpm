#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
use clap::Parser;
use zpm::cli::{Cli, Commands};
use zpm::config::Config;

// Helper to simulate CLI parsing via Cli::parse_from?

fn parse_cli(args: &[&str]) -> Cli {
    Cli::try_parse_from(args).expect("cli parse failed")
}

#[test]
fn parse_install() {
    let cli = parse_cli(&["zpm", "install"]);
    assert!(matches!(cli.command, Some(Commands::Install(_))));
}

#[test]
fn parse_add() {
    let cli = parse_cli(&["zpm", "add", "react"]);
    match cli.command {
        Some(Commands::Add(a)) => assert_eq!(a.packages, vec!["react"]),
        _ => panic!("expected add"),
    }
}

#[test]
fn parse_add_dev() {
    let cli = parse_cli(&["zpm", "add", "typescript", "--dev"]);
    match cli.command {
        Some(Commands::Add(a)) => {
            assert_eq!(a.packages, vec!["typescript"]);
            assert!(a.dev);
        }
        _ => panic!("expected add"),
    }
}

#[test]
fn parse_remove() {
    let cli = parse_cli(&["zpm", "remove", "react"]);
    match cli.command {
        Some(Commands::Remove(a)) => assert_eq!(a.packages, vec!["react"]),
        _ => panic!("expected remove"),
    }
}

#[test]
fn parse_run() {
    let cli = parse_cli(&["zpm", "run", "dev"]);
    match cli.command {
        Some(Commands::Run(a)) => assert_eq!(a.script.as_deref(), Some("dev")),
        _ => panic!("expected run"),
    }
}

#[test]
fn parse_run_with_args() {
    let cli = parse_cli(&["zpm", "run", "dev", "--", "--port", "3000"]);
    match cli.command {
        Some(Commands::Run(a)) => {
            assert_eq!(a.script.as_deref(), Some("dev"));
            assert_eq!(a.args, vec!["--port", "3000"]);
        }
        _ => panic!("expected run"),
    }
}

#[test]
fn parse_exec() {
    let cli = parse_cli(&["zpm", "exec", "vite", "--host", "0.0.0.0"]);
    match cli.command {
        Some(Commands::Exec(a)) => {
            assert_eq!(a.command.as_deref(), Some("vite"));
            assert_eq!(a.args, vec!["--host", "0.0.0.0"]);
        }
        _ => panic!("expected exec"),
    }
}

#[test]
fn parse_aliases() {
    let cli = parse_cli(&["zpm", "i"]);
    assert!(matches!(cli.command, Some(Commands::Install(_))));
    let cli = parse_cli(&["zpm", "a", "react"]);
    assert!(matches!(cli.command, Some(Commands::Add(_))));
    let cli = parse_cli(&["zpm", "rm", "react"]);
    assert!(matches!(cli.command, Some(Commands::Remove(_))));
    let cli = parse_cli(&["zpm", "up"]);
    assert!(matches!(cli.command, Some(Commands::Update(_))));
    let cli = parse_cli(&["zpm", "r", "dev"]);
    assert!(matches!(cli.command, Some(Commands::Run(_))));
    let cli = parse_cli(&["zpm", "x", "vite"]);
    assert!(matches!(cli.command, Some(Commands::Exec(_))));
}

#[test]
fn dry_run_flag() {
    let cli = parse_cli(&["zpm", "--dry-run", "install"]);
    assert!(cli.dry_run);
}

#[test]
fn pm_override() {
    let cli = parse_cli(&["zpm", "--pm", "pnpm", "install"]);
    assert_eq!(cli.pm.as_deref(), Some("pnpm"));
}

#[test]
fn verbose_flag() {
    let cli = parse_cli(&["zpm", "--verbose", "install"]);
    assert!(cli.verbose);
}

#[test]
fn root_flag() {
    let cli = parse_cli(&["zpm", "--root", "install"]);
    assert!(cli.root);
}

#[test]
fn no_interactive_flag() {
    let cli = parse_cli(&["zpm", "--no-interactive", "install"]);
    assert!(cli.no_interactive);
}

// Test resolve_cli integrates correctly with detection and command translation
#[test]
fn resolve_add_react_pnpm() {
    use zpm::cli::resolve_cli;
    let cli = parse_cli(&["zpm", "--pm", "pnpm", "add", "react"]);
    let config = Config::default();
    let cwd = std::env::current_dir().unwrap();
    let spec = resolve_cli(&cli, &config, &cwd).unwrap().unwrap();
    assert_eq!(spec.program, "pnpm");
    assert_eq!(spec.args, vec!["add", "react"]);
}

#[test]
fn resolve_add_dev_bun_uses_lowercase_d() {
    use zpm::cli::resolve_cli;
    let cli = parse_cli(&["zpm", "--pm", "bun", "add", "typescript", "--dev"]);
    let config = Config::default();
    let cwd = std::env::current_dir().unwrap();
    let spec = resolve_cli(&cli, &config, &cwd).unwrap().unwrap();
    assert_eq!(spec.program, "bun");
    assert_eq!(spec.args, vec!["add", "typescript", "-d"]);
}

#[test]
fn resolve_install_frozen_npm() {
    use zpm::cli::resolve_cli;
    let cli = parse_cli(&["zpm", "--pm", "npm", "install", "--frozen"]);
    let config = Config::default();
    let cwd = std::env::current_dir().unwrap();
    let spec = resolve_cli(&cli, &config, &cwd).unwrap().unwrap();
    assert_eq!(spec.program, "npm");
    assert_eq!(spec.args, vec!["ci"]);
}

#[test]
fn resolve_install_frozen_yarn_berry() {
    use zpm::cli::resolve_cli;
    let cli = parse_cli(&["zpm", "--pm", "yarn@berry", "install", "--frozen"]);
    let config = Config::default();
    let cwd = std::env::current_dir().unwrap();
    let spec = resolve_cli(&cli, &config, &cwd).unwrap().unwrap();
    assert_eq!(spec.program, "yarn");
    assert_eq!(spec.args, vec!["install", "--immutable"]);
}

#[test]
fn resolve_run_npm_inserts_dashdash() {
    use zpm::cli::resolve_cli;
    let cli = parse_cli(&["zpm", "--pm", "npm", "run", "build", "--watch", "-o"]);
    let config = Config::default();
    let cwd = std::env::current_dir().unwrap();
    let spec = resolve_cli(&cli, &config, &cwd).unwrap().unwrap();
    assert_eq!(spec.program, "npm");
    // npm run inserts --
    assert_eq!(spec.args, vec!["run", "build", "--", "--watch", "-o"]);
}

#[test]
fn resolve_run_pnpm_no_dashdash() {
    use zpm::cli::resolve_cli;
    let cli = parse_cli(&["zpm", "--pm", "pnpm", "run", "build", "--watch", "-o"]);
    let config = Config::default();
    let cwd = std::env::current_dir().unwrap();
    let spec = resolve_cli(&cli, &config, &cwd).unwrap().unwrap();
    assert_eq!(spec.args, vec!["run", "build", "--watch", "-o"]);
}

#[test]
fn resolve_exec_pnpm() {
    use zpm::cli::resolve_cli;
    let cli = parse_cli(&["zpm", "--pm", "pnpm", "exec", "vite", "--host", "0.0.0.0"]);
    let config = Config::default();
    let cwd = std::env::current_dir().unwrap();
    let spec = resolve_cli(&cli, &config, &cwd).unwrap().unwrap();
    assert_eq!(spec.program, "pnpm");
    assert_eq!(spec.args, vec!["dlx", "vite", "--host", "0.0.0.0"]);
}

#[test]
fn resolve_dedupe_check_maps() {
    use zpm::cli::resolve_cli;
    let cli = parse_cli(&["zpm", "--pm", "npm", "dedupe", "--check"]);
    let config = Config::default();
    let cwd = std::env::current_dir().unwrap();
    let spec = resolve_cli(&cli, &config, &cwd).unwrap().unwrap();
    assert_eq!(spec.args, vec!["dedupe", "--dry-run"]);

    let cli = parse_cli(&["zpm", "--pm", "pnpm", "dedupe", "--check"]);
    let spec = resolve_cli(&cli, &config, &cwd).unwrap().unwrap();
    assert_eq!(spec.args, vec!["dedupe", "--check"]);
}

#[test]
fn unsupported_dedupe_errors() {
    use zpm::cli::resolve_cli;
    let cli = parse_cli(&["zpm", "--pm", "bun", "dedupe"]);
    let config = Config::default();
    let cwd = std::env::current_dir().unwrap();
    let res = resolve_cli(&cli, &config, &cwd);
    assert!(res.is_err());
}

#[test]
fn clean_maps_to_frozen() {
    use zpm::cli::resolve_cli;
    let cli = parse_cli(&["zpm", "--pm", "npm", "clean"]);
    let config = Config::default();
    let cwd = std::env::current_dir().unwrap();
    let spec = resolve_cli(&cli, &config, &cwd).unwrap().unwrap();
    assert_eq!(spec.args, vec!["ci"]);
}

#[test]
fn bare_zpm_without_subcommand_uses_install() {
    use zpm::cli::resolve_cli;
    let cli = parse_cli(&["zpm", "--pm", "npm"]);
    // trailing empty should be install
    let config = Config::default();
    let cwd = std::env::current_dir().unwrap();
    let spec = resolve_cli(&cli, &config, &cwd).unwrap().unwrap();
    assert_eq!(spec.program, "npm");
    assert_eq!(spec.args, vec!["i"]);
}

#[test]
fn bare_zpm_with_package_is_add() {
    use zpm::cli::resolve_cli;
    // Simulate `zpm react` without subcommand: trailing contains react
    let cli = parse_cli(&["zpm", "--pm", "npm", "react"]);
    // Our Clap will treat "react" as trailing? Actually without subcommand, Cli.trailing captures it.
    // parse_cli with ["zpm", "--pm", "npm", "react"] will have command=None and trailing=["react"]
    // Let's manually construct: Cli::try_parse_from should put "react" into trailing.
    // But our earlier parse_cli used that; check.
    let config = Config::default();
    let cwd = std::env::current_dir().unwrap();
    let spec = resolve_cli(&cli, &config, &cwd).unwrap().unwrap();
    assert_eq!(spec.program, "npm");
    assert_eq!(spec.args, vec!["i", "react"]);
}
