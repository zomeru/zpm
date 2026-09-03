#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
mod common;
use common::{EnvGuard, TempProject};
use std::fs;
use std::path::Path;
use zpm::package_manager::CommandSpec;
use zpm::process::{check_command_exists, execute_command};

#[test]
fn check_command_exists_positive_and_negative() {
    // cargo should exist
    assert!(check_command_exists("cargo"));
    assert!(check_command_exists("sh"));
    assert!(!check_command_exists(
        "this_command_does_not_exist_xyz_12345"
    ));
}

#[test]
fn execute_command_dry_run_does_not_spawn() {
    let proj = TempProject::new();
    let spec = CommandSpec::new("this_should_not_exist", vec!["arg".to_string()]);
    let code = execute_command(&spec, proj.path(), true, false).unwrap();
    assert_eq!(code, 0);
    // verbose + dry_run should also return 0 and not error
    let code2 = execute_command(&spec, proj.path(), true, true).unwrap();
    assert_eq!(code2, 0);
}

#[test]
fn execute_command_verbose_prints_without_spawn() {
    let proj = TempProject::new();
    let spec = CommandSpec::new("echo", vec!["hello".to_string()]);
    // dry_run verbose still not spawn, just print
    let code = execute_command(&spec, proj.path(), true, true).unwrap();
    assert_eq!(code, 0);
}

#[test]
fn execute_command_missing_executable_returns_127() {
    let proj = TempProject::new();
    let spec = CommandSpec::new("nonexistent_binary_xyz_123", vec![]);
    let code = execute_command(&spec, proj.path(), false, false).unwrap();
    assert_eq!(code, 127);
}

#[test]
fn execute_command_success_and_failure_via_sh() {
    // Use sh -c 'exit 0' and 'exit 42' to test exit codes
    let proj = TempProject::new();
    let spec_success = CommandSpec::new("sh", vec!["-c".to_string(), "exit 0".to_string()]);
    let code = execute_command(&spec_success, proj.path(), false, false).unwrap();
    assert_eq!(code, 0);
    let spec_fail = CommandSpec::new("sh", vec!["-c".to_string(), "exit 42".to_string()]);
    let code = execute_command(&spec_fail, proj.path(), false, false).unwrap();
    assert_eq!(code, 42);
}

#[test]
fn execute_command_respects_cwd() {
    let proj = TempProject::new();
    let sub = proj.mkdir("sub");
    fs::write(sub.join("marker.txt"), "hello").unwrap();
    // Run sh to check that cwd is as expected: `pwd` should be sub if spec.cwd Some, else cwd param
    // We test via using spec.cwd override
    let mut spec = CommandSpec::new(
        "sh",
        vec!["-c".to_string(), "test -f marker.txt".to_string()],
    );
    spec.cwd = Some(sub.clone());
    let code = execute_command(&spec, proj.path(), false, false).unwrap();
    assert_eq!(code, 0, "spec cwd should be used");

    // Without spec.cwd, should use passed cwd
    let spec2 = CommandSpec::new(
        "sh",
        vec!["-c".to_string(), "test -f marker.txt".to_string()],
    );
    let code2 = execute_command(&spec2, &sub, false, false).unwrap();
    assert_eq!(code2, 0);
    // If cwd is wrong, test should fail
    let spec3 = CommandSpec::new(
        "sh",
        vec!["-c".to_string(), "test -f marker.txt".to_string()],
    );
    let code3 = execute_command(&spec3, proj.path(), false, false).unwrap();
    assert_ne!(code3, 0);
}

#[test]
fn execute_command_with_fake_executable_via_path() {
    let guard = EnvGuard::new_with_lock(&["PATH"]);
    let proj = TempProject::new();
    let fake = proj.create_fake_executable("fake_pm_xyz", 0);
    // Add its bin dir to PATH
    let bin_dir = proj.bin_dir();
    let current = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), current);
    guard.set("PATH", &new_path);
    assert!(check_command_exists("fake_pm_xyz"));
    let spec = CommandSpec::new("fake_pm_xyz", vec!["--version".to_string()]);
    let code = execute_command(&spec, proj.path(), false, false).unwrap();
    assert_eq!(code, 0);
    // non-zero exit
    let proj2 = TempProject::new();
    let fake2 = proj2.create_fake_executable("fake_fail_xyz", 7);
    let bin2 = proj2.bin_dir();
    let new_path2 = format!("{}:{}", bin2.display(), current);
    guard.set("PATH", &new_path2);
    let spec2 = CommandSpec::new("fake_fail_xyz", vec![]);
    let code2 = execute_command(&spec2, proj2.path(), false, false).unwrap();
    assert_eq!(code2, 7);
}

#[test]
fn execute_command_args_forwarded_literally() {
    // Test that hostile args are forwarded without shell interpretation by using a logging fake
    let guard = EnvGuard::new_with_lock(&["PATH"]);
    let proj = TempProject::new();
    let log = proj.path().join("args.log");
    let fake = proj.create_logging_executable("fake_logger_xyz", &log);
    let bin_dir = proj.bin_dir();
    let current = std::env::var("PATH").unwrap_or_default();
    guard.set("PATH", &format!("{}:{}", bin_dir.display(), current));
    assert!(check_command_exists("fake_logger_xyz"));
    let hostile = vec![
        "foo;echo hacked".to_string(),
        "$(whoami)".to_string(),
        "&&".to_string(),
        "|".to_string(),
    ];
    let spec = CommandSpec::new("fake_logger_xyz", hostile.clone());
    let code = execute_command(&spec, proj.path(), false, false).unwrap();
    assert_eq!(code, 0);
    // The fake executable logs args via echo "$@" ; check log file contains hostile strings as separate args
    // On Unix, our fake script does: echo "$@" > log ; That will join args with space
    let logged = fs::read_to_string(&log).unwrap();
    for h in hostile {
        assert!(
            logged.contains(&h),
            "logged should contain hostile literal {}",
            h
        );
    }
}
