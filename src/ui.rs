//! Presentation layer — colors, prompts, and formatting.
//!
//! Domain modules produce structured values; this module decides how to
//! render them. It respects `NO_COLOR`, TTY detection, and CI.

use console::style;
use dialoguer::{FuzzySelect, MultiSelect, Select, theme::ColorfulTheme};

use crate::package_manager::Agent;

pub fn is_color_enabled() -> bool {
    if std::env::var("ZPM_FORCE_COLOR").is_ok() {
        console::set_colors_enabled(true);
        return true;
    }
    if std::env::var("FORCE_COLOR").is_ok() {
        console::set_colors_enabled(true);
        return true;
    }
    if std::env::var("NO_COLOR").is_ok() {
        console::set_colors_enabled(false);
        return false;
    }
    let enabled = console::user_attended();
    console::set_colors_enabled(enabled);
    enabled
}

pub fn header(text: &str) -> String {
    if is_color_enabled() {
        style(text).cyan().bold().to_string()
    } else {
        text.to_string()
    }
}

pub fn success(text: &str) -> String {
    if is_color_enabled() {
        style(text).green().to_string()
    } else {
        text.to_string()
    }
}

pub fn error(text: &str) -> String {
    if is_color_enabled() {
        style(text).red().bold().to_string()
    } else {
        text.to_string()
    }
}

pub fn dim(text: &str) -> String {
    if is_color_enabled() {
        style(text).dim().to_string()
    } else {
        text.to_string()
    }
}

pub fn print_detected(agent: Agent, verbose: bool) {
    if verbose {
        eprintln!(
            "{} {}",
            dim("Detected package manager:"),
            style(agent.as_str()).yellow()
        );
    }
}

pub fn print_command(spec: &crate::package_manager::CommandSpec) {
    eprintln!(
        "{} {}",
        dim("Command:"),
        style(spec.to_string_pretty()).magenta()
    );
}

pub fn select_agent(agents: Vec<Agent>) -> Option<Agent> {
    let items: Vec<String> = agents.iter().map(|a| a.as_str().to_string()).collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose the package manager")
        .items(&items)
        .default(0)
        .interact_opt()
        .ok()??;
    Some(agents[selection])
}

pub fn select_script(scripts: &[(String, String, String)]) -> Option<String> {
    if scripts.is_empty() {
        eprintln!("{}", error("No scripts found in package.json"));
        return None;
    }
    let items: Vec<String> = scripts
        .iter()
        .map(|(k, _cmd, desc)| {
            if desc.is_empty() || desc == k {
                k.clone()
            } else {
                format!("{:<20} {}", k, dim(desc))
            }
        })
        .collect();
    let sel = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select script to run")
        .items(&items)
        .default(0)
        .interact_opt()
        .ok()??;
    Some(scripts[sel].0.clone())
}

pub fn select_packages(packages: &[String]) -> Option<Vec<String>> {
    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select packages to remove (space to select, enter to confirm)")
        .items(packages)
        .interact_opt()
        .ok()??;
    Some(selection.into_iter().map(|i| packages[i].clone()).collect())
}

pub fn confirm_auto_install(agent: &str, version: Option<&str>) -> bool {
    let msg = if let Some(v) = version {
        format!("Detected {agent}@{v} but it is not installed. Install it globally?")
    } else {
        format!("Detected {agent} but it is not installed. Install it globally?")
    };
    dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(msg)
        .default(false)
        .interact_opt()
        .ok()
        .flatten()
        .unwrap_or(false)
}
