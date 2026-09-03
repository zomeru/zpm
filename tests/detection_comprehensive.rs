#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
mod common;
use common::TempProject;
use std::fs;
use tempfile::TempDir;
use zpm::detection::{detect, detect_with_fallback, has_lockfile};
use zpm::package_manager::Agent;

fn agent_str(p: &std::path::Path) -> Option<String> {
    detect(p).map(|r| r.agent.as_str().to_string())
}

#[test]
fn detects_all_lockfiles_individually() {
    let cases = [
        ("package-lock.json", "npm"),
        ("npm-shrinkwrap.json", "npm"),
        ("pnpm-lock.yaml", "pnpm"),
        ("pnpm-workspace.yaml", "pnpm"),
        ("yarn.lock", "yarn"),
        ("bun.lock", "bun"),
        ("bun.lockb", "bun"),
        ("deno.lock", "deno"),
        ("aube-lock.yaml", "aube"),
        ("aube-workspace.yaml", "aube"),
        ("nub.lock", "nub"),
    ];
    for (file, expected) in cases {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(file), "").unwrap();
        assert_eq!(
            agent_str(tmp.path()).as_deref(),
            Some(expected),
            "file {}",
            file
        );
    }
}

#[test]
fn lockfile_precedence_order() {
    // LOCKS order: aube before bun before deno before nub before pnpm before yarn before npm
    let tmp = TempProject::new();
    tmp.write("pnpm-lock.yaml", "");
    tmp.write("yarn.lock", "");
    tmp.write("package-lock.json", "");
    // pnpm should win over yarn and npm
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm"));

    let tmp2 = TempProject::new();
    tmp2.write("yarn.lock", "");
    tmp2.write("package-lock.json", "");
    assert_eq!(agent_str(tmp2.path()).as_deref(), Some("yarn"));

    let tmp3 = TempProject::new();
    tmp3.write("bun.lock", "");
    tmp3.write("package-lock.json", "");
    assert_eq!(agent_str(tmp3.path()).as_deref(), Some("bun"));

    let tmp4 = TempProject::new();
    tmp4.write("aube-lock.yaml", "");
    tmp4.write("pnpm-lock.yaml", "");
    assert_eq!(agent_str(tmp4.path()).as_deref(), Some("aube"));
}

#[test]
fn package_manager_overrides_lockfile_same_dir() {
    let tmp = TempProject::new();
    tmp.write("pnpm-lock.yaml", "");
    tmp.write("package.json", r#"{"packageManager":"npm@10.0.0"}"#);
    // packageManager field should override lockfile mapping
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("npm"));

    let tmp2 = TempProject::new();
    tmp2.write("package-lock.json", "");
    tmp2.write("package.json", r#"{"packageManager":"pnpm@8.15.0"}"#);
    assert_eq!(agent_str(tmp2.path()).as_deref(), Some("pnpm"));
}

#[test]
fn package_manager_field_variants() {
    let cases = [
        (r#"{"packageManager":"npm@10.2.3"}"#, "npm"),
        (r#"{"packageManager":"pnpm@8.15.0"}"#, "pnpm"),
        (r#"{"packageManager":"pnpm@6.32.0"}"#, "pnpm@6"),
        (r#"{"packageManager":"yarn@1.22.22"}"#, "yarn"),
        (r#"{"packageManager":"yarn@3.6.4"}"#, "yarn@berry"),
        (r#"{"packageManager":"yarn@4.0.0"}"#, "yarn@berry"),
        (r#"{"packageManager":"bun@1.0.0"}"#, "bun"),
        (r#"{"packageManager":"deno@2.0.0"}"#, "deno"),
        (r#"{"packageManager":"aube@1.0.0"}"#, "aube"),
        (r#"{"packageManager":"nub@1.0.0"}"#, "nub"),
    ];
    for (json, expected) in cases {
        let tmp = TempProject::new();
        tmp.write("package.json", json);
        assert_eq!(
            agent_str(tmp.path()).as_deref(),
            Some(expected),
            "json {}",
            json
        );
    }
}

#[test]
fn package_manager_yarn_berry_vs_classic_boundaries() {
    let tmp = TempProject::new();
    tmp.write("package.json", r#"{"packageManager":"yarn@1.0.0"}"#);
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("yarn"));
    let tmp2 = TempProject::new();
    tmp2.write("package.json", r#"{"packageManager":"yarn@2.0.0"}"#);
    assert_eq!(agent_str(tmp2.path()).as_deref(), Some("yarn@berry"));
    let tmp3 = TempProject::new();
    tmp3.write("package.json", r#"{"packageManager":"yarn@1.999.0"}"#);
    assert_eq!(agent_str(tmp3.path()).as_deref(), Some("yarn"));
}

#[test]
fn pnpm_version_boundaries() {
    let tmp = TempProject::new();
    tmp.write("package.json", r#"{"packageManager":"pnpm@6.0.0"}"#);
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm@6"));
    let tmp2 = TempProject::new();
    tmp2.write("package.json", r#"{"packageManager":"pnpm@7.0.0"}"#);
    assert_eq!(agent_str(tmp2.path()).as_deref(), Some("pnpm"));
    let tmp3 = TempProject::new();
    tmp3.write("package.json", r#"{"packageManager":"pnpm@5.18.0"}"#);
    assert_eq!(agent_str(tmp3.path()).as_deref(), Some("pnpm@6"));
}

#[test]
fn package_manager_with_caret_and_range_normalization() {
    let cases = [
        r#"{"packageManager":"pnpm@^8.15.0"}"#,
        r#"{"packageManager":"pnpm@^7.0.0"}"#,
        r#"{"packageManager":"yarn@^3.2.0"}"#,
        r#"{"packageManager":"npm@^10.0.0"}"#,
    ];
    for json in cases {
        let tmp = TempProject::new();
        tmp.write("package.json", json);
        assert!(agent_str(tmp.path()).is_some(), "failed for {}", json);
    }
    // complex range like "pnpm@8.15.0 || 9.0.0" should normalize to 8.15.0
    // The normalize logic takes first digits sequence; after ^ handling, it may keep "8.15.0"
}

#[test]
fn package_manager_invalid_values_fallback_to_lock_or_none() {
    let tmp = TempProject::new();
    tmp.write("package.json", r#"{"packageManager":"invalid"}"#);
    // invalid manager should not be detected; with no lock, None
    assert_eq!(agent_str(tmp.path()), None);

    let tmp2 = TempProject::new();
    tmp2.write("package.json", r#"{"packageManager":"npm"}"#); // no version
    assert_eq!(agent_str(tmp2.path()).as_deref(), Some("npm"));

    let tmp3 = TempProject::new();
    tmp3.write("package.json", r#"{"packageManager":"@invalid"}"#);
    assert_eq!(agent_str(tmp3.path()), None);

    let tmp4 = TempProject::new();
    tmp4.write("package.json", r#"{"packageManager":"pnpm@"}"#);
    assert_eq!(agent_str(tmp4.path()), None);

    let tmp5 = TempProject::new();
    tmp5.write("package.json", r#"{"packageManager":""}"#);
    assert_eq!(agent_str(tmp5.path()), None);
}

#[test]
fn detects_via_dev_engines() {
    let tmp = TempProject::new();
    tmp.write(
        "package.json",
        r#"{"devEngines":{"packageManager":{"name":"pnpm","version":"9.0.0"}}}"#,
    );
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm"));

    let tmp2 = TempProject::new();
    tmp2.write(
        "package.json",
        r#"{"devEngines":{"packageManager":{"name":"yarn","version":"3.2.0"}}}"#,
    );
    assert_eq!(agent_str(tmp2.path()).as_deref(), Some("yarn@berry"));

    // missing version
    let tmp3 = TempProject::new();
    tmp3.write(
        "package.json",
        r#"{"devEngines":{"packageManager":{"name":"bun"}}}"#,
    );
    assert_eq!(agent_str(tmp3.path()).as_deref(), Some("bun"));

    // devEngines without name
    let tmp4 = TempProject::new();
    tmp4.write("package.json", r#"{"devEngines":{"packageManager":{}}}"#);
    assert_eq!(agent_str(tmp4.path()), None);
}

#[test]
fn deno_early_vs_fallback() {
    // early deno check: only target dir
    let tmp = TempProject::new();
    tmp.write("deno.json", "{}");
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("deno"));
    let nested = tmp.mkdir("a/b");
    // early check for nested should still find deno in cwd? Actually early_deno_check checks first dir only for deno.json
    // Then deno_fallback checks ancestors for deno.json after other checks
    // For nested, deno.json in parent should be found via fallback after checking nested dir's own signals
    assert_eq!(agent_str(&nested).as_deref(), Some("deno"));

    // deno.jsonc variant
    let tmp2 = TempProject::new();
    tmp2.write("deno.jsonc", "{}");
    assert_eq!(agent_str(tmp2.path()).as_deref(), Some("deno"));

    // ensure deno early takes precedence over other lockfiles in same dir?
    // early_deno_check returns before checking lockfiles, so deno wins if in target dir
    let tmp3 = TempProject::new();
    tmp3.write("deno.json", "{}");
    tmp3.write("pnpm-lock.yaml", "");
    assert_eq!(agent_str(tmp3.path()).as_deref(), Some("deno"));
}

#[test]
fn deno_fallback_ancestor_with_pnpm_lock_in_nested() {
    // If nested has pnpm lock, it should be detected before falling back to deno ancestor
    let tmp = TempProject::new();
    tmp.write("deno.json", "{}");
    let nested = tmp.mkdir("packages/app");
    fs::write(nested.join("pnpm-lock.yaml"), "").unwrap();
    assert_eq!(agent_str(&nested).as_deref(), Some("pnpm"));
}

#[test]
fn rush_detection() {
    let tmp = TempProject::new();
    tmp.write("rush.json", "{}");
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm-rush"));

    // rush should take precedence over lockfiles in same dir (check_rush before lockfile)
    let tmp2 = TempProject::new();
    tmp2.write("rush.json", "{}");
    tmp2.write("pnpm-lock.yaml", "");
    assert_eq!(agent_str(tmp2.path()).as_deref(), Some("pnpm-rush"));
}

#[test]
fn install_metadata_detection() {
    // node_modules/.pnpm
    let tmp = TempProject::new();
    tmp.mkdir("node_modules/.pnpm");
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm"));

    // node_modules/.yarn_integrity -> yarn classic
    let tmp2 = TempProject::new();
    tmp2.write("node_modules/.yarn_integrity", "");
    assert_eq!(agent_str(tmp2.path()).as_deref(), Some("yarn"));

    // node_modules/.yarn-state.yml -> yarn berry? check is_yarn_classic_metadata only for .yarn_integrity, others -> berry
    let tmp3 = TempProject::new();
    tmp3.write("node_modules/.yarn-state.yml", "");
    // per signals, .yarn-state.yml maps to yarn, then is_yarn_classic_metadata returns false, so YarnBerry
    assert_eq!(agent_str(tmp3.path()).as_deref(), Some("yarn@berry"));

    // .pnp.cjs -> yarn berry (since only .yarn_integrity is classic)
    let tmp4 = TempProject::new();
    tmp4.write(".pnp.cjs", "");
    assert_eq!(agent_str(tmp4.path()).as_deref(), Some("yarn@berry"));
    assert!(agent_str(tmp4.path()).is_some());

    // node_modules/.aube, .deno, .package-lock.json etc.
    let tmp5 = TempProject::new();
    tmp5.mkdir("node_modules/.aube");
    assert_eq!(agent_str(tmp5.path()).as_deref(), Some("aube"));

    let tmp6 = TempProject::new();
    tmp6.mkdir("node_modules/.deno");
    assert_eq!(agent_str(tmp6.path()).as_deref(), Some("deno"));

    let tmp7 = TempProject::new();
    tmp7.write("node_modules/.package-lock.json", "");
    assert_eq!(agent_str(tmp7.path()).as_deref(), Some("npm"));
}

#[test]
fn has_lockfile_variants() {
    let tmp = TempProject::new();
    assert!(!has_lockfile(tmp.path()));
    tmp.write("package-lock.json", "");
    assert!(has_lockfile(tmp.path()));
    // nested
    let nested = tmp.mkdir("nested/a");
    assert!(has_lockfile(&nested));

    // rush counts
    let tmp2 = TempProject::new();
    tmp2.write("rush.json", "");
    assert!(has_lockfile(tmp2.path()));

    // packageManager field counts
    let tmp3 = TempProject::new();
    tmp3.write("package.json", r#"{"packageManager":"pnpm@8.0.0"}"#);
    assert!(has_lockfile(tmp3.path()));

    // ancestor lockfile
    let tmp4 = TempProject::new();
    tmp4.write("pnpm-lock.yaml", "");
    let sub = tmp4.mkdir("sub");
    assert!(has_lockfile(&sub));
}

#[test]
fn nested_ancestor_precedence_closest_wins() {
    let root = TempProject::new();
    root.write("pnpm-lock.yaml", "");
    let sub = root.mkdir("packages/app");
    fs::write(sub.join("package-lock.json"), "").unwrap();
    // closest dir (sub) has npm lock, should win over ancestor pnpm
    assert_eq!(agent_str(&sub).as_deref(), Some("npm"));

    let sub2 = root.mkdir("packages/other");
    // no lock in sub2, should fallback to root pnpm
    assert_eq!(agent_str(&sub2).as_deref(), Some("pnpm"));
}

#[test]
fn malformed_package_json_not_crash() {
    let tmp = TempProject::new();
    tmp.write("package.json", "not json");
    tmp.write("pnpm-lock.yaml", "");
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("pnpm"));

    let tmp2 = TempProject::new();
    tmp2.write("package.json", "not json");
    assert_eq!(agent_str(tmp2.path()), None);

    let tmp3 = TempProject::new();
    tmp3.write("package.json", r#"{"packageManager": 123}"#); // invalid type
    assert_eq!(agent_str(tmp3.path()), None);
}

#[test]
fn test_detect_with_fallback() {
    let tmp = TempProject::new();
    let res = detect_with_fallback(tmp.path(), Some(Agent::Pnpm)).unwrap();
    assert_eq!(res.agent, Agent::Pnpm);
    assert_eq!(res.strategy, "default");
    // when detection succeeds, fallback not used
    tmp.write("pnpm-lock.yaml", "");
    let res2 = detect_with_fallback(tmp.path(), Some(Agent::Npm)).unwrap();
    assert_eq!(res2.agent, Agent::Pnpm);
    // no fallback provided
    let tmp3 = TempProject::new();
    assert!(detect_with_fallback(tmp3.path(), None).is_none());
}

#[test]
fn filesystem_root_boundaries() {
    // ancestors should handle root; we just ensure detect doesn't panic on nested deep
    let tmp = TempProject::new();
    let deep = tmp.mkdir("a/b/c/d/e/f/g");
    assert_eq!(agent_str(&deep), None);
    tmp.write("pnpm-lock.yaml", "");
    assert_eq!(agent_str(&deep).as_deref(), Some("pnpm"));
}

#[test]
fn unknown_lock_semantics() {
    // Ensure that presence of unrelated files does not interfere
    let tmp = TempProject::new();
    tmp.write("some-random.lock", "");
    tmp.write("package.json", r#"{"name":"foo"}"#);
    assert_eq!(agent_str(tmp.path()), None);
}

#[test]
fn bun_lock_variants() {
    let tmp = TempProject::new();
    tmp.write("bun.lockb", "");
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("bun"));
    let tmp2 = TempProject::new();
    tmp2.write("bun.lock", "");
    assert_eq!(agent_str(tmp2.path()).as_deref(), Some("bun"));
}

#[test]
fn aube_and_nub_locks() {
    let tmp = TempProject::new();
    tmp.write("aube-lock.yaml", "");
    assert_eq!(agent_str(tmp.path()).as_deref(), Some("aube"));
    let tmp2 = TempProject::new();
    tmp2.write("aube-workspace.yaml", "");
    assert_eq!(agent_str(tmp2.path()).as_deref(), Some("aube"));
    let tmp3 = TempProject::new();
    tmp3.write("nub.lock", "");
    assert_eq!(agent_str(tmp3.path()).as_deref(), Some("nub"));
}
