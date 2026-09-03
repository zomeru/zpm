use std::path::PathBuf;

use clap::Parser;
use zpm::{app, cli::Cli, config};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cfg = config::load_config();
    let code = app::run(cli, cfg, cwd)?;
    std::process::exit(code);
}
