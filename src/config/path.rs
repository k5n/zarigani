use std::env;
use std::path::PathBuf;

use super::ConfigError;

pub fn default_config_path() -> Result<PathBuf, ConfigError> {
    let home_dir = env::home_dir().ok_or(ConfigError::HomeDirUnavailable)?;
    Ok(home_dir.join(".zarigani").join("config.toml"))
}
