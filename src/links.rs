use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{DotlinkError, Result};

const LINKS_FILE: &str = "links.yaml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LinksRecord {
    #[serde(default)]
    pub targets: BTreeMap<PathBuf, BTreeMap<PathBuf, Vec<PathBuf>>>,
}

impl LinksRecord {
    pub fn load() -> Result<Self> {
        let path = Self::links_path()?;
        if !path.exists() {
            return Ok(LinksRecord::default());
        }

        let content = fs::read_to_string(&path)?;
        let record: LinksRecord = serde_yaml::from_str(&content)
            .map_err(|e| DotlinkError::LinksParseError(e.to_string()))?;
        Ok(record)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::links_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)
            .map_err(|e| DotlinkError::LinksSaveError(e.to_string()))?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn links_path() -> Result<PathBuf> {
        if let Ok(config_path) = std::env::var("AMU_CONFIG") {
            let config = PathBuf::from(config_path);
            if let Some(parent) = config.parent() {
                return Ok(parent.join(LINKS_FILE));
            }
        }
        let home = dirs::home_dir().ok_or_else(|| {
            DotlinkError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find home directory",
            ))
        })?;
        Ok(home.join(".config").join("amu").join(LINKS_FILE))
    }

    pub fn record_links(&mut self, target: &Path, source: &Path, links: Vec<PathBuf>) {
        let sources = self.targets.entry(target.to_path_buf()).or_default();
        sources.insert(source.to_path_buf(), links);
    }

    pub fn remove_source(&mut self, target: &Path, source: &Path) -> Option<Vec<PathBuf>> {
        let sources = self.targets.get_mut(target)?;
        let removed = sources.remove(source);
        if sources.is_empty() {
            self.targets.remove(target);
        }
        removed
    }

    pub fn remove_target(&mut self, target: &Path) {
        self.targets.remove(target);
    }

    pub fn get_links(&self, target: &Path, source: &Path) -> Option<&Vec<PathBuf>> {
        self.targets.get(target)?.get(source)
    }
}

/// Scan target directory for all dangling symlinks.
/// Returns target-relative paths of dangling links.
pub fn find_dangling_links(target: &Path) -> Vec<PathBuf> {
    let mut dangling = Vec::new();
    find_dangling_recursive(target, target, &mut dangling);
    dangling.sort();
    dangling
}

fn find_dangling_recursive(target_base: &Path, current: &Path, dangling: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_symlink() {
            // Dangling: symlink exists but target does not
            if !path.exists() {
                if let Ok(relative) = path.strip_prefix(target_base) {
                    dangling.push(relative.to_path_buf());
                }
            }
        } else if path.is_dir() {
            find_dangling_recursive(target_base, &path, dangling);
        }
    }
}

/// Scan target directory for symlinks pointing to source, returning target-relative paths.
pub fn scan_links_for_source(target: &Path, source: &Path) -> Vec<PathBuf> {
    let mut links = Vec::new();
    scan_links_recursive(target, source, target, &mut links);
    links.sort();
    links
}

fn scan_links_recursive(target_base: &Path, source: &Path, current: &Path, links: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_symlink() {
            if let Ok(link_target) = fs::read_link(&path) {
                let resolved = if link_target.is_absolute() {
                    link_target.clone()
                } else {
                    path.parent().unwrap_or(current).join(&link_target)
                };
                let abs_target = resolved.canonicalize().unwrap_or(resolved);
                if abs_target.starts_with(source) {
                    if let Ok(relative) = path.strip_prefix(target_base) {
                        links.push(relative.to_path_buf());
                    }
                }
            }
        } else if path.is_dir() {
            scan_links_recursive(target_base, source, &path, links);
        }
    }
}

/// Remove only dangling symlinks from recorded links. Returns count of removed links.
pub fn cleanup_dangling_links(target: &Path, recorded: &[PathBuf]) -> usize {
    let mut removed = 0;
    let mut dirs_to_check: Vec<PathBuf> = Vec::new();

    for relative in recorded {
        let full_path = target.join(relative);
        if full_path.is_symlink() && !full_path.exists() {
            // Symlink exists but target does not — dangling
            if fs::remove_file(&full_path).is_ok() {
                removed += 1;
                if let Some(parent) = full_path.parent() {
                    if parent != target {
                        dirs_to_check.push(parent.to_path_buf());
                    }
                }
            }
        }
    }

    // Remove empty directories depth-first
    dirs_to_check.sort_by_key(|b| std::cmp::Reverse(b.components().count()));
    dirs_to_check.dedup();

    for dir in &dirs_to_check {
        remove_empty_dirs_up_to(dir, target);
    }

    removed
}

/// Remove recorded symlinks that are dangling or point to source. Returns count of removed links.
pub fn cleanup_recorded_links(target: &Path, source: &Path, recorded: &[PathBuf]) -> usize {
    let mut removed = 0;

    // Collect directories that may become empty
    let mut dirs_to_check: Vec<PathBuf> = Vec::new();

    for relative in recorded {
        let full_path = target.join(relative);
        if full_path.is_symlink() {
            // Check if dangling or pointing to source
            let should_remove = if let Ok(link_target) = fs::read_link(&full_path) {
                let resolved = if link_target.is_absolute() {
                    link_target.clone()
                } else {
                    full_path.parent().unwrap_or(target).join(&link_target)
                };
                // Dangling (canonicalize fails) or points to source
                match resolved.canonicalize() {
                    Ok(abs) => abs.starts_with(source),
                    Err(_) => true, // dangling
                }
            } else {
                false
            };

            if should_remove && fs::remove_file(&full_path).is_ok() {
                removed += 1;
                if let Some(parent) = full_path.parent() {
                    if parent != target {
                        dirs_to_check.push(parent.to_path_buf());
                    }
                }
            }
        }
    }

    // Remove empty directories depth-first
    // Sort by path length descending to process deepest directories first
    dirs_to_check.sort_by_key(|b| std::cmp::Reverse(b.components().count()));
    dirs_to_check.dedup();

    for dir in &dirs_to_check {
        remove_empty_dirs_up_to(dir, target);
    }

    removed
}

fn remove_empty_dirs_up_to(dir: &Path, stop_at: &Path) {
    let mut current = dir.to_path_buf();
    while current != stop_at && current.starts_with(stop_at) {
        // Only remove if truly empty
        match fs::read_dir(&current) {
            Ok(mut entries) => {
                if entries.next().is_some() {
                    break; // not empty
                }
            }
            Err(_) => break,
        }
        if fs::remove_dir(&current).is_err() {
            break;
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_links_record_default() {
        let record = LinksRecord::default();
        assert!(record.targets.is_empty());
    }

    #[test]
    fn test_record_and_get_links() {
        let mut record = LinksRecord::default();
        let target = PathBuf::from("/home/user/.config");
        let source = PathBuf::from("/home/user/dotfiles/config");
        let links = vec![PathBuf::from("file1.txt"), PathBuf::from("sub/file2.txt")];

        record.record_links(&target, &source, links.clone());
        assert_eq!(record.get_links(&target, &source), Some(&links));
    }

    #[test]
    fn test_remove_source_returns_links() {
        let mut record = LinksRecord::default();
        let target = PathBuf::from("/home/user/.config");
        let source = PathBuf::from("/home/user/dotfiles/config");
        let links = vec![PathBuf::from("file1.txt")];

        record.record_links(&target, &source, links.clone());
        let removed = record.remove_source(&target, &source);
        assert_eq!(removed, Some(links));
        assert!(record.targets.is_empty());
    }

    #[test]
    fn test_remove_source_keeps_other_sources() {
        let mut record = LinksRecord::default();
        let target = PathBuf::from("/home/user/.config");
        let source1 = PathBuf::from("/home/user/dotfiles/config1");
        let source2 = PathBuf::from("/home/user/dotfiles/config2");

        record.record_links(&target, &source1, vec![PathBuf::from("a.txt")]);
        record.record_links(&target, &source2, vec![PathBuf::from("b.txt")]);

        record.remove_source(&target, &source1);
        assert!(record.get_links(&target, &source1).is_none());
        assert!(record.get_links(&target, &source2).is_some());
    }

    #[test]
    fn test_remove_target() {
        let mut record = LinksRecord::default();
        let target = PathBuf::from("/home/user/.config");
        let source = PathBuf::from("/home/user/dotfiles/config");

        record.record_links(&target, &source, vec![PathBuf::from("file.txt")]);
        record.remove_target(&target);
        assert!(record.targets.is_empty());
    }

    #[test]
    fn test_scan_links_for_source() {
        let temp = TempDir::new().unwrap();
        // Canonicalize to handle macOS /tmp -> /private/tmp
        let base = temp.path().canonicalize().unwrap();
        let target = base.join("target");
        let source = base.join("source");

        fs::create_dir(&target).unwrap();
        fs::create_dir(&source).unwrap();
        fs::write(source.join("file1.txt"), "hello").unwrap();
        fs::create_dir(source.join("sub")).unwrap();
        fs::write(source.join("sub").join("file2.txt"), "world").unwrap();

        // Create symlinks
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source.join("file1.txt"), target.join("file1.txt")).unwrap();
            fs::create_dir(target.join("sub")).unwrap();
            std::os::unix::fs::symlink(
                source.join("sub").join("file2.txt"),
                target.join("sub").join("file2.txt"),
            )
            .unwrap();
        }

        let links = scan_links_for_source(&target, &source);
        assert_eq!(links.len(), 2);
        assert!(links.contains(&PathBuf::from("file1.txt")));
        assert!(links.contains(&PathBuf::from("sub/file2.txt")));
    }

    #[test]
    fn test_cleanup_recorded_links_removes_dangling() {
        let temp = TempDir::new().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let target = base.join("target");
        let source = base.join("source");

        fs::create_dir(&target).unwrap();
        fs::create_dir(&source).unwrap();
        fs::write(source.join("file1.txt"), "hello").unwrap();

        // Create symlink then delete source file to make it dangling
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source.join("file1.txt"), target.join("file1.txt")).unwrap();
            fs::remove_file(source.join("file1.txt")).unwrap();
        }

        let recorded = vec![PathBuf::from("file1.txt")];
        let removed = cleanup_recorded_links(&target, &source, &recorded);

        assert_eq!(removed, 1);
        assert!(!target.join("file1.txt").exists());
    }

    #[test]
    fn test_cleanup_removes_empty_dirs() {
        let temp = TempDir::new().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let target = base.join("target");
        let source = base.join("source");

        fs::create_dir(&target).unwrap();
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("sub")).unwrap();
        fs::write(source.join("sub").join("file.txt"), "hello").unwrap();

        // Create symlink in subdirectory
        #[cfg(unix)]
        {
            fs::create_dir(target.join("sub")).unwrap();
            std::os::unix::fs::symlink(
                source.join("sub").join("file.txt"),
                target.join("sub").join("file.txt"),
            )
            .unwrap();
            fs::remove_file(source.join("sub").join("file.txt")).unwrap();
        }

        let recorded = vec![PathBuf::from("sub/file.txt")];
        let removed = cleanup_recorded_links(&target, &source, &recorded);

        assert_eq!(removed, 1);
        assert!(!target.join("sub").exists()); // empty dir also removed
    }

    #[test]
    fn test_save_and_load() {
        let temp = TempDir::new().unwrap();
        let links_path = temp.path().join("links.yaml");

        // Set AMU_CONFIG so links_path resolves to our temp dir
        let config_path = temp.path().join("config.yaml");
        std::env::set_var("AMU_CONFIG", &config_path);

        let mut record = LinksRecord::default();
        record.record_links(
            &PathBuf::from("/home/user/.config"),
            &PathBuf::from("/home/user/dotfiles/config"),
            vec![PathBuf::from("file1.txt"), PathBuf::from("sub/file2.txt")],
        );

        record.save().unwrap();

        // Verify file exists
        assert!(links_path.exists());

        // Load and verify
        let loaded = LinksRecord::load().unwrap();
        assert_eq!(loaded.targets, record.targets);

        // Clean up
        std::env::remove_var("AMU_CONFIG");
    }
}
