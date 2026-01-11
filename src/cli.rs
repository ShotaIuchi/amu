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

        /// Show what would be done without making changes
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Remove symlinks and unregister a source directory
    Remove {
        /// Source directory to unlink
        source: PathBuf,

        /// Target directory (defaults to current directory)
        target: Option<PathBuf>,

        /// Show what would be done without making changes
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Reapply registered sources for a target
    Update {
        /// Target directory to update (defaults to current directory)
        target: Option<PathBuf>,

        /// Update all targets
        #[arg(long)]
        all: bool,

        /// Update all targets that reference this source
        #[arg(short, long)]
        source: Option<PathBuf>,

        /// Show what would be done without making changes
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Restore links from configuration (for new machine setup)
    Restore {
        /// Target directory to restore (defaults to current directory)
        target: Option<PathBuf>,

        /// Restore all targets
        #[arg(long)]
        all: bool,

        /// Show what would be done without making changes
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// List registered sources
    List {
        /// Target directory to list (defaults to current directory)
        target: Option<PathBuf>,

        /// List all targets
        #[arg(long)]
        all: bool,

        /// Show actual symlinks
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show status of registered links
    Status {
        /// Target directory to check (defaults to current directory)
        target: Option<PathBuf>,

        /// Check all targets
        #[arg(long)]
        all: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Remove symlinks and clear configuration
    Clear {
        /// Target directory to clear (defaults to current directory)
        target: Option<PathBuf>,

        /// Clear all targets
        #[arg(long)]
        all: bool,

        /// Show what would be done without making changes
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
}
