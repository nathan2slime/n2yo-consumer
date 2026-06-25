use std::{error::Error, fmt, time::Duration};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utoipa::ToSchema;

#[derive(Clone)]
pub struct OpenMeteoClient {
    http: reqwest::Client,
    elevation_url: String,
}

impl OpenMeteoClient {
    pub fn new(elevation_url: impl Into<String>, timeout_seconds: u64) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .map_err(|error| format!("failed to build Open-Meteo HTTP client: {error}"))?;

        Ok(Self {
            http,
            elevation_url: elevation_url.into(),
        })
    }

    pub async fn elevation(
        &self,
        latitude: f64,
        longitude: f64,
    ) -> Result<ElevationResponse, OpenMeteoClientError> {
        let upstream: OpenMeteoElevationResponse = self
            .get_json(&[("latitude", latitude), ("longitude", longitude)])
            .await?;

        let altitude_m = upstream
            .elevation
            .first()
            .copied()
            .flatten()
            .ok_or(OpenMeteoClientError::MissingElevation)?;

        Ok(ElevationResponse {
            latitude,
            longitude,
            altitude_m,
            source: "open-meteo".to_owned(),
            resolution_m: 90,
        })
    }

    async fn get_json<T>(&self, query: &[(&str, f64)]) -> Result<T, OpenMeteoClientError>
    where
        T: DeserializeOwned,
    {
        let response = self
            .http
            .get(&self.elevation_url)
            .query(query)
            .send()
            .await
            .map_err(|error| OpenMeteoClientError::RequestFailed(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| OpenMeteoClientError::RequestFailed(error.to_string()))?;

        if !status.is_success() {
            return Err(OpenMeteoClientError::UpstreamStatus { status, body });
        }

        serde_json::from_str(&body).map_err(|error| OpenMeteoClientError::InvalidResponse {
            message: error.to_string(),
            body,
        })
    }
}

#[derive(Debug)]
pub enum OpenMeteoClientError {
    RequestFailed(String),
    UpstreamStatus { status: StatusCode, body: String },
    InvalidResponse { message: String, body: String },
    MissingElevation,
}

impl fmt::Display for OpenMeteoClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestFailed(message) => write!(f, "Open-Meteo request failed: {message}"),
            Self::UpstreamStatus { status, body } => {
                write!(f, "Open-Meteo returned HTTP {status}: {body}")
            }
            Self::InvalidResponse { message, body } => {
                write!(
                    f,
                    "Open-Meteo response did not match the expected schema: {message}; body: {body}"
                )
            }
            Self::MissingElevation => write!(f, "Open-Meteo response did not include elevation"),
        }
    }
}

impl Error for OpenMeteoClientError {}

#[derive(Debug, Serialize, ToSchema)]
pub struct ElevationResponse {
    #[schema(example = 52.52)]
    pub latitude: f64,
    #[schema(example = 13.41)]
    pub longitude: f64,
    #[schema(example = 38.0)]
    pub altitude_m: f64,
    #[schema(example = "open-meteo")]
    pub source: String,
    #[schema(example = 90)]
    pub resolution_m: u16,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoElevationResponse {
    elevation: Vec<Option<f64>>,
}
