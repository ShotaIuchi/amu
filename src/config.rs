use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{DotlinkError, Result};

const CONFIG_DIR: &str = "dotlink";
const CONFIG_FILE: &str = "config.yaml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub targets: BTreeMap<PathBuf, Vec<PathBuf>>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&path)?;
        let config: Config = serde_yaml::from_str(&content)
            .map_err(|e| DotlinkError::ConfigParseError(e.to_string()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)
            .map_err(|e| DotlinkError::ConfigSaveError(e.to_string()))?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn add_source(&mut self, target: PathBuf, source: PathBuf) -> Result<()> {
        let sources = self.targets.entry(target.clone()).or_default();
        if sources.contains(&source) {
            return Err(DotlinkError::AlreadyRegistered { src: source, dest: target });
        }
        sources.push(source);
        Ok(())
    }

    pub fn remove_source(&mut self, target: &Path, source: &Path) -> Result<()> {
        let sources = self.targets.get_mut(target).ok_or_else(|| {
            DotlinkError::NotRegistered {
                src: source.to_path_buf(),
                dest: target.to_path_buf(),
            }
        })?;

        let pos = sources.iter().position(|s| s == source).ok_or_else(|| {
            DotlinkError::NotRegistered {
                src: source.to_path_buf(),
                dest: target.to_path_buf(),
            }
        })?;

        sources.remove(pos);

        if sources.is_empty() {
            self.targets.remove(target);
        }

        Ok(())
    }

    pub fn get_sources(&self, target: &Path) -> Option<&Vec<PathBuf>> {
        self.targets.get(target)
    }

    fn config_path() -> Result<PathBuf> {
        if let Ok(path) = std::env::var("DOTLINK_CONFIG") {
            return Ok(PathBuf::from(path));
        }
        let home = dirs::home_dir()
            .ok_or_else(|| DotlinkError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find home directory",
            )))?;
        Ok(home.join(".config").join(CONFIG_DIR).join(CONFIG_FILE))
    }
}

pub fn expand_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    let expanded = shellexpand::tilde(&path_str);
    PathBuf::from(expanded.as_ref())
}

pub fn normalize_path(path: &Path) -> Result<PathBuf> {
    let expanded = expand_path(path);
    expanded.canonicalize().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            DotlinkError::SourceNotFound(expanded)
        } else {
            DotlinkError::IoError(e)
        }
    })
}

pub fn resolve_target(target: Option<PathBuf>) -> Result<PathBuf> {
    match target {
        Some(t) => {
            let expanded = expand_path(&t);
            expanded.canonicalize().map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    DotlinkError::TargetNotFound(expanded)
                } else {
                    DotlinkError::IoError(e)
                }
            })
        }
        None => std::env::current_dir().map_err(DotlinkError::IoError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_expand_path_with_tilde() {
        let home = dirs::home_dir().unwrap();
        let path = Path::new("~/.config");
        let expanded = expand_path(path);
        assert_eq!(expanded, home.join(".config"));
    }

    #[test]
    fn test_expand_path_without_tilde() {
        let path = Path::new("/usr/local/bin");
        let expanded = expand_path(path);
        assert_eq!(expanded, PathBuf::from("/usr/local/bin"));
    }

    #[test]
    fn test_config_add_and_remove_source() {
        let mut config = Config::default();
        let target = PathBuf::from("/home/user/.config");
        let source = PathBuf::from("/home/user/dotfiles/config");

        config.add_source(target.clone(), source.clone()).unwrap();
        assert_eq!(config.targets.get(&target).unwrap(), &vec![source.clone()]);

        config.remove_source(&target, &source).unwrap();
        assert!(config.targets.get(&target).is_none());
    }

    #[test]
    fn test_config_add_duplicate_source() {
        let mut config = Config::default();
        let target = PathBuf::from("/home/user/.config");
        let source = PathBuf::from("/home/user/dotfiles/config");

        config.add_source(target.clone(), source.clone()).unwrap();
        let result = config.add_source(target, source);
        assert!(matches!(result, Err(DotlinkError::AlreadyRegistered { .. })));
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let mut config = Config::default();
        config.targets.insert(
            PathBuf::from("/home/user/.config"),
            vec![PathBuf::from("/home/user/dotfiles/config")],
        );

        let content = serde_yaml::to_string(&config).unwrap();
        fs::write(&config_path, &content).unwrap();

        let loaded_content = fs::read_to_string(&config_path).unwrap();
        let loaded: Config = serde_yaml::from_str(&loaded_content).unwrap();
        assert_eq!(loaded.targets, config.targets);
    }
}
