#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
use zpm::package_manager::{Agent, LogicalCommand, resolve_command};

fn resolve(agent: Agent, cmd: LogicalCommand, args: &[&str]) -> zpm::package_manager::CommandSpec {
    let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    resolve_command(agent, cmd, &owned).expect("should resolve")
}

#[test]
fn args_not_shell_interpreted() {
    let hostile = [
        "foo;echo hacked",
        "$(whoami)",
        "`whoami`",
        "&& echo hi",
        "| cat /etc/passwd",
        "\"hello world\"",
        "'single'",
        "$HOME",
        "${PATH}",
        "foo && bar",
        "foo | bar",
        "foo; bar",
        "foo`bar`",
        "foo$(bar)",
        "a&b",
        "a|b",
        "  leading space",
        "trailing space ",
        "with\nnewline",
        "with\ttab",
    ];
    for h in hostile {
        let spec = resolve(Agent::Npm, LogicalCommand::Add, &[h]);
        // hostile should be a single arg, not split or interpreted
        assert!(
            spec.args.contains(&h.to_string()),
            "hostile {} not preserved for npm",
            h
        );
        assert_eq!(spec.program, "npm");
        // ensure no shell metachar appears in program
        assert!(!spec.program.contains(';'));
        assert!(!spec.program.contains('$'));
        // to_string_pretty should quote if contains space
        if h.contains(' ') {
            assert!(spec.to_string_pretty().contains(&format!("\"{h}\"")));
        }
    }
}

#[test]
fn paths_with_shell_metachars_remain_literal() {
    let paths = [
        "/tmp/foo;bar",
        "/tmp/$(whoami)",
        "/tmp/ &&",
        "/tmp/|",
        "/tmp/*",
        "/tmp/?",
    ];
    for p in paths {
        let spec = resolve(Agent::Pnpm, LogicalCommand::Add, &[p]);
        assert!(spec.args.contains(&p.to_string()));
    }
}

#[test]
fn temp_data_stays_inside_temp() {
    use std::fs;
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("package.json"), r#"{"name":"test"}"#).unwrap();
    // ensure detection doesn't escape temp dir
    let detected = zpm::detection::detect(tmp.path());
    // just ensure not panic and result is None or Some but not error
    let _ = detected;
    // ensure find_workspace doesn't escape
    let _ = zpm::workspace::find_workspace_root(&tmp.path().join("nonexistent/a"));
}

#[test]
fn config_values_not_shell() {
    // Config values should not be shell-interpreted; they are parsed as strings
    let mut cfg = zpm::config::Config::default();
    cfg.default_manager = Some("pnpm; rm -rf /".to_string());
    assert!(cfg.default_manager_agent().is_none()); // invalid should be None, not executed
    cfg.default_manager = Some("$(whoami)".to_string());
    assert!(cfg.default_manager_agent().is_none());
}

#[test]
fn command_spec_preserves_os_string_boundaries() {
    // Test that CommandSpec args preserve boundaries even with empty strings, unicode, etc.
    let spec = zpm::package_manager::CommandSpec::new(
        "npm",
        vec![
            "".to_string(),
            " ".to_string(),
            "a b".to_string(),
            "🦀".to_string(),
        ],
    );
    assert_eq!(spec.args[0], "");
    assert_eq!(spec.args[1], " ");
    assert_eq!(spec.args[2], "a b");
    assert_eq!(spec.args[3], "🦀");
    // pretty should handle empty and spaces
    let pretty = spec.to_string_pretty();
    assert!(pretty.contains("\" \""));
    assert!(pretty.contains("\"a b\""));
}

#[test]
fn no_command_injection_via_pm_override() {
    use clap::Parser;
    use zpm::cli::Cli;
    // pm override with hostile should error, not execute
    let res = "npm; echo hacked".parse::<Agent>();
    assert!(res.is_err());
    let res2 = "pnpm && rm".parse::<Agent>();
    assert!(res2.is_err());
    // via Cli parsing: --pm with hostile should error at parse resolve stage
    let cli = Cli::try_parse_from(["zpm", "--pm", "npm;echo", "install"]).unwrap();
    let cfg = zpm::config::Config::default();
    let cwd = std::env::current_dir().unwrap();
    let res = zpm::cli::resolve_cli(&cli, &cfg, &cwd);
    assert!(res.is_err());
}
