#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
mod common;
use common::EnvGuard;
use zpm::package_manager::Agent;
use zpm::package_manager::CommandSpec;
use zpm::ui;

#[test]
fn ui_color_enabled_respects_no_color() {
    let guard = EnvGuard::new_with_lock(&["NO_COLOR"]);
    guard.set("NO_COLOR", "1");
    assert!(!ui::is_color_enabled());
    guard.remove("NO_COLOR");
    // without NO_COLOR, is_color_enabled depends on user_attended (false in test) => false
    // But we just verify it doesn't panic and respects env when set
    let _ = ui::is_color_enabled();
    guard.set("NO_COLOR", "");
    // empty value still is_ok? env var exists even if empty, so is_ok true => false
    // Our code checks is_ok, not value. So empty still disables color
    assert!(!ui::is_color_enabled());
    guard.remove("NO_COLOR");
}

#[test]
fn ui_formatters_respect_color() {
    let guard = EnvGuard::new_with_lock(&["NO_COLOR"]);
    guard.set("NO_COLOR", "1");
    assert_eq!(ui::header("hello"), "hello");
    assert_eq!(ui::success("ok"), "ok");
    assert_eq!(ui::error("fail"), "fail");
    assert_eq!(ui::dim("dimmed"), "dimmed");
    guard.remove("NO_COLOR");
    // Without NO_COLOR, but still non-TTY, color may still be disabled? Actually user_attended false => is_color_enabled false => same plain
    assert_eq!(ui::header("hello"), "hello");
    // We can't test color enabled without TTY; but ensure functions return non-empty
    let colored = ui::header("test");
    assert!(!colored.is_empty());
    assert!(colored.contains("test"));
}

#[test]
fn ui_print_functions_dont_panic() {
    // These print to stderr; just ensure they don't panic
    ui::print_detected(Agent::Npm, false);
    ui::print_detected(Agent::Pnpm, true);
    let spec = CommandSpec::new("npm", vec!["install".to_string()]);
    ui::print_command(&spec);
}

#[test]
fn ui_select_none_on_non_tty_or_empty() {
    // In non-interactive env, select_* should return None quickly because user_attended false?
    // However our ui::select_* still tries to interact via dialoguer which will fail when not TTY,
    // but they use .ok()?? and return None via ?? mechanism.
    // We test that they don't panic and return None for empty
    assert_eq!(ui::select_script(&[]), None);
    // Empty packages? select_packages with empty slice should return Some(empty?) Let's see code: it calls MultiSelect with items empty, interact_opt may return None or Some?
    // But select_packages doesn't check empty beforehand, so it may attempt prompt. In non-TTY, it should return None via flatten? Let's verify it doesn't panic
    let _ = ui::select_packages(&[]);
    let _ = ui::select_packages(&["a".to_string(), "b".to_string()]);
    let _ = ui::select_agent(vec![Agent::Npm, Agent::Pnpm]);
    let _ = ui::confirm_auto_install("npm", None);
    let _ = ui::confirm_auto_install("pnpm", Some("8.0.0"));
}

#[test]
fn ui_header_success_error_dim_consistency() {
    let guard = EnvGuard::new_with_lock(&["NO_COLOR"]);
    guard.set("NO_COLOR", "1");
    for text in ["", "hello", "a b c", "unicode: 🦀", "long text with spaces"] {
        assert_eq!(ui::header(text), text);
        assert_eq!(ui::success(text), text);
        assert_eq!(ui::error(text), text);
        assert_eq!(ui::dim(text), text);
    }
    guard.remove("NO_COLOR");
}

#[test]
fn ui_forced_color_branch() {
    let guard = EnvGuard::new_with_lock(&["ZPM_FORCE_COLOR", "NO_COLOR", "FORCE_COLOR"]);
    guard.remove("NO_COLOR");
    guard.remove("FORCE_COLOR");
    guard.set("ZPM_FORCE_COLOR", "1");
    assert!(ui::is_color_enabled());
    // When forced, header should contain ANSI codes, not plain
    let h = ui::header("test");
    assert_ne!(h, "test");
    assert!(h.contains("test"));
    let s = ui::success("ok");
    assert_ne!(s, "ok");
    let e = ui::error("fail");
    assert_ne!(e, "fail");
    let d = ui::dim("dim");
    assert_ne!(d, "dim");
    guard.remove("ZPM_FORCE_COLOR");
    guard.set("FORCE_COLOR", "1");
    guard.remove("NO_COLOR");
    assert!(ui::is_color_enabled());
    guard.remove("FORCE_COLOR");
}

#[test]
fn ui_confirm_auto_install_messages() {
    // Ensure both branches of message formatting are covered (version Some vs None)
    // In non-TTY, these will return false without prompting, but should not panic
    assert!(!ui::confirm_auto_install("npm", None));
    assert!(!ui::confirm_auto_install("pnpm", Some("9.0.0")));
}
