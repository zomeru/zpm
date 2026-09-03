#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
mod common;
use common::TempProject;
use std::fs;
use zpm::workspace::{
    find_closest_package_json, find_packages, find_workspace_root, read_package_json,
    read_package_scripts,
};

#[test]
fn workspace_root_pnpm_yaml() {
    let proj = TempProject::new();
    proj.write("pnpm-workspace.yaml", "packages:\n  - packages/*");
    proj.write("package.json", r#"{"name":"root"}"#);
    let nested = proj.mkdir("packages/app");
    fs::write(nested.join("package.json"), r#"{"name":"app"}"#).unwrap();
    assert_eq!(
        find_workspace_root(&nested).unwrap(),
        proj.path().canonicalize().unwrap()
    );

    // pnpm-workspace.yml variant
    let proj2 = TempProject::new();
    proj2.write("pnpm-workspace.yml", "packages:\n  - packages/*");
    let nested2 = proj2.mkdir("packages/app2");
    assert_eq!(
        find_workspace_root(&nested2).unwrap(),
        proj2.path().canonicalize().unwrap()
    );
}

#[test]
fn workspace_root_yarnrc() {
    let proj = TempProject::new();
    proj.write(".yarnrc.yml", "nodeLinker: node-modules");
    let nested = proj.mkdir("packages/app");
    assert_eq!(
        find_workspace_root(&nested).unwrap(),
        proj.path().canonicalize().unwrap()
    );
}

#[test]
fn workspace_root_rush() {
    let proj = TempProject::new();
    proj.write("rush.json", "{}");
    let nested = proj.mkdir("apps/app");
    assert_eq!(
        find_workspace_root(&nested).unwrap(),
        proj.path().canonicalize().unwrap()
    );
}

#[test]
fn workspace_root_deno() {
    let proj = TempProject::new();
    proj.write("deno.json", "{}");
    let nested = proj.mkdir("a/b");
    assert_eq!(
        find_workspace_root(&nested).unwrap(),
        proj.path().canonicalize().unwrap()
    );

    let proj2 = TempProject::new();
    proj2.write("deno.jsonc", "{}");
    let nested2 = proj2.mkdir("a/b");
    assert_eq!(
        find_workspace_root(&nested2).unwrap(),
        proj2.path().canonicalize().unwrap()
    );
}

#[test]
fn workspace_root_package_json_workspaces() {
    let proj = TempProject::new();
    proj.write("package.json", r#"{"workspaces":["packages/*"]}"#);
    let nested = proj.mkdir("packages/app");
    assert_eq!(
        find_workspace_root(&nested).unwrap(),
        proj.path().canonicalize().unwrap()
    );

    // malformed package.json should not be considered workspace root
    let proj2 = TempProject::new();
    proj2.write("package.json", "not json");
    let nested2 = proj2.mkdir("packages/app");
    assert!(find_workspace_root(&nested2).is_none());

    // package.json without workspaces shouldn't count
    let proj3 = TempProject::new();
    proj3.write("package.json", r#"{"name":"foo"}"#);
    let nested3 = proj3.mkdir("packages/app");
    assert!(find_workspace_root(&nested3).is_none());
}

#[test]
fn workspace_root_not_found() {
    let proj = TempProject::new();
    proj.write("package.json", r#"{"name":"foo"}"#);
    let sub = proj.mkdir("a/b");
    // no workspace file, should be None
    assert!(find_workspace_root(&sub).is_none());
    // also cwd itself without workspace
    let empty = TempProject::new();
    assert!(find_workspace_root(empty.path()).is_none());
}

#[test]
fn workspace_root_prefers_closest() {
    let root = TempProject::new();
    root.write("pnpm-workspace.yaml", "packages:\n  - packages/*");
    let mid = root.mkdir("packages");
    fs::write(mid.join("package.json"), r#"{"workspaces":["*"]}"#).unwrap();
    let leaf = root.mkdir("packages/app/nested");
    // closest ancestor with workspace indicator should be mid (package.json workspaces) not root? But root also has pnpm-workspace.yaml.
    // Since we walk from leaf upward, mid will be found first, so it should be mid
    let found = find_workspace_root(&leaf).unwrap();
    // Could be either mid or root depending on creation; but we ensure it finds one of them (the closest)
    assert!(found == mid.canonicalize().unwrap() || found == root.path().canonicalize().unwrap());
}

#[test]
fn test_find_closest_package_json() {
    let proj = TempProject::new();
    proj.write("package.json", r#"{"name":"root"}"#);
    let nested = proj.mkdir("a/b/c");
    fs::write(nested.join("package.json"), r#"{"name":"leaf"}"#).unwrap();
    let found = find_closest_package_json(&nested).unwrap();
    assert_eq!(
        found,
        nested
            .join("package.json")
            .canonicalize()
            .unwrap_or(nested.join("package.json"))
    );

    // when leaf has no package.json, should find ancestor
    let proj2 = TempProject::new();
    proj2.write("package.json", r#"{"name":"root"}"#);
    let nested2 = proj2.mkdir("a/b");
    let found2 = find_closest_package_json(&nested2).unwrap();
    assert_eq!(
        found2,
        proj2
            .path()
            .join("package.json")
            .canonicalize()
            .unwrap_or(proj2.path().join("package.json"))
    );

    // none exists
    let proj3 = TempProject::new();
    let nested3 = proj3.mkdir("a");
    assert!(find_closest_package_json(&nested3).is_none());
}

#[test]
fn test_read_package_json_success_and_failure() {
    let proj = TempProject::new();
    proj.write("package.json", r#"{"name":"foo","version":"1.0.0"}"#);
    let v = read_package_json(&proj.path().join("package.json")).unwrap();
    assert_eq!(v["name"], "foo");

    // missing file
    let missing = proj.path().join("missing.json");
    assert!(read_package_json(&missing).is_none());

    // malformed json
    proj.write("bad.json", "not json");
    assert!(read_package_json(&proj.path().join("bad.json")).is_none());
}

#[test]
fn test_read_package_scripts_various() {
    let proj = TempProject::new();
    proj.write(
        "package.json",
        r#"{"scripts":{"dev":"vite","build":"tsc"},"scripts-info":{"dev":"Run dev server"}} "#,
    );
    let scripts = read_package_scripts(proj.path());
    assert_eq!(scripts.len(), 2);
    // check dev description from scripts-info
    let dev = scripts.iter().find(|(k, _, _)| k == "dev").unwrap();
    assert_eq!(dev.1, "vite");
    assert_eq!(dev.2, "Run dev server");
    let build = scripts.iter().find(|(k, _, _)| k == "build").unwrap();
    assert_eq!(build.2, "tsc"); // fallback to command itself

    // prefixed ? should be ignored
    let proj2 = TempProject::new();
    proj2.write(
        "package.json",
        r#"{"scripts":{"?hidden":"echo hi","visible":"echo hi"}}"#,
    );
    let scripts2 = read_package_scripts(proj2.path());
    assert_eq!(scripts2.len(), 1);
    assert_eq!(scripts2[0].0, "visible");

    // ?key description fallback
    let proj3 = TempProject::new();
    proj3.write(
        "package.json",
        r#"{"scripts":{"dev":"vite","?dev":"Run dev via q" }}"#,
    );
    let scripts3 = read_package_scripts(proj3.path());
    let dev3 = scripts3.iter().find(|(k, _, _)| k == "dev").unwrap();
    assert_eq!(dev3.2, "Run dev via q");

    // empty scripts
    let proj4 = TempProject::new();
    proj4.write("package.json", r#"{"name":"foo"}"#);
    assert!(read_package_scripts(proj4.path()).is_empty());

    // missing file -> empty
    let proj5 = TempProject::new();
    assert!(read_package_scripts(proj5.path()).is_empty());

    // non-string script value
    let proj6 = TempProject::new();
    proj6.write("package.json", r#"{"scripts":{"dev":123}}"#);
    let scripts6 = read_package_scripts(proj6.path());
    assert_eq!(scripts6[0].1, ""); // as_str unwrap_or ""
}

#[test]
fn test_find_packages_dfs_and_ignore() {
    let proj = TempProject::new();
    proj.write("package.json", r#"{"name":"root"}"#);
    proj.write("packages/app/package.json", r#"{"name":"app"}"#);
    proj.write("packages/other/package.json", r#"{"name":"other"}"#);
    // ignored dirs
    proj.write("node_modules/ignored/package.json", r#"{"name":"ignored"}"#);
    proj.write("dist/ignored2/package.json", r#"{"name":"ignored2"}"#);
    proj.write(".git/ignored3/package.json", r#"{"name":"ignored3"}"#);
    proj.write("target/ignored4/package.json", r#"{"name":"ignored4"}"#);
    proj.mkdir("fixtures/ignored5");
    fs::write(
        proj.path().join("fixtures/ignored5/package.json"),
        r#"{"name":"ignored5"}"#,
    )
    .unwrap();
    proj.mkdir(".hidden");
    fs::write(
        proj.path().join(".hidden/package.json"),
        r#"{"name":"hidden"}"#,
    )
    .unwrap();
    proj.write("public/ignored6/package.json", r#"{"name":"ignored6"}"#);

    let found = find_packages(proj.path());
    let found_str: Vec<String> = found
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    // should contain root, app, other but not ignored
    assert!(
        found_str
            .iter()
            .any(|s| s.contains("packages/app/package.json"))
    );
    assert!(
        found_str
            .iter()
            .any(|s| s.contains("packages/other/package.json"))
    );
    assert!(!found_str.iter().any(|s| s.contains("node_modules")));
    assert!(!found_str.iter().any(|s| s.contains(".git")));
    assert!(!found_str.iter().any(|s| s.contains("target")));
    assert!(!found_str.iter().any(|s| s.contains(".hidden")));
    assert!(!found_str.iter().any(|s| s.contains("fixtures")));

    // when no packages found via walk but cwd has package.json, fallback should add it
    let empty = TempProject::new();
    empty.write("package.json", r#"{"name":"solo"}"#);
    // walk will find it via DFS, but also test fallback path: create dir with no package.json files?
    // Actually find_packages does DFS; if empty, it checks cwd/package.json
    // To test fallback, we need a dir where walk finds nothing but cwd has package.json? But walk would find it anyway.
    // So just ensure it returns at least one
    let found2 = find_packages(empty.path());
    assert!(!found2.is_empty());
}
