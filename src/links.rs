use std::fs;
use std::path::{Path, PathBuf};

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

/// Remove only dangling symlinks from the given list. Returns count of removed links.
pub fn cleanup_dangling_links(target: &Path, recorded: &[PathBuf]) -> usize {
    let mut removed = 0;
    let mut dirs_to_check: Vec<PathBuf> = Vec::new();

    for relative in recorded {
        let full_path = target.join(relative);
        if full_path.is_symlink() && !full_path.exists()
            && fs::remove_file(&full_path).is_ok()
        {
            removed += 1;
            if let Some(parent) = full_path.parent() {
                if parent != target {
                    dirs_to_check.push(parent.to_path_buf());
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

fn remove_empty_dirs_up_to(dir: &Path, stop_at: &Path) {
    let mut current = dir.to_path_buf();
    while current != stop_at && current.starts_with(stop_at) {
        match fs::read_dir(&current) {
            Ok(mut entries) => {
                if entries.next().is_some() {
                    break;
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
