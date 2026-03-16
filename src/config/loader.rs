use std::fs;
use std::path::Path;

use crate::config::{AppConfig, ConfigError, default_config_path};

pub fn load_default() -> Result<AppConfig, ConfigError> {
    let path = default_config_path()?;
    load_from_path(&path)
}

pub fn load_from_path(path: &Path) -> Result<AppConfig, ConfigError> {
    let raw = fs::read_to_string(path).map_err(|source| ConfigError::ReadFile {
        path: path.display().to_string(),
        source,
    })?;

    let config: AppConfig = toml::from_str(&raw).map_err(ConfigError::InvalidToml)?;
    config.validate()?;
    Ok(config)
}
