//! Agent identity.
//!
//! The `Agent` enum is the single source of truth for package-manager
//! identity (including flavors such as `yarn@berry` and `pnpm@6`). All
//! display names, string parsing, and install-command helpers live here.

use std::fmt;
use std::str::FromStr;

use crate::error::{Result, ZpmError};

/// Supported package managers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Agent {
    Npm,
    Yarn,
    YarnBerry,
    Pnpm,
    PnpmAt6,
    PnpmRush,
    Bun,
    Deno,
    Aube,
    Nub,
}

impl Agent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Agent::Npm => "npm",
            Agent::Yarn => "yarn",
            Agent::YarnBerry => "yarn@berry",
            Agent::Pnpm => "pnpm",
            Agent::PnpmAt6 => "pnpm@6",
            Agent::PnpmRush => "pnpm-rush",
            Agent::Bun => "bun",
            Agent::Deno => "deno",
            Agent::Aube => "aube",
            Agent::Nub => "nub",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Agent::YarnBerry => "yarn (berry)",
            Agent::PnpmRush => "rush-pnpm",
            Agent::PnpmAt6 => "pnpm@6",
            _ => self.as_str(),
        }
    }

    pub fn all() -> &'static [Agent] {
        &[
            Agent::Npm,
            Agent::Yarn,
            Agent::YarnBerry,
            Agent::Pnpm,
            Agent::PnpmAt6,
            Agent::PnpmRush,
            Agent::Bun,
            Agent::Deno,
            Agent::Aube,
            Agent::Nub,
        ]
    }

    pub fn install_command(&self) -> &'static str {
        self.as_str()
            .split('@')
            .next()
            .expect("str::split always yields at least one element")
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Agent {
    type Err = ZpmError;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "npm" => Ok(Agent::Npm),
            "yarn" => Ok(Agent::Yarn),
            "yarn@berry" | "yarn-berry" | "berry" => Ok(Agent::YarnBerry),
            "pnpm" => Ok(Agent::Pnpm),
            "pnpm@6" | "pnpm6" => Ok(Agent::PnpmAt6),
            "pnpm-rush" | "rush-pnpm" | "rush" => Ok(Agent::PnpmRush),
            "bun" => Ok(Agent::Bun),
            "deno" => Ok(Agent::Deno),
            "aube" => Ok(Agent::Aube),
            "nub" => Ok(Agent::Nub),
            other => Err(ZpmError::UnknownAgent(other.to_string())),
        }
    }
}
