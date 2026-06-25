use actix_web::{App, HttpResponse, HttpServer, web};
use service::{
    config::env::AppConfig,
    http::{ApiDoc, AppState, configure},
    n2yo::N2yoClient,
    open_meteo::OpenMeteoClient,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let config = AppConfig::from_env().map_err(|error| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, error.to_string())
    })?;

    log::info!("starting aumigo service");
    log::info!(
        "service config loaded (bind={}, n2yo_base_url={}, n2yo_timeout_seconds={}, open_meteo_elevation_url={}, open_meteo_timeout_seconds={})",
        config.api_bind,
        config.n2yo_base_url,
        config.n2yo_timeout_seconds,
        config.open_meteo_elevation_url,
        config.open_meteo_timeout_seconds
    );

    let n2yo_client = N2yoClient::new(
        config.n2yo_base_url.clone(),
        config.n2yo_api_key.clone(),
        config.n2yo_timeout_seconds,
    )
    .map_err(std::io::Error::other)?;
    let open_meteo_client = OpenMeteoClient::new(
        config.open_meteo_elevation_url.clone(),
        config.open_meteo_timeout_seconds,
    )
    .map_err(std::io::Error::other)?;
    let http_state = AppState::new(
        n2yo_client,
        open_meteo_client,
        env!("CARGO_PKG_VERSION").to_owned(),
    );

    let openapi = ApiDoc::openapi()
        .to_pretty_json()
        .expect("failed to serialize OpenAPI document");
    log::info!("openapi docs ready at /service-docs/openapi.json and /docs/");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(http_state.clone()))
            .app_data(web::Data::new(openapi.clone()))
            .configure(configure)
            .service(
                SwaggerUi::new("/docs/{_:.*}").url("/service-docs/openapi.json", ApiDoc::openapi()),
            )
            .route(
                "/service-docs/openapi.json",
                web::get().to(|openapi: web::Data<String>| async move {
                    HttpResponse::Ok()
                        .content_type("application/json")
                        .body(openapi.get_ref().clone())
                }),
            )
    })
    .bind(config.api_bind.clone())?
    .run()
    .await
}
