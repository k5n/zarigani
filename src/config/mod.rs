use std::io;

use thiserror::Error;

mod loader;
mod path;
mod types;

pub use loader::{load_default, load_from_path};
pub use path::default_config_path;
pub use types::AppConfig;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to resolve home directory")]
    HomeDirUnavailable,

    #[error("failed to read config file `{path}`: {source}")]
    ReadFile {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("invalid config format: {0}")]
    InvalidToml(#[from] toml::de::Error),

    #[error("invalid config: {0}")]
    Validation(String),
}
