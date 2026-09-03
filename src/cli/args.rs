//! Clap definitions — user intent types.
//!
//! These types describe what the user typed. They contain no business logic
//! and are not passed deep into the application beyond `resolve_cli`. That
//! keeps CLI parsing (clap) isolated from domain translation.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "zpm", version, about = "Fast package-manager abstraction CLI — one CLI, detects the manager, translates intent", long_about = None)]
pub struct Cli {
    /// Override detected package manager (npm, pnpm, yarn, yarn@berry, bun, deno, aube, nub)
    #[arg(long = "pm", global = true, value_name = "MANAGER")]
    pub pm: Option<String>,

    /// Change working directory
    #[arg(long = "cwd", global = true, value_name = "DIR")]
    pub cwd: Option<PathBuf>,

    /// Change directory (shorthand, like ni's -C)
    #[arg(short = 'C', global = true, value_name = "DIR")]
    pub directory: Option<PathBuf>,

    /// Print what would be executed without running it
    #[arg(long = "dry-run", global = true)]
    pub dry_run: bool,

    /// Verbose output (show detection and command)
    #[arg(long, short, global = true)]
    pub verbose: bool,

    /// Disable interactive prompts
    #[arg(long = "no-interactive", global = true)]
    pub no_interactive: bool,

    /// Operate at workspace root
    #[arg(long = "root", global = true)]
    pub root: bool,

    /// Enable / disable color
    #[arg(long = "color", global = true, value_name = "WHEN")]
    pub color: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Arguments for install/add when no subcommand is given (like `zpm react`)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    pub trailing: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Install all dependencies (auto-detects frozen if needed)
    #[command(alias = "i")]
    Install(InstallArgs),

    /// Add dependencies
    #[command(alias = "a")]
    Add(AddArgs),

    /// Remove dependencies
    #[command(alias = "rm", alias = "uninstall")]
    Remove(RemoveArgs),

    /// Update dependencies
    #[command(alias = "up", alias = "upgrade")]
    Update(UpdateArgs),

    /// Run a package.json script
    #[command(alias = "r")]
    Run(RunArgs),

    /// Execute a binary (npx / dlx / bunx)
    #[command(alias = "x", alias = "dlx")]
    Exec(ExecArgs),

    /// Deduplicate dependencies
    Dedupe(DedupeArgs),

    /// Clean install (frozen lockfile)
    #[command(alias = "ci", alias = "clean-install")]
    Clean(CleanArgs),

    /// Print detected package manager
    Agent(AgentArgs),

    /// Show help for all commands (hidden alias for completion)
    #[command(hide = true)]
    Completion(CompletionArgs),
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Install with frozen lockfile
    #[arg(long = "frozen", conflicts_with = "frozen_if_present")]
    pub frozen: bool,

    /// Frozen only if lockfile exists
    #[arg(long = "frozen-if-present")]
    pub frozen_if_present: bool,

    /// Production install (omit dev dependencies)
    #[arg(long = "production", short = 'P', alias = "prod")]
    pub production: bool,

    /// Global install (rare)
    #[arg(long = "global", short = 'g')]
    pub global: bool,

    #[arg(last = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Packages to add
    #[arg(required = true, value_name = "PACKAGES")]
    pub packages: Vec<String>,

    /// Add as dev dependency
    #[arg(long = "dev", short = 'D', alias = "save-dev", visible_alias = "D")]
    pub dev: bool,

    /// Add as peer dependency
    #[arg(long = "peer", alias = "save-peer")]
    pub peer: bool,

    /// Add as optional dependency
    #[arg(long = "optional", alias = "save-optional")]
    pub optional: bool,

    /// Exact version
    #[arg(long = "exact", short = 'E')]
    pub exact: bool,

    /// Global
    #[arg(long = "global", short = 'g')]
    pub global: bool,

    /// Production vs dev? forwarded
    #[arg(long = "frozen")]
    pub frozen: bool,

    /// Workspace root
    #[arg(long = "workspace", short = 'w')]
    pub workspace: bool,

    #[arg(last = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

#[derive(Debug, Args)]
pub struct RemoveArgs {
    #[arg(required = false, value_name = "PACKAGES")]
    pub packages: Vec<String>,

    #[arg(long = "global", short = 'g')]
    pub global: bool,

    /// Interactive multi-select when no packages given
    #[arg(long = "interactive", short = 'i')]
    pub interactive: bool,

    #[arg(last = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

#[derive(Debug, Args)]
pub struct UpdateArgs {
    #[arg(value_name = "PACKAGES")]
    pub packages: Vec<String>,

    #[arg(long = "interactive", short = 'i')]
    pub interactive: bool,

    #[arg(long = "latest")]
    pub latest: bool,

    #[arg(long = "global", short = 'g')]
    pub global: bool,

    #[arg(last = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    /// Script name (if omitted, interactive picker)
    #[arg(allow_hyphen_values = true)]
    pub script: Option<String>,

    /// Args forwarded to script (including workspace flags like -w packages/foo)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,

    /// Run even if script not present
    #[arg(long = "if-present")]
    pub if_present: bool,

    /// Prompt for package in monorepo
    #[arg(long = "package", short = 'p')]
    pub package: bool,
}

#[derive(Debug, Args)]
pub struct ExecArgs {
    /// Binary to execute
    pub command: Option<String>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,

    /// Prefer local binary
    #[arg(long = "local")]
    pub local: bool,
}

#[derive(Debug, Args)]
pub struct DedupeArgs {
    #[arg(long = "check", short = 'c')]
    pub check: bool,

    #[arg(last = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

#[derive(Debug, Args)]
pub struct CleanArgs {
    #[arg(last = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

#[derive(Debug, Args)]
pub struct AgentArgs {}

#[derive(Debug, Args)]
pub struct CompletionArgs {
    #[arg(long)]
    pub bash: bool,
    #[arg(long)]
    pub zsh: bool,
    #[arg(long)]
    pub fish: bool,
}
