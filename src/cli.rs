use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "amu")]
#[command(about = "Merge multiple sources into one target with symlinks using stow", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Register a source directory and create symlinks
    Add {
        /// Source directory to link from
        source: PathBuf,

        /// Target directory to link to (defaults to current directory)
        target: Option<PathBuf>,
    },

    /// Remove symlinks and unregister a source directory
    Remove {
        /// Source directory to unlink
        source: PathBuf,

        /// Target directory (defaults to current directory)
        target: Option<PathBuf>,
    },

    /// Reapply all registered sources for a target
    Update {
        /// Target directory to update (defaults to all targets)
        target: Option<PathBuf>,
    },

    /// List registered sources
    List {
        /// Target directory to list (defaults to all targets)
        target: Option<PathBuf>,
    },

    /// Show status of all registered links
    Status,

    /// Remove all symlinks and clear configuration
    Clear,
}
