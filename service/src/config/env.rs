use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    net::SocketAddr,
};

use config::{Config, Environment};
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub api_bind: String,
    pub n2yo_api_key: String,
    pub n2yo_base_url: String,
    pub n2yo_timeout_seconds: u64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppConfigError> {
        dotenvy::dotenv().ok();

        let settings = Config::builder()
            .set_default("api_bind", "0.0.0.0:8080")
            .map_err(|source| AppConfigError::InvalidEnvironment(source.to_string()))?
            .set_default("n2yo_api_key", "")
            .map_err(|source| AppConfigError::InvalidEnvironment(source.to_string()))?
            .set_default("n2yo_base_url", "https://api.n2yo.com/rest/v1/satellite")
            .map_err(|source| AppConfigError::InvalidEnvironment(source.to_string()))?
            .set_default("n2yo_timeout_seconds", 10)
            .map_err(|source| AppConfigError::InvalidEnvironment(source.to_string()))?
            .add_source(Environment::default())
            .build()
            .map_err(|source| AppConfigError::InvalidEnvironment(source.to_string()))?;

        let raw: RawAppConfig = settings
            .try_deserialize()
            .map_err(|source| AppConfigError::InvalidEnvironment(source.to_string()))?;

        raw.try_into()
    }
}

#[derive(Debug, Deserialize)]
struct RawAppConfig {
    api_bind: String,
    n2yo_api_key: String,
    n2yo_base_url: String,
    n2yo_timeout_seconds: u64,
}

impl TryFrom<RawAppConfig> for AppConfig {
    type Error = AppConfigError;

    fn try_from(raw: RawAppConfig) -> Result<Self, Self::Error> {
        raw.api_bind
            .parse::<SocketAddr>()
            .map_err(|_| AppConfigError::InvalidField {
                field: "API_BIND",
                message: "must be a valid socket address such as 0.0.0.0:8080".to_owned(),
            })?;

        if raw.n2yo_api_key.trim().is_empty() {
            return Err(AppConfigError::InvalidField {
                field: "N2YO_API_KEY",
                message: "must not be empty".to_owned(),
            });
        }

        if raw.n2yo_base_url.trim().is_empty() {
            return Err(AppConfigError::InvalidField {
                field: "N2YO_BASE_URL",
                message: "must not be empty".to_owned(),
            });
        }

        if !raw.n2yo_base_url.starts_with("http://") && !raw.n2yo_base_url.starts_with("https://") {
            return Err(AppConfigError::InvalidField {
                field: "N2YO_BASE_URL",
                message: "must start with http:// or https://".to_owned(),
            });
        }

        if raw.n2yo_timeout_seconds == 0 {
            return Err(AppConfigError::InvalidField {
                field: "N2YO_TIMEOUT_SECONDS",
                message: "must be greater than 0".to_owned(),
            });
        }

        Ok(AppConfig {
            api_bind: raw.api_bind,
            n2yo_api_key: raw.n2yo_api_key,
            n2yo_base_url: raw.n2yo_base_url,
            n2yo_timeout_seconds: raw.n2yo_timeout_seconds,
        })
    }
}

#[derive(Debug)]
pub enum AppConfigError {
    InvalidEnvironment(String),
    InvalidField {
        field: &'static str,
        message: String,
    },
}

impl Display for AppConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEnvironment(message) => {
                write!(f, "failed to deserialize environment: {}", message)
            }
            Self::InvalidField { field, message } => {
                write!(f, "invalid {}: {}", field, message)
            }
        }
    }
}

impl Error for AppConfigError {}
