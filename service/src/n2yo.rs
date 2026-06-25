use std::{error::Error, fmt, time::Duration};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utoipa::ToSchema;

#[derive(Clone)]
pub struct N2yoClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl N2yoClient {
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        timeout_seconds: u64,
    ) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .map_err(|error| format!("failed to build N2YO HTTP client: {error}"))?;

        Ok(Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            api_key: api_key.into(),
        })
    }

    pub async fn tle(&self, id: u32) -> Result<TleResponse, N2yoClientError> {
        self.get_json(&format!("tle/{id}")).await
    }

    pub async fn positions(
        &self,
        id: u32,
        observer_lat: f64,
        observer_lng: f64,
        observer_alt: f64,
        seconds: u16,
    ) -> Result<PositionsResponse, N2yoClientError> {
        self.get_json(&format!(
            "positions/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{seconds}/"
        ))
        .await
    }

    pub async fn visual_passes(
        &self,
        id: u32,
        observer_lat: f64,
        observer_lng: f64,
        observer_alt: f64,
        days: u8,
        min_visibility: u32,
    ) -> Result<VisualPassesResponse, N2yoClientError> {
        self.get_json(&format!(
            "visualpasses/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{days}/{min_visibility}/"
        ))
        .await
    }

    pub async fn radio_passes(
        &self,
        id: u32,
        observer_lat: f64,
        observer_lng: f64,
        observer_alt: f64,
        days: u8,
        min_elevation: u8,
    ) -> Result<RadioPassesResponse, N2yoClientError> {
        self.get_json(&format!(
            "radiopasses/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{days}/{min_elevation}/"
        ))
        .await
    }

    pub async fn above(
        &self,
        observer_lat: f64,
        observer_lng: f64,
        observer_alt: f64,
        search_radius: u8,
        category_id: u32,
    ) -> Result<AboveResponse, N2yoClientError> {
        self.get_json(&format!(
            "above/{observer_lat}/{observer_lng}/{observer_alt}/{search_radius}/{category_id}/"
        ))
        .await
    }

    async fn get_json<T>(&self, path: &str) -> Result<T, N2yoClientError>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}/{}&apiKey={}", self.base_url, path, self.api_key);
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|error| N2yoClientError::RequestFailed(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| N2yoClientError::RequestFailed(error.to_string()))?;

        if !status.is_success() {
            return Err(N2yoClientError::UpstreamStatus { status, body });
        }

        serde_json::from_str(&body).map_err(|error| N2yoClientError::InvalidResponse {
            message: error.to_string(),
            body,
        })
    }
}

#[derive(Debug)]
pub enum N2yoClientError {
    RequestFailed(String),
    UpstreamStatus { status: StatusCode, body: String },
    InvalidResponse { message: String, body: String },
}

impl fmt::Display for N2yoClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestFailed(message) => write!(f, "N2YO request failed: {message}"),
            Self::UpstreamStatus { status, body } => {
                write!(f, "N2YO returned HTTP {status}: {body}")
            }
            Self::InvalidResponse { message, body } => {
                write!(
                    f,
                    "N2YO response did not match the expected schema: {message}; body: {body}"
                )
            }
        }
    }
}

impl Error for N2yoClientError {}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TleResponse {
    pub info: TleInfo,
    #[schema(example = "1 25544U 98067A ...\r\n2 25544 51.6412 ...")]
    pub tle: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TleInfo {
    #[schema(example = 25544)]
    pub satid: u32,
    #[schema(example = "SPACE STATION")]
    pub satname: String,
    #[schema(example = 4)]
    pub transactionscount: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PositionsResponse {
    pub info: SatelliteInfo,
    pub positions: Vec<PositionPoint>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SatelliteInfo {
    #[schema(example = "SPACE STATION")]
    pub satname: String,
    #[schema(example = 25544)]
    pub satid: u32,
    #[schema(example = 5)]
    pub transactionscount: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PositionPoint {
    #[schema(example = -39.90318514)]
    pub satlatitude: f64,
    #[schema(example = 158.28897924)]
    pub satlongitude: f64,
    #[schema(example = 417.85)]
    pub sataltitude: f64,
    #[schema(example = 254.31)]
    pub azimuth: f64,
    #[schema(example = -69.09)]
    pub elevation: f64,
    #[schema(example = 44.77078138)]
    pub ra: f64,
    #[schema(example = -43.99279118)]
    pub dec: f64,
    #[schema(example = 1521354418)]
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VisualPassesResponse {
    pub info: PassesInfo,
    pub passes: Vec<VisualPass>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RadioPassesResponse {
    pub info: PassesInfo,
    pub passes: Vec<RadioPass>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PassesInfo {
    #[schema(example = 25544)]
    pub satid: u32,
    #[schema(example = "SPACE STATION")]
    pub satname: String,
    #[schema(example = 4)]
    pub transactionscount: u32,
    #[schema(example = 3)]
    pub passescount: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VisualPass {
    #[schema(example = 307.21)]
    pub start_az: f64,
    #[schema(example = "NW")]
    pub start_az_compass: String,
    #[schema(example = 13.08)]
    pub start_el: f64,
    #[serde(rename = "startUTC")]
    #[schema(example = 1521368025)]
    pub start_utc: i64,
    #[schema(example = 225.45)]
    pub max_az: f64,
    #[schema(example = "SW")]
    pub max_az_compass: String,
    #[schema(example = 78.27)]
    pub max_el: f64,
    #[serde(rename = "maxUTC")]
    #[schema(example = 1521368345)]
    pub max_utc: i64,
    #[schema(example = 132.82)]
    pub end_az: f64,
    #[schema(example = "SE")]
    pub end_az_compass: String,
    #[schema(example = 0)]
    pub end_el: f64,
    #[serde(rename = "endUTC")]
    #[schema(example = 1521368660)]
    pub end_utc: i64,
    #[schema(example = -2.4)]
    pub mag: f64,
    #[schema(example = 485)]
    pub duration: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RadioPass {
    #[schema(example = 311.57)]
    pub start_az: f64,
    #[schema(example = "NW")]
    pub start_az_compass: String,
    #[serde(rename = "startUTC")]
    #[schema(example = 1521451295)]
    pub start_utc: i64,
    #[schema(example = 37.98)]
    pub max_az: f64,
    #[schema(example = "NE")]
    pub max_az_compass: String,
    #[schema(example = 52.19)]
    pub max_el: f64,
    #[serde(rename = "maxUTC")]
    #[schema(example = 1521451615)]
    pub max_utc: i64,
    #[schema(example = 118.6)]
    pub end_az: f64,
    #[schema(example = "ESE")]
    pub end_az_compass: String,
    #[serde(rename = "endUTC")]
    #[schema(example = 1521451925)]
    pub end_utc: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AboveResponse {
    pub info: AboveInfo,
    pub above: Vec<AboveSatellite>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AboveInfo {
    #[schema(example = "Amateur radio")]
    pub category: String,
    #[schema(example = 17)]
    pub transactionscount: u32,
    #[schema(example = 3)]
    pub satcount: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AboveSatellite {
    #[schema(example = 20480)]
    pub satid: u32,
    #[schema(example = "JAS 1B (FUJI 2)")]
    pub satname: String,
    #[schema(example = "1990-013C")]
    pub int_designator: String,
    #[schema(example = "1990-02-07")]
    pub launch_date: String,
    #[schema(example = 49.5744)]
    pub satlat: f64,
    #[schema(example = -96.7081)]
    pub satlng: f64,
    #[schema(example = 1227.9326)]
    pub satalt: f64,
}
