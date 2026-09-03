pub mod app;
pub mod catalog;
pub mod cli;
pub mod config;
pub mod detection;
pub mod error;
pub(crate) mod fs;
pub mod package_manager;
pub mod process;
pub mod ui;
pub mod workspace;

pub use error::{Result, ZpmError};
