//! Configuration management for conv.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Application configuration settings loaded from disk.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub jobs: Option<usize>,
    pub force: Option<bool>,
    pub keep_going: Option<bool>,
    pub output_dir: Option<PathBuf>,
    #[serde(default)]
    pub backend_paths: HashMap<String, PathBuf>,
}

impl Config {
    /// Loads configuration from `~/.config/conv/config.toml` or `$XDG_CONFIG_HOME/conv/config.toml`.
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| {
                if path.exists() {
                    Self::from_file(&path).ok()
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    /// Loads configuration from an explicit file path.
    pub fn from_file(path: &Path) -> Result<Self, anyhow::Error> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Resolves the default configuration file path.
    pub fn config_path() -> Option<PathBuf> {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg.is_empty() {
                return Some(PathBuf::from(xdg).join("conv").join("config.toml"));
            }
        }
        std::env::var("HOME").ok().map(|home| {
            PathBuf::from(home)
                .join(".config")
                .join("conv")
                .join("config.toml")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
jobs = 6
force = true
keep_going = true
output_dir = "/tmp/converted"

[backend_paths]
ffmpeg = "/usr/bin/ffmpeg"
magick = "/usr/bin/magick"
"#
        )
        .unwrap();

        let cfg = Config::from_file(file.path()).unwrap();
        assert_eq!(cfg.jobs, Some(6));
        assert_eq!(cfg.force, Some(true));
        assert_eq!(cfg.keep_going, Some(true));
        assert_eq!(cfg.output_dir, Some(PathBuf::from("/tmp/converted")));
        assert_eq!(
            cfg.backend_paths.get("ffmpeg"),
            Some(&PathBuf::from("/usr/bin/ffmpeg"))
        );
    }

    #[test]
    fn test_empty_config() {
        let file = NamedTempFile::new().unwrap();
        let cfg = Config::from_file(file.path()).unwrap();
        assert_eq!(cfg.jobs, None);
        assert_eq!(cfg.force, None);
    }
}
