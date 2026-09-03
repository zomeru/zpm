#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
use std::fs;
use tempfile::TempDir;
use zpm::detection::{detect, find_workspace_root, has_lockfile};

fn agent_str(path: &std::path::Path) -> Option<String> {
    detect(path).map(|r| r.agent.as_str().to_string())
}

#[test]
fn lockfile_detection() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("package-lock.json"), "{}").unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("npm"));

    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("pnpm-lock.yaml"), "").unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm"));

    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("yarn.lock"), "").unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("yarn"));

    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("bun.lockb"), "").unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("bun"));

    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("bun.lock"), "").unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("bun"));

    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("deno.lock"), "").unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("deno"));

    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("aube-lock.yaml"), "").unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("aube"));

    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("nub.lock"), "").unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("nub"));
}

#[test]
fn package_manager_field() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"packageManager":"pnpm@8.15.0"}"#,
    )
    .unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm"));

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"packageManager":"yarn@3.2.0"}"#,
    )
    .unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("yarn@berry"));

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"packageManager":"yarn@1.22.0"}"#,
    )
    .unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("yarn"));

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"packageManager":"npm@10.0.0"}"#,
    )
    .unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("npm"));

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"packageManager":"bun@1.0.0"}"#,
    )
    .unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("bun"));

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"packageManager":"deno@2.0.0"}"#,
    )
    .unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("deno"));

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"packageManager":"pnpm@6.32.0"}"#,
    )
    .unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm@6"));

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"packageManager":"aube@1.0.0"}"#,
    )
    .unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("aube"));

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"packageManager":"nub@1.0.0"}"#,
    )
    .unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("nub"));
}

#[test]
fn deno_json_detection() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("deno.json"), "{}").unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("deno"));

    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("deno.jsonc"), "{}").unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("deno"));
}

#[test]
fn rush_detection() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("rush.json"), "{}").unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm-rush"));
}

#[test]
fn nested_workspace_detection() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("pnpm-lock.yaml"), "").unwrap();
    let nested = tmp.path().join("packages/app");
    fs::create_dir_all(&nested).unwrap();
    assert_eq!(agent_str(&nested).as_deref(), Some("pnpm"));
    assert!(has_lockfile(&nested));
}

#[test]
fn conflicting_lockfiles_uses_precedence() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("package-lock.json"), "{}").unwrap();
    fs::write(tmp.path().join("pnpm-lock.yaml"), "").unwrap();
    // pnpm has higher precedence due to LOCKS order
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm"));
}

#[test]
fn workspace_root_detection() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("pnpm-workspace.yaml"),
        "packages:\n  - packages/*",
    )
    .unwrap();
    fs::write(tmp.path().join("package.json"), r#"{"name":"root"}"#).unwrap();
    let nested = tmp.path().join("packages/app");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("package.json"), r#"{"name":"app"}"#).unwrap();
    assert_eq!(
        find_workspace_root(&nested).unwrap(),
        tmp.path().canonicalize().unwrap()
    );

    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"workspaces":["packages/*"]}"#,
    )
    .unwrap();
    let nested = tmp.path().join("packages/app");
    fs::create_dir_all(&nested).unwrap();
    assert_eq!(
        find_workspace_root(&nested).unwrap(),
        tmp.path().canonicalize().unwrap()
    );

    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("rush.json"), "{}").unwrap();
    let nested = tmp.path().join("apps/app");
    fs::create_dir_all(&nested).unwrap();
    assert_eq!(
        find_workspace_root(&nested).unwrap(),
        tmp.path().canonicalize().unwrap()
    );
}

#[test]
fn has_lockfile_check() {
    let tmp = TempDir::new().unwrap();
    assert!(!has_lockfile(tmp.path()));
    fs::write(tmp.path().join("yarn.lock"), "").unwrap();
    assert!(has_lockfile(tmp.path()));
}

#[test]
fn dev_engines_field() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"devEngines":{"packageManager":{"name":"pnpm","version":"9.0.0"}}}"#,
    )
    .unwrap();
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm"));
}

#[test]
fn version_range_in_package_manager() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"packageManager":"pnpm@^8.15.0"}"#,
    )
    .unwrap();
    // Should parse ^ prefix and detect pnpm
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm"));
}
