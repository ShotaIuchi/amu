use std::path::Path;
use std::process::Command;

use crate::error::{DotlinkError, Result};

pub fn check_installed() -> Result<()> {
    let output = Command::new("which")
        .arg("stow")
        .output()
        .map_err(|_| DotlinkError::StowNotFound)?;

    if output.status.success() {
        Ok(())
    } else {
        Err(DotlinkError::StowNotFound)
    }
}

pub fn stow(source: &Path, target: &Path) -> Result<()> {
    run_stow(&[], source, target)
}

pub fn unstow(source: &Path, target: &Path) -> Result<()> {
    run_stow(&["-D"], source, target)
}

pub fn restow(source: &Path, target: &Path) -> Result<()> {
    run_stow(&["-R"], source, target)
}

pub fn dry_run(source: &Path, target: &Path) -> Result<String> {
    let (parent, dirname) = split_source_path(source)?;

    let output = Command::new("stow")
        .arg("-n")
        .arg("-v")
        .arg("--no-folding")
        .arg("-t")
        .arg(target)
        .arg("-d")
        .arg(&parent)
        .arg(&dirname)
        .output()
        .map_err(|e| DotlinkError::StowError(e.to_string()))?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(stderr)
}

fn run_stow(extra_args: &[&str], source: &Path, target: &Path) -> Result<()> {
    let (parent, dirname) = split_source_path(source)?;

    let mut cmd = Command::new("stow");
    cmd.arg("--no-folding");
    for arg in extra_args {
        cmd.arg(arg);
    }
    cmd.arg("-t").arg(target);
    cmd.arg("-d").arg(&parent);
    cmd.arg(&dirname);

    let output = cmd.output().map_err(|e| DotlinkError::StowError(e.to_string()))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(DotlinkError::StowError(stderr.to_string()))
    }
}

fn split_source_path(source: &Path) -> Result<(String, String)> {
    let parent = source
        .parent()
        .ok_or_else(|| DotlinkError::StowError("Invalid source path: no parent directory".into()))?
        .to_string_lossy()
        .to_string();

    let dirname = source
        .file_name()
        .ok_or_else(|| DotlinkError::StowError("Invalid source path: no directory name".into()))?
        .to_string_lossy()
        .to_string();

    Ok((parent, dirname))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_split_source_path() {
        let source = PathBuf::from("/home/user/dotfiles/nvim");
        let (parent, dirname) = split_source_path(&source).unwrap();
        assert_eq!(parent, "/home/user/dotfiles");
        assert_eq!(dirname, "nvim");
    }

    #[test]
    fn test_split_source_path_nested() {
        let source = PathBuf::from("/home/user/work/dotfiles/.config");
        let (parent, dirname) = split_source_path(&source).unwrap();
        assert_eq!(parent, "/home/user/work/dotfiles");
        assert_eq!(dirname, ".config");
    }
}
