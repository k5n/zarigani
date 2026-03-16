use serde::Deserialize;

use super::ConfigError;
use crate::providers::openai::OpenAiCompatibleProviderConfig;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub provider: ProviderSection,
}

#[derive(Debug, Deserialize)]
pub struct ProviderSection {
    pub openai_compatible: OpenAiCompatibleProviderConfig,
}

impl AppConfig {
    pub fn load_default() -> Result<Self, ConfigError> {
        crate::config::load_default()
    }

    pub fn load_from_path(path: &std::path::Path) -> Result<Self, ConfigError> {
        crate::config::load_from_path(path)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.provider.openai_compatible.base_url.trim().is_empty() {
            return Err(ConfigError::Validation(
                "provider.openai_compatible.base_url must not be empty".to_string(),
            ));
        }

        if self.provider.openai_compatible.model.trim().is_empty() {
            return Err(ConfigError::Validation(
                "provider.openai_compatible.model must not be empty".to_string(),
            ));
        }

        if let Some(temperature) = self.provider.openai_compatible.temperature {
            if !temperature.is_finite() || !(0.0..=2.0).contains(&temperature) {
                return Err(ConfigError::Validation(format!(
                    "provider.openai_compatible.temperature is out of range: {}",
                    temperature
                )));
            }
        }

        if let Some(max_tokens) = self.provider.openai_compatible.max_tokens {
            if max_tokens == 0 {
                return Err(ConfigError::Validation(
                    "provider.openai_compatible.max_tokens must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }
}
