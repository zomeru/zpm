//! CLI layer — clap parsing plus domain translation.
//!
//! `cli::args` owns the `clap` types (user intent). `cli::resolve` converts
//! that intent into a `CommandSpec` without leaking clap types into the
//! domain. This keeps the dependency direction `CLI → domain` clear.

pub mod args;
pub mod resolve;

pub use args::{
    AddArgs, AgentArgs, CleanArgs, Cli, Commands, CompletionArgs, DedupeArgs, ExecArgs,
    InstallArgs, RemoveArgs, RunArgs, UpdateArgs,
};
pub use resolve::resolve_cli;
