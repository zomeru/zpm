//! CLI → domain translation.
//!
//! Converts `Cli` (user intent, as parsed by clap) into a structured
//! `CommandSpec`. No shell is involved; the resolver preserves argv boundaries
//! and delegates per-manager formatting to `crate::package_manager`.

use super::args::{
    AddArgs, CleanArgs, Cli, Commands, CompletionArgs, DedupeArgs, ExecArgs, InstallArgs,
    RemoveArgs, RunArgs, UpdateArgs,
};
use crate::{
    config::Config,
    detection,
    package_manager::{Agent, CommandSpec, LogicalCommand, resolve_command},
    ui,
};

/// Resolve `Cli` + `Config` + `cwd` into an executable `CommandSpec`.
///
/// Returns `Ok(None)` when nothing should be executed (e.g. `zpm agent` whose
/// side effect is printing, or an interactive cancellation).
pub fn resolve_cli(
    cli: &Cli,
    config: &Config,
    cwd: &std::path::Path,
) -> anyhow::Result<Option<CommandSpec>> {
    let mut effective_cwd = effective_working_dir(cli, cwd);
    if cli.root {
        if let Some(root) = detection::find_workspace_root(&effective_cwd) {
            effective_cwd = root;
        }
    }

    let agent_opt = resolve_agent_override(cli)?;
    let detected = if let Some(a) = agent_opt {
        Some(crate::detection::DetectionResult {
            agent: a,
            name: a.as_str().to_string(),
            version: None,
            path: effective_cwd.clone(),
            strategy: "override".to_string(),
        })
    } else {
        detection::detect(&effective_cwd)
    };

    let (agent, has_lock) = match detected {
        Some(d) => (d.agent, true),
        None => {
            if let Some(a) = config.default_manager_agent() {
                (a, false)
            } else if std::env::var("CI").is_ok()
                || cli.no_interactive
                || (!ui::is_color_enabled() && !console::user_attended())
            {
                (Agent::Npm, false)
            } else {
                let choices = vec![
                    Agent::Npm,
                    Agent::Pnpm,
                    Agent::Yarn,
                    Agent::YarnBerry,
                    Agent::Bun,
                    Agent::Deno,
                    Agent::Aube,
                    Agent::Nub,
                ];
                if let Some(selected) = ui::select_agent(choices) {
                    (selected, false)
                } else {
                    return Ok(None);
                }
            }
        }
    };

    // Bare invocation without subcommand — ni compatibility (`zpm`, `zpm react`).
    let Some(command) = cli.command.as_ref() else {
        return resolve_bare(agent, cli, has_lock);
    };

    let spec = match command {
        Commands::Install(a) => resolve_for_install(agent, a, has_lock)?,
        Commands::Add(a) => resolve_for_add(agent, a)?,
        Commands::Remove(a) => resolve_for_remove(agent, a, &effective_cwd)?,
        Commands::Update(a) => resolve_for_update(agent, a)?,
        Commands::Run(a) => resolve_for_run(agent, a, &effective_cwd, cli)?,
        Commands::Exec(a) => resolve_for_exec(agent, a)?,
        Commands::Dedupe(a) => resolve_for_dedupe(agent, a)?,
        Commands::Clean(a) => resolve_for_clean(agent, a)?,
        Commands::Agent(_) => {
            println!("{}", agent.as_str());
            return Ok(None);
        }
        Commands::Completion(c) => {
            print_completion(c);
            return Ok(None);
        }
    };

    let mut spec = spec;
    spec.cwd = Some(effective_cwd);
    Ok(Some(spec))
}

fn effective_working_dir(cli: &Cli, cwd: &std::path::Path) -> std::path::PathBuf {
    if let Some(dir) = &cli.directory {
        cwd.join(dir)
    } else if let Some(dir) = &cli.cwd {
        cwd.join(dir)
    } else {
        cwd.to_path_buf()
    }
}

fn resolve_agent_override(cli: &Cli) -> anyhow::Result<Option<Agent>> {
    if let Some(pm) = &cli.pm {
        return Ok(Some(pm.parse::<Agent>().map_err(|e| anyhow::anyhow!(e))?));
    }
    if let Ok(val) = std::env::var("ZPM_PM") {
        if !val.is_empty() {
            return Ok(Some(val.parse::<Agent>().map_err(|e| anyhow::anyhow!(e))?));
        }
    }
    Ok(None)
}

fn resolve_bare(agent: Agent, cli: &Cli, has_lock: bool) -> anyhow::Result<Option<CommandSpec>> {
    if cli.trailing.is_empty() {
        let resolved = resolve_for_install(
            agent,
            &InstallArgs {
                frozen: false,
                frozen_if_present: false,
                production: false,
                global: false,
                extra: vec![],
            },
            has_lock,
        )?;
        return Ok(Some(resolved));
    }
    let spec = parse_ni_like(agent, cli.trailing.clone(), has_lock)?;
    Ok(Some(spec))
}

fn print_completion(c: &CompletionArgs) {
    if c.bash {
        println!("{}", include_str!("../assets/completion.bash"));
    } else if c.zsh {
        println!("{}", include_str!("../assets/completion.zsh"));
    } else if c.fish {
        println!("{}", include_str!("../assets/completion.fish"));
    } else {
        println!("Usage: zpm completion --bash|--zsh|--fish");
    }
}

fn parse_ni_like(
    agent: Agent,
    mut args: Vec<String>,
    has_lock: bool,
) -> anyhow::Result<CommandSpec> {
    if agent == Agent::Bun {
        args = args
            .into_iter()
            .map(|a| if a == "-D" { "-d".to_string() } else { a })
            .collect();
    }
    if agent == Agent::Npm {
        args = args
            .into_iter()
            .map(|a| {
                if a == "-P" {
                    "--omit=dev".to_string()
                } else {
                    a
                }
            })
            .collect();
    }
    if args.contains(&"-P".to_string()) {
        args = args
            .into_iter()
            .map(|a| {
                if a == "-P" {
                    "--production".to_string()
                } else {
                    a
                }
            })
            .collect();
    }

    if args.contains(&"-g".to_string()) {
        let filtered: Vec<String> = args.into_iter().filter(|a| a != "-g").collect();
        return resolve_command(agent, LogicalCommand::Global, &filtered)
            .ok_or_else(|| anyhow::anyhow!("command not supported for agent {agent}"));
    }
    if args.contains(&"--frozen-if-present".to_string()) {
        let filtered: Vec<String> = args
            .into_iter()
            .filter(|a| a != "--frozen-if-present")
            .collect();
        let cmd = if has_lock {
            LogicalCommand::Frozen
        } else {
            LogicalCommand::Install
        };
        return resolve_command(agent, cmd, &filtered)
            .ok_or_else(|| anyhow::anyhow!("command not supported"));
    }
    if args.contains(&"--frozen".to_string()) {
        let filtered: Vec<String> = args.into_iter().filter(|a| a != "--frozen").collect();
        return resolve_command(agent, LogicalCommand::Frozen, &filtered)
            .ok_or_else(|| anyhow::anyhow!("command not supported"));
    }
    if args.is_empty() || args.iter().all(|a| a.starts_with('-')) {
        return resolve_command(agent, LogicalCommand::Install, &args)
            .ok_or_else(|| anyhow::anyhow!("command not supported"));
    }
    resolve_command(agent, LogicalCommand::Add, &args)
        .ok_or_else(|| anyhow::anyhow!("command not supported"))
}

fn resolve_for_install(
    agent: Agent,
    args: &InstallArgs,
    has_lock: bool,
) -> anyhow::Result<CommandSpec> {
    let extra = args.extra.clone();
    let mut prod_args = Vec::new();
    if args.production {
        if agent == Agent::Npm {
            prod_args.push("--omit=dev".to_string());
        } else {
            prod_args.push("--production".to_string());
        }
    }

    if args.frozen {
        let mut frozen_args = prod_args;
        frozen_args.extend(extra);
        return resolve_command(agent, LogicalCommand::Frozen, &frozen_args)
            .ok_or_else(|| anyhow::anyhow!("frozen install not supported for {agent}"));
    }
    if args.frozen_if_present {
        let cmd = if has_lock {
            LogicalCommand::Frozen
        } else {
            LogicalCommand::Install
        };
        let mut all = prod_args;
        all.extend(extra.clone());
        return resolve_command(agent, cmd, &all)
            .ok_or_else(|| anyhow::anyhow!("command not supported"));
    }

    if args.global {
        let mut all = prod_args.clone();
        all.extend(extra);
        return resolve_command(agent, LogicalCommand::Global, &all)
            .ok_or_else(|| anyhow::anyhow!("global not supported"));
    }

    let mut all = prod_args;
    all.extend(extra);
    resolve_command(agent, LogicalCommand::Install, &all)
        .ok_or_else(|| anyhow::anyhow!("install not supported"))
}

fn resolve_for_add(agent: Agent, args: &AddArgs) -> anyhow::Result<CommandSpec> {
    if args.global {
        let packages = args.packages.clone();
        let mut extra_flags = Vec::new();
        if args.dev {
            if agent == Agent::Bun {
                extra_flags.push("-d".to_string());
            } else {
                extra_flags.push("-D".to_string());
            }
        }
        if args.peer {
            extra_flags.push("--save-peer".to_string());
        }
        if args.exact {
            extra_flags.push("-E".to_string());
        }
        extra_flags.extend(args.extra.clone());
        let mut all = packages;
        all.extend(extra_flags);
        return resolve_command(agent, LogicalCommand::Global, &all)
            .ok_or_else(|| anyhow::anyhow!("global add not supported"));
    }

    let all_packages = args.packages.clone();
    let mut flags = Vec::new();
    if args.dev {
        if agent == Agent::Bun {
            flags.push("-d".to_string());
        } else {
            flags.push("-D".to_string());
        }
    }
    if args.peer {
        flags.push("--save-peer".to_string());
    }
    if args.optional {
        flags.push("--save-optional".to_string());
    }
    if args.exact {
        flags.push("-E".to_string());
    }
    if args.frozen {
        flags.push("--frozen".to_string());
    }
    flags.extend(args.extra.clone());

    let mut all = all_packages;
    all.extend(flags);
    resolve_command(agent, LogicalCommand::Add, &all)
        .ok_or_else(|| anyhow::anyhow!("add not supported"))
}

fn resolve_for_remove(
    agent: Agent,
    args: &RemoveArgs,
    cwd: &std::path::Path,
) -> anyhow::Result<CommandSpec> {
    let mut packages = args.packages.clone();
    if packages.is_empty() && !args.global {
        if !args.interactive {
            let is_interactive = console::user_attended()
                && std::env::var("ZPM_NO_INTERACTIVE")
                    .map(|v| v != "true")
                    .unwrap_or(true);
            if is_interactive {
                let pkg_path = crate::workspace::find_closest_package_json(cwd)
                    .unwrap_or_else(|| cwd.join("package.json"));
                if let Ok(content) = std::fs::read_to_string(&pkg_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        let deps = json.get("dependencies").and_then(|v| v.as_object());
                        let dev_deps = json.get("devDependencies").and_then(|v| v.as_object());
                        let mut all_deps = Vec::new();
                        if let Some(d) = deps {
                            for k in d.keys() {
                                all_deps.push(k.clone());
                            }
                        }
                        if let Some(d) = dev_deps {
                            for k in d.keys() {
                                all_deps.push(k.clone());
                            }
                        }
                        if !all_deps.is_empty() {
                            if let Some(selected) = ui::select_packages(&all_deps) {
                                if selected.is_empty() {
                                    anyhow::bail!("No packages selected");
                                }
                                packages = selected;
                            } else {
                                anyhow::bail!("Cancelled");
                            }
                        } else {
                            anyhow::bail!("No dependencies found in package.json");
                        }
                    } else {
                        anyhow::bail!("No dependencies found");
                    }
                } else {
                    anyhow::bail!("No dependencies found");
                }
            } else {
                anyhow::bail!("No packages specified for remove");
            }
        } else {
            anyhow::bail!("No packages specified");
        }
    }

    let extra = args.extra.clone();
    let mut all = packages;
    all.extend(extra);
    if args.global {
        resolve_command(agent, LogicalCommand::GlobalUninstall, &all)
            .ok_or_else(|| anyhow::anyhow!("global uninstall not supported"))
    } else {
        resolve_command(agent, LogicalCommand::Uninstall, &all)
            .ok_or_else(|| anyhow::anyhow!("uninstall not supported"))
    }
}

fn resolve_for_update(agent: Agent, args: &UpdateArgs) -> anyhow::Result<CommandSpec> {
    let interactive = args.interactive;
    let packages = args.packages.clone();
    let mut extra = args.extra.clone();
    if args.latest {
        extra.push("--latest".to_string());
    }
    let mut all = packages;
    all.extend(extra);
    if interactive {
        resolve_command(agent, LogicalCommand::UpgradeInteractive, &all)
            .or_else(|| resolve_command(agent, LogicalCommand::Upgrade, &all))
            .ok_or_else(|| anyhow::anyhow!("upgrade not supported for {agent}"))
    } else {
        resolve_command(agent, LogicalCommand::Upgrade, &all)
            .ok_or_else(|| anyhow::anyhow!("upgrade not supported"))
    }
}

fn resolve_for_run(
    agent: Agent,
    args: &RunArgs,
    cwd: &std::path::Path,
    cli: &Cli,
) -> anyhow::Result<CommandSpec> {
    let mut script = args.script.clone();
    let mut forwarded = args.args.clone();

    if script.is_none() && forwarded.is_empty() && !args.package {
        let is_tty = console::user_attended() && !cli.no_interactive;
        if is_tty {
            let scripts = crate::workspace::read_package_scripts(cwd);
            if scripts.is_empty() {
                anyhow::bail!("No scripts found in package.json");
            }
            if let Some(selected) = ui::select_script(&scripts) {
                script = Some(selected);
            } else {
                anyhow::bail!("No script selected");
            }
        } else {
            anyhow::bail!("No script specified and not in interactive terminal");
        }
    }

    if script.as_deref() == Some("-p") {
        if !forwarded.is_empty() {
            script = Some(forwarded.remove(0));
        } else {
            script = None;
        }
    }

    let mut all_raw = Vec::new();
    if let Some(s) = script.clone() {
        all_raw.push(s);
    }
    all_raw.extend(forwarded.clone());
    let mut all = Vec::new();
    let mut i = 0;
    while i < all_raw.len() {
        let arg = &all_raw[i];
        if (arg == "-w" || arg == "--workspace")
            && i + 1 < all_raw.len()
            && !all_raw[i + 1].starts_with('-')
        {
            all.push(format!("{}={}", arg, all_raw[i + 1]));
            i += 2;
        } else {
            all.push(arg.clone());
            i += 1;
        }
    }

    if args.if_present {
        all.insert(0, "--if-present".to_string());
        let spec = resolve_command(agent, LogicalCommand::Run, &all)
            .ok_or_else(|| anyhow::anyhow!("run not supported"))?;
        return Ok(spec);
    }

    resolve_command(agent, LogicalCommand::Run, &all)
        .ok_or_else(|| anyhow::anyhow!("run not supported for {agent}"))
}

fn resolve_for_exec(agent: Agent, args: &ExecArgs) -> anyhow::Result<CommandSpec> {
    let command = args.command.clone().unwrap_or_default();
    if command.is_empty() {
        anyhow::bail!("No command specified for exec");
    }
    let mut all = vec![command];
    all.extend(args.args.clone());
    let cmd_type = if args.local {
        LogicalCommand::ExecuteLocal
    } else {
        LogicalCommand::Execute
    };
    resolve_command(agent, cmd_type, &all)
        .ok_or_else(|| anyhow::anyhow!("execute not supported for {agent}"))
}

fn resolve_for_dedupe(agent: Agent, args: &DedupeArgs) -> anyhow::Result<CommandSpec> {
    let mut extra = args.extra.clone();
    if args.check {
        let mapped = match agent {
            Agent::Npm => "--dry-run",
            Agent::Pnpm | Agent::PnpmAt6 | Agent::PnpmRush | Agent::Aube => "--check",
            _ => "--check",
        };
        extra.insert(0, mapped.to_string());
    }
    resolve_command(agent, LogicalCommand::Dedupe, &extra)
        .ok_or_else(|| anyhow::anyhow!("dedupe not supported for {agent}"))
}

fn resolve_for_clean(agent: Agent, args: &CleanArgs) -> anyhow::Result<CommandSpec> {
    let extra = args.extra.clone();
    resolve_command(agent, LogicalCommand::Frozen, &extra)
        .ok_or_else(|| anyhow::anyhow!("clean install not supported for {agent}"))
}
