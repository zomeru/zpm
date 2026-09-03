#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::bool_assert_comparison)]
#![allow(unused)]
#![allow(clippy::uninlined_format_args)]
use std::str::FromStr;
use zpm::package_manager::Agent;

#[test]
fn agent_as_str_roundtrip() {
    for agent in Agent::all() {
        let s = agent.as_str();
        let parsed: Agent = s.parse().expect("parse failed");
        // Note: yarn@berry as_str is yarn@berry, parsed should equal
        // pnpm@6 as_str is pnpm@6 etc.
        assert_eq!(parsed.as_str(), s, "roundtrip failed for {:?}", agent);
    }
}

#[test]
fn agent_display_equals_as_str() {
    for agent in Agent::all() {
        assert_eq!(format!("{}", agent), agent.as_str());
    }
}

#[test]
fn agent_display_name() {
    assert_eq!(Agent::YarnBerry.display_name(), "yarn (berry)");
    assert_eq!(Agent::PnpmRush.display_name(), "rush-pnpm");
    assert_eq!(Agent::PnpmAt6.display_name(), "pnpm@6");
    assert_eq!(Agent::Npm.display_name(), "npm");
    assert_eq!(Agent::Bun.display_name(), "bun");
}

#[test]
fn agent_all_contains_expected() {
    let all = Agent::all();
    assert!(all.contains(&Agent::Npm));
    assert!(all.contains(&Agent::Yarn));
    assert!(all.contains(&Agent::YarnBerry));
    assert!(all.contains(&Agent::Pnpm));
    assert!(all.contains(&Agent::Bun));
    assert!(all.contains(&Agent::Deno));
    assert!(all.contains(&Agent::Aube));
    assert!(all.contains(&Agent::Nub));
    assert_eq!(all.len(), 10);
}

#[test]
fn agent_install_command_strips_version_suffix() {
    assert_eq!(Agent::Npm.install_command(), "npm");
    assert_eq!(Agent::YarnBerry.install_command(), "yarn");
    assert_eq!(Agent::PnpmAt6.install_command(), "pnpm");
    assert_eq!(Agent::PnpmRush.install_command(), "pnpm-rush");
    // for pnpm-rush the split('@') still yields "pnpm-rush"
}

#[test]
fn agent_from_str_aliases() {
    assert_eq!("npm".parse::<Agent>().unwrap(), Agent::Npm);
    assert_eq!("NPM".parse::<Agent>().unwrap(), Agent::Npm);
    assert_eq!("yarn".parse::<Agent>().unwrap(), Agent::Yarn);
    assert_eq!("yarn@berry".parse::<Agent>().unwrap(), Agent::YarnBerry);
    assert_eq!("yarn-berry".parse::<Agent>().unwrap(), Agent::YarnBerry);
    assert_eq!("berry".parse::<Agent>().unwrap(), Agent::YarnBerry);
    assert_eq!("pnpm".parse::<Agent>().unwrap(), Agent::Pnpm);
    assert_eq!("pnpm@6".parse::<Agent>().unwrap(), Agent::PnpmAt6);
    assert_eq!("pnpm6".parse::<Agent>().unwrap(), Agent::PnpmAt6);
    assert_eq!("pnpm-rush".parse::<Agent>().unwrap(), Agent::PnpmRush);
    assert_eq!("rush-pnpm".parse::<Agent>().unwrap(), Agent::PnpmRush);
    assert_eq!("rush".parse::<Agent>().unwrap(), Agent::PnpmRush);
    assert_eq!("bun".parse::<Agent>().unwrap(), Agent::Bun);
    assert_eq!("deno".parse::<Agent>().unwrap(), Agent::Deno);
    assert_eq!("aube".parse::<Agent>().unwrap(), Agent::Aube);
    assert_eq!("nub".parse::<Agent>().unwrap(), Agent::Nub);
}

#[test]
fn agent_from_str_case_insensitive() {
    assert_eq!("PnPm".parse::<Agent>().unwrap(), Agent::Pnpm);
    assert_eq!("YARN@BERRY".parse::<Agent>().unwrap(), Agent::YarnBerry);
    assert_eq!("BUN".parse::<Agent>().unwrap(), Agent::Bun);
}

#[test]
fn agent_unknown_errors() {
    let err = "unknown_pm_xyz".parse::<Agent>().unwrap_err();
    assert!(err.to_string().contains("unknown agent"));
}

#[test]
fn agent_unknown_with_version_like() {
    let err = "npm@unknown".parse::<Agent>().unwrap_err();
    // only exact matches listed; "npm@unknown" is not an alias, so error
    assert!(err.to_string().contains("unknown agent"));
}

#[test]
fn agent_cli_override_variants() {
    // used via Cli pm override, but here test parsing directly
    for s in [
        "npm", "pnpm", "yarn", "berry", "pnpm@6", "rush", "bun", "deno", "aube", "nub",
    ] {
        assert!(s.parse::<Agent>().is_ok(), "failed for {}", s);
    }
}
