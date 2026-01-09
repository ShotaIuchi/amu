use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DotlinkError>;

#[derive(Error, Debug)]
pub enum DotlinkError {
    #[error("stow is not installed\n\nInstall with:\n  macOS:  brew install stow\n  Ubuntu: sudo apt install stow\n  Arch:   sudo pacman -S stow")]
    StowNotFound,

    #[error("Source directory does not exist: {0}")]
    SourceNotFound(PathBuf),

    #[error("Target directory does not exist: {0}")]
    TargetNotFound(PathBuf),

    #[error("Already registered: {src} -> {dest}")]
    AlreadyRegistered { src: PathBuf, dest: PathBuf },

    #[error("Not registered: {src} -> {dest}")]
    NotRegistered { src: PathBuf, dest: PathBuf },

    #[error("Failed to parse config file: {0}")]
    ConfigParseError(String),

    #[error("Failed to save config file: {0}")]
    ConfigSaveError(String),

    #[error("stow command failed: {0}")]
    StowError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
