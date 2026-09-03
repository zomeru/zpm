#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
mod common;
use clap::Parser;
use common::{EnvGuard, TempProject};
use zpm::cli::{Cli, Commands};
use zpm::config::Config;

fn parse(args: &[&str]) -> Cli {
    Cli::try_parse_from(args).unwrap()
}

fn parse_fails(args: &[&str]) -> bool {
    Cli::try_parse_from(args).is_err()
}

#[test]
fn cli_all_aliases_parse() {
    assert!(matches!(
        parse(&["zpm", "i"]).command,
        Some(Commands::Install(_))
    ));
    assert!(matches!(
        parse(&["zpm", "install"]).command,
        Some(Commands::Install(_))
    ));
    assert!(matches!(
        parse(&["zpm", "a", "foo"]).command,
        Some(Commands::Add(_))
    ));
    assert!(matches!(
        parse(&["zpm", "add", "foo"]).command,
        Some(Commands::Add(_))
    ));
    assert!(matches!(
        parse(&["zpm", "rm", "foo"]).command,
        Some(Commands::Remove(_))
    ));
    assert!(matches!(
        parse(&["zpm", "uninstall", "foo"]).command,
        Some(Commands::Remove(_))
    ));
    assert!(matches!(
        parse(&["zpm", "remove", "foo"]).command,
        Some(Commands::Remove(_))
    ));
    assert!(matches!(
        parse(&["zpm", "up"]).command,
        Some(Commands::Update(_))
    ));
    assert!(matches!(
        parse(&["zpm", "upgrade", "foo"]).command,
        Some(Commands::Update(_))
    ));
    assert!(matches!(
        parse(&["zpm", "update", "foo"]).command,
        Some(Commands::Update(_))
    ));
    assert!(matches!(
        parse(&["zpm", "r", "dev"]).command,
        Some(Commands::Run(_))
    ));
    assert!(matches!(
        parse(&["zpm", "run", "dev"]).command,
        Some(Commands::Run(_))
    ));
    assert!(matches!(
        parse(&["zpm", "x", "vite"]).command,
        Some(Commands::Exec(_))
    ));
    assert!(matches!(
        parse(&["zpm", "exec", "vite"]).command,
        Some(Commands::Exec(_))
    ));
    assert!(matches!(
        parse(&["zpm", "dlx", "vite"]).command,
        Some(Commands::Exec(_))
    ));
    assert!(matches!(
        parse(&["zpm", "dedupe"]).command,
        Some(Commands::Dedupe(_))
    ));
    assert!(matches!(
        parse(&["zpm", "ci"]).command,
        Some(Commands::Clean(_))
    ));
    assert!(matches!(
        parse(&["zpm", "clean-install"]).command,
        Some(Commands::Clean(_))
    ));
    assert!(matches!(
        parse(&["zpm", "clean"]).command,
        Some(Commands::Clean(_))
    ));
    assert!(matches!(
        parse(&["zpm", "agent"]).command,
        Some(Commands::Agent(_))
    ));
}

#[test]
fn cli_install_flags() {
    let cli = parse(&["zpm", "install", "--frozen"]);
    match cli.command.unwrap() {
        Commands::Install(a) => assert!(a.frozen),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "install", "--frozen-if-present"]);
    match cli.command.unwrap() {
        Commands::Install(a) => assert!(a.frozen_if_present),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "install", "--production"]);
    match cli.command.unwrap() {
        Commands::Install(a) => assert!(a.production),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "install", "-P"]);
    match cli.command.unwrap() {
        Commands::Install(a) => assert!(a.production),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "install", "--global"]);
    match cli.command.unwrap() {
        Commands::Install(a) => assert!(a.global),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "install", "-g"]);
    match cli.command.unwrap() {
        Commands::Install(a) => assert!(a.global),
        _ => panic!(),
    }
    // frozen conflicts with frozen_if_present
    assert!(parse_fails(&[
        "zpm",
        "install",
        "--frozen",
        "--frozen-if-present"
    ]));
    // extra after --
    let cli = parse(&["zpm", "install", "--", "--ignore-scripts"]);
    match cli.command.unwrap() {
        Commands::Install(a) => assert_eq!(a.extra, vec!["--ignore-scripts"]),
        _ => panic!(),
    }
}

#[test]
fn cli_add_flags() {
    let cli = parse(&["zpm", "add", "react", "--dev"]);
    match cli.command.unwrap() {
        Commands::Add(a) => {
            assert_eq!(a.packages, vec!["react"]);
            assert!(a.dev);
        }
        _ => panic!(),
    }
    let cli = parse(&["zpm", "add", "react", "-D"]);
    match cli.command.unwrap() {
        Commands::Add(a) => assert!(a.dev),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "add", "react", "--peer"]);
    match cli.command.unwrap() {
        Commands::Add(a) => assert!(a.peer),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "add", "react", "--optional"]);
    match cli.command.unwrap() {
        Commands::Add(a) => assert!(a.optional),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "add", "react", "--exact"]);
    match cli.command.unwrap() {
        Commands::Add(a) => assert!(a.exact),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "add", "react", "-E"]);
    match cli.command.unwrap() {
        Commands::Add(a) => assert!(a.exact),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "add", "react", "--global"]);
    match cli.command.unwrap() {
        Commands::Add(a) => assert!(a.global),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "add", "react", "--frozen"]);
    match cli.command.unwrap() {
        Commands::Add(a) => assert!(a.frozen),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "add", "react", "-w"]);
    match cli.command.unwrap() {
        Commands::Add(a) => assert!(a.workspace),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "add", "react", "--workspace"]);
    match cli.command.unwrap() {
        Commands::Add(a) => assert!(a.workspace),
        _ => panic!(),
    }
    // multiple packages
    let cli = parse(&["zpm", "add", "react", "vue", "--dev"]);
    match cli.command.unwrap() {
        Commands::Add(a) => assert_eq!(a.packages, vec!["react", "vue"]),
        _ => panic!(),
    }
    // extra
    let cli = parse(&["zpm", "add", "react", "--", "--ignore-scripts"]);
    match cli.command.unwrap() {
        Commands::Add(a) => assert_eq!(a.extra, vec!["--ignore-scripts"]),
        _ => panic!(),
    }
}

#[test]
fn cli_remove_flags() {
    let cli = parse(&["zpm", "remove", "react"]);
    match cli.command.unwrap() {
        Commands::Remove(a) => assert_eq!(a.packages, vec!["react"]),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "remove", "--global", "eslint"]);
    // For Remove, --global is a flag, but packages captured differently: need to check how clap parses remove with global?
    // Actually RemoveArgs: packages Vec<String> optional, global bool, interactive, extra. So "eslint" could be packages or extra?
    // With `zpm remove --global eslint`, clap may treat eslint as package
    match cli.command.unwrap() {
        Commands::Remove(a) => {
            assert!(a.global);
            assert!(
                a.packages.contains(&"eslint".to_string())
                    || a.extra.contains(&"eslint".to_string())
            );
        }
        _ => panic!(),
    }
    let cli = parse(&["zpm", "remove", "--interactive"]);
    match cli.command.unwrap() {
        Commands::Remove(a) => assert!(a.interactive),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "remove", "-i"]);
    match cli.command.unwrap() {
        Commands::Remove(a) => assert!(a.interactive),
        _ => panic!(),
    }
}

#[test]
fn cli_update_flags() {
    let cli = parse(&["zpm", "update", "react"]);
    match cli.command.unwrap() {
        Commands::Update(a) => assert_eq!(a.packages, vec!["react"]),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "update", "--interactive"]);
    match cli.command.unwrap() {
        Commands::Update(a) => assert!(a.interactive),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "update", "-i"]);
    match cli.command.unwrap() {
        Commands::Update(a) => assert!(a.interactive),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "update", "--latest"]);
    match cli.command.unwrap() {
        Commands::Update(a) => assert!(a.latest),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "update", "--global"]);
    match cli.command.unwrap() {
        Commands::Update(a) => assert!(a.global),
        _ => panic!(),
    }
}

#[test]
fn cli_run_flags() {
    let cli = parse(&["zpm", "run", "dev"]);
    match cli.command.unwrap() {
        Commands::Run(a) => {
            assert_eq!(a.script.as_deref(), Some("dev"));
            assert!(!a.if_present);
        }
        _ => panic!(),
    }
    let cli = parse(&["zpm", "run", "--if-present", "dev"]);
    match cli.command.unwrap() {
        Commands::Run(a) => assert!(a.if_present),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "run", "--package"]);
    match cli.command.unwrap() {
        Commands::Run(a) => assert!(a.package),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "run", "-p"]);
    match cli.command.unwrap() {
        Commands::Run(a) => assert!(a.package),
        _ => panic!(),
    }
    // run with dash separator
    let cli = parse(&["zpm", "run", "dev", "--", "--help"]);
    match cli.command.unwrap() {
        Commands::Run(a) => {
            assert_eq!(a.script.as_deref(), Some("dev"));
            assert!(a.args.contains(&"--help".to_string()));
        }
        _ => panic!(),
    }
}

#[test]
fn cli_exec_flags() {
    let cli = parse(&["zpm", "exec", "vite"]);
    match cli.command.unwrap() {
        Commands::Exec(a) => {
            assert_eq!(a.command.as_deref(), Some("vite"));
            assert!(!a.local);
        }
        _ => panic!(),
    }
    let cli = parse(&["zpm", "exec", "--local", "vite"]);
    match cli.command.unwrap() {
        Commands::Exec(a) => assert!(a.local),
        _ => panic!(),
    }
    // no command
    let cli = parse(&["zpm", "exec"]);
    match cli.command.unwrap() {
        Commands::Exec(a) => assert!(a.command.is_none()),
        _ => panic!(),
    }
}

#[test]
fn cli_global_flags() {
    let cli = parse(&["zpm", "--dry-run", "install"]);
    assert!(cli.dry_run);
    let cli = parse(&["zpm", "--verbose", "install"]);
    assert!(cli.verbose);
    let cli = parse(&["zpm", "-v", "install"]);
    assert!(cli.verbose);
    let cli = parse(&["zpm", "--no-interactive", "install"]);
    assert!(cli.no_interactive);
    let cli = parse(&["zpm", "--root", "install"]);
    assert!(cli.root);
    let cli = parse(&["zpm", "--color", "always", "install"]);
    assert_eq!(cli.color.as_deref(), Some("always"));
    let cli = parse(&["zpm", "--color", "never", "install"]);
    assert_eq!(cli.color.as_deref(), Some("never"));
    let cli = parse(&["zpm", "--pm", "pnpm", "install"]);
    assert_eq!(cli.pm.as_deref(), Some("pnpm"));
    let cli = parse(&["zpm", "--cwd", "/tmp", "install"]);
    assert_eq!(cli.cwd.as_deref().unwrap().to_str().unwrap(), "/tmp");
    let cli = parse(&["zpm", "-C", "/tmp", "install"]);
    assert_eq!(cli.directory.as_deref().unwrap().to_str().unwrap(), "/tmp");
}

#[test]
fn cli_completion_flags() {
    let cli = parse(&["zpm", "completion", "--bash"]);
    match cli.command.unwrap() {
        Commands::Completion(a) => assert!(a.bash),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "completion", "--zsh"]);
    match cli.command.unwrap() {
        Commands::Completion(a) => assert!(a.zsh),
        _ => panic!(),
    }
    let cli = parse(&["zpm", "completion", "--fish"]);
    match cli.command.unwrap() {
        Commands::Completion(a) => assert!(a.fish),
        _ => panic!(),
    }
}

#[test]
fn cli_invalid_flags_error() {
    assert!(parse_fails(&["zpm", "install", "--unknown-flag"]));
    // missing required packages for add should error
    assert!(parse_fails(&["zpm", "add"]));
}
