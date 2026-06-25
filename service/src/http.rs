use std::fmt;

use actix_web::{HttpRequest, HttpResponse, ResponseError, body::BoxBody, http::StatusCode, web};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use crate::n2yo::{
    AboveResponse, N2yoClient, N2yoClientError, PositionsResponse, RadioPassesResponse,
    TleResponse, VisualPassesResponse,
};
use crate::open_meteo::{ElevationResponse, OpenMeteoClient, OpenMeteoClientError};

#[derive(Clone)]
pub struct AppState {
    pub n2yo: N2yoClient,
    pub open_meteo: OpenMeteoClient,
    pub service_version: String,
}

impl AppState {
    pub fn new(n2yo: N2yoClient, open_meteo: OpenMeteoClient, service_version: String) -> Self {
        Self {
            n2yo,
            open_meteo,
            service_version,
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health))
        .service(
            web::scope("/observer")
                .route("/elevation/{latitude}/{longitude}", web::get().to(get_elevation)),
        )
        .service(
            web::scope("/satellite")
                .route("/tle/{id}", web::get().to(get_tle))
                .route(
                    "/positions/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{seconds}",
                    web::get().to(get_positions),
                )
                .route(
                    "/visualpasses/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{days}/{min_visibility}",
                    web::get().to(get_visual_passes),
                )
                .route(
                    "/radiopasses/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{days}/{min_elevation}",
                    web::get().to(get_radio_passes),
                )
                .route(
                    "/above/{observer_lat}/{observer_lng}/{observer_alt}/{search_radius}/{category_id}",
                    web::get().to(get_above),
                ),
        );
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "n2yo-consumer",
        description = "N2YO REST API consumer for satellite tracking data.",
        version = "0.1.0"
    ),
    paths(
        health,
        get_elevation,
        get_tle,
        get_positions,
        get_visual_passes,
        get_radio_passes,
        get_above
    ),
    components(
        schemas(
            HealthResponse,
            ServiceHealth,
            ErrorResponse,
            ElevationResponse,
            TleResponse,
            crate::n2yo::TleInfo,
            PositionsResponse,
            crate::n2yo::SatelliteInfo,
            crate::n2yo::PositionPoint,
            VisualPassesResponse,
            RadioPassesResponse,
            crate::n2yo::PassesInfo,
            crate::n2yo::VisualPass,
            crate::n2yo::RadioPass,
            AboveResponse,
            crate::n2yo::AboveInfo,
            crate::n2yo::AboveSatellite
        )
    ),
    tags(
        (name = "Infrastructure", description = "Operational endpoints for service status and diagnostics."),
        (name = "Observer", description = "Endpoints that resolve observer location details."),
        (name = "Satellites", description = "Endpoints that proxy supported N2YO satellite APIs.")
    )
)]
pub struct ApiDoc;

#[utoipa::path(
    get,
    path = "/health",
    tag = "Infrastructure",
    operation_id = "getHealth",
    summary = "Check API health",
    responses((status = 200, description = "The API is running.", body = HealthResponse))
)]
pub async fn health(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_owned(),
        version: state.service_version.clone(),
        services: vec![ServiceHealth {
            name: "n2yo".to_owned(),
            status: "configured".to_owned(),
        }, ServiceHealth {
            name: "open-meteo".to_owned(),
            status: "configured".to_owned(),
        }],
    })
}

#[utoipa::path(
    get,
    path = "/observer/elevation/{latitude}/{longitude}",
    tag = "Observer",
    operation_id = "getObserverElevation",
    summary = "Get observer elevation",
    params(
        ("latitude" = f64, Path, description = "Observer latitude in decimal degrees."),
        ("longitude" = f64, Path, description = "Observer longitude in decimal degrees.")
    ),
    responses(
        (status = 200, description = "Elevation returned by Open-Meteo.", body = ElevationResponse),
        (status = 400, description = "Invalid request parameters.", body = ErrorResponse),
        (status = 502, description = "Open-Meteo request failed.", body = ErrorResponse)
    )
)]
pub async fn get_elevation(
    state: web::Data<AppState>,
    request: HttpRequest,
    path: web::Path<(f64, f64)>,
) -> Result<HttpResponse, ApiError> {
    let (latitude, longitude) = path.into_inner();
    validate_coordinates(latitude, longitude, "latitude", "longitude", request.path())?;

    let response = state
        .open_meteo
        .elevation(latitude, longitude)
        .await
        .map_err(|error| ApiError::from_open_meteo(error, request.path()))?;

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/satellite/tle/{id}",
    tag = "Satellites",
    operation_id = "getTle",
    summary = "Get satellite TLE",
    params(("id" = u32, Path, description = "NORAD satellite id.")),
    responses(
        (status = 200, description = "TLE returned by N2YO.", body = TleResponse),
        (status = 400, description = "Invalid request parameters.", body = ErrorResponse),
        (status = 502, description = "N2YO request failed.", body = ErrorResponse)
    )
)]
pub async fn get_tle(
    state: web::Data<AppState>,
    request: HttpRequest,
    id: web::Path<u32>,
) -> Result<HttpResponse, ApiError> {
    let response = state
        .n2yo
        .tle(id.into_inner())
        .await
        .map_err(|error| ApiError::from_n2yo(error, request.path()))?;

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/satellite/positions/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{seconds}",
    tag = "Satellites",
    operation_id = "getSatellitePositions",
    summary = "Get satellite positions",
    params(
        ("id" = u32, Path, description = "NORAD satellite id."),
        ("observer_lat" = f64, Path, description = "Observer latitude in decimal degrees."),
        ("observer_lng" = f64, Path, description = "Observer longitude in decimal degrees."),
        ("observer_alt" = f64, Path, description = "Observer altitude above sea level in meters."),
        ("seconds" = u16, Path, description = "Number of future positions to return. Maximum 300.")
    ),
    responses(
        (status = 200, description = "Positions returned by N2YO.", body = PositionsResponse),
        (status = 400, description = "Invalid request parameters.", body = ErrorResponse),
        (status = 502, description = "N2YO request failed.", body = ErrorResponse)
    )
)]
pub async fn get_positions(
    state: web::Data<AppState>,
    request: HttpRequest,
    path: web::Path<(u32, f64, f64, f64, u16)>,
) -> Result<HttpResponse, ApiError> {
    let (id, observer_lat, observer_lng, observer_alt, seconds) = path.into_inner();
    validate_observer(observer_lat, observer_lng, request.path())?;

    if seconds == 0 || seconds > 300 {
        return Err(ApiError::bad_request(
            "seconds must be between 1 and 300",
            request.path(),
        ));
    }

    let response = state
        .n2yo
        .positions(id, observer_lat, observer_lng, observer_alt, seconds)
        .await
        .map_err(|error| ApiError::from_n2yo(error, request.path()))?;

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/satellite/visualpasses/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{days}/{min_visibility}",
    tag = "Satellites",
    operation_id = "getVisualPasses",
    summary = "Get visual passes",
    params(
        ("id" = u32, Path, description = "NORAD satellite id."),
        ("observer_lat" = f64, Path, description = "Observer latitude in decimal degrees."),
        ("observer_lng" = f64, Path, description = "Observer longitude in decimal degrees."),
        ("observer_alt" = f64, Path, description = "Observer altitude above sea level in meters."),
        ("days" = u8, Path, description = "Number of prediction days. Maximum 10."),
        ("min_visibility" = u32, Path, description = "Minimum visible duration in seconds.")
    ),
    responses(
        (status = 200, description = "Visual passes returned by N2YO.", body = VisualPassesResponse),
        (status = 400, description = "Invalid request parameters.", body = ErrorResponse),
        (status = 502, description = "N2YO request failed.", body = ErrorResponse)
    )
)]
pub async fn get_visual_passes(
    state: web::Data<AppState>,
    request: HttpRequest,
    path: web::Path<(u32, f64, f64, f64, u8, u32)>,
) -> Result<HttpResponse, ApiError> {
    let (id, observer_lat, observer_lng, observer_alt, days, min_visibility) = path.into_inner();
    validate_observer(observer_lat, observer_lng, request.path())?;
    validate_days(days, request.path())?;

    let response = state
        .n2yo
        .visual_passes(
            id,
            observer_lat,
            observer_lng,
            observer_alt,
            days,
            min_visibility,
        )
        .await
        .map_err(|error| ApiError::from_n2yo(error, request.path()))?;

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/satellite/radiopasses/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{days}/{min_elevation}",
    tag = "Satellites",
    operation_id = "getRadioPasses",
    summary = "Get radio passes",
    params(
        ("id" = u32, Path, description = "NORAD satellite id."),
        ("observer_lat" = f64, Path, description = "Observer latitude in decimal degrees."),
        ("observer_lng" = f64, Path, description = "Observer longitude in decimal degrees."),
        ("observer_alt" = f64, Path, description = "Observer altitude above sea level in meters."),
        ("days" = u8, Path, description = "Number of prediction days. Maximum 10."),
        ("min_elevation" = u8, Path, description = "Minimum max elevation in degrees.")
    ),
    responses(
        (status = 200, description = "Radio passes returned by N2YO.", body = RadioPassesResponse),
        (status = 400, description = "Invalid request parameters.", body = ErrorResponse),
        (status = 502, description = "N2YO request failed.", body = ErrorResponse)
    )
)]
pub async fn get_radio_passes(
    state: web::Data<AppState>,
    request: HttpRequest,
    path: web::Path<(u32, f64, f64, f64, u8, u8)>,
) -> Result<HttpResponse, ApiError> {
    let (id, observer_lat, observer_lng, observer_alt, days, min_elevation) = path.into_inner();
    validate_observer(observer_lat, observer_lng, request.path())?;
    validate_days(days, request.path())?;

    let response = state
        .n2yo
        .radio_passes(
            id,
            observer_lat,
            observer_lng,
            observer_alt,
            days,
            min_elevation,
        )
        .await
        .map_err(|error| ApiError::from_n2yo(error, request.path()))?;

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/satellite/above/{observer_lat}/{observer_lng}/{observer_alt}/{search_radius}/{category_id}",
    tag = "Satellites",
    operation_id = "getSatellitesAbove",
    summary = "Get satellites above observer",
    params(
        ("observer_lat" = f64, Path, description = "Observer latitude in decimal degrees."),
        ("observer_lng" = f64, Path, description = "Observer longitude in decimal degrees."),
        ("observer_alt" = f64, Path, description = "Observer altitude above sea level in meters."),
        ("search_radius" = u8, Path, description = "Search radius in degrees. Range 0 to 90."),
        ("category_id" = u32, Path, description = "N2YO category id. Use 0 for all categories.")
    ),
    responses(
        (status = 200, description = "Satellites above observer returned by N2YO.", body = AboveResponse),
        (status = 400, description = "Invalid request parameters.", body = ErrorResponse),
        (status = 502, description = "N2YO request failed.", body = ErrorResponse)
    )
)]
pub async fn get_above(
    state: web::Data<AppState>,
    request: HttpRequest,
    path: web::Path<(f64, f64, f64, u8, u32)>,
) -> Result<HttpResponse, ApiError> {
    let (observer_lat, observer_lng, observer_alt, search_radius, category_id) = path.into_inner();
    validate_observer(observer_lat, observer_lng, request.path())?;

    if search_radius > 90 {
        return Err(ApiError::bad_request(
            "search_radius must be between 0 and 90",
            request.path(),
        ));
    }

    let response = state
        .n2yo
        .above(
            observer_lat,
            observer_lng,
            observer_alt,
            search_radius,
            category_id,
        )
        .await
        .map_err(|error| ApiError::from_n2yo(error, request.path()))?;

    Ok(HttpResponse::Ok().json(response))
}

fn validate_observer(lat: f64, lng: f64, path: &str) -> Result<(), ApiError> {
    validate_coordinates(lat, lng, "observer_lat", "observer_lng", path)
}

fn validate_coordinates(
    lat: f64,
    lng: f64,
    lat_field: &'static str,
    lng_field: &'static str,
    path: &str,
) -> Result<(), ApiError> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err(ApiError::bad_request(
            format!("{lat_field} must be between -90 and 90"),
            path,
        ));
    }

    if !(-180.0..=180.0).contains(&lng) {
        return Err(ApiError::bad_request(
            format!("{lng_field} must be between -180 and 180"),
            path,
        ));
    }

    Ok(())
}

fn validate_days(days: u8, path: &str) -> Result<(), ApiError> {
    if days == 0 || days > 10 {
        return Err(ApiError::bad_request("days must be between 1 and 10", path));
    }

    Ok(())
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    #[schema(example = "ok")]
    pub status: String,
    #[schema(example = "0.1.0")]
    pub version: String,
    pub services: Vec<ServiceHealth>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ServiceHealth {
    #[schema(example = "n2yo")]
    pub name: String,
    #[schema(example = "configured")]
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    #[schema(example = 400)]
    pub status_code: u16,
    #[schema(example = "VALIDATION_ERROR")]
    pub code: String,
    #[schema(example = "seconds must be between 1 and 300")]
    pub message: String,
    #[schema(example = "/satellite/positions/25544/41.702/-76.014/0/301")]
    pub path: String,
}

#[derive(Debug)]
pub struct ApiError {
    status_code: StatusCode,
    code: &'static str,
    message: String,
    path: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>, path: &str) -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            code: "VALIDATION_ERROR",
            message: message.into(),
            path: path.to_owned(),
        }
    }

    fn from_n2yo(error: N2yoClientError, path: &str) -> Self {
        log::warn!("N2YO request failed: {error}");

        Self {
            status_code: StatusCode::BAD_GATEWAY,
            code: "N2YO_UPSTREAM_ERROR",
            message: "failed to fetch data from N2YO".to_owned(),
            path: path.to_owned(),
        }
    }

    fn from_open_meteo(error: OpenMeteoClientError, path: &str) -> Self {
        log::warn!("Open-Meteo request failed: {error}");

        Self {
            status_code: StatusCode::BAD_GATEWAY,
            code: "OPEN_METEO_UPSTREAM_ERROR",
            message: "failed to fetch elevation from Open-Meteo".to_owned(),
            path: path.to_owned(),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code).json(ErrorResponse {
            status_code: self.status_code.as_u16(),
            code: self.code.to_owned(),
            message: self.message.clone(),
            path: self.path.clone(),
        })
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
