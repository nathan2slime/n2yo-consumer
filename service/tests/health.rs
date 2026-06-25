use actix_web::{App, http::StatusCode, test, web};
use service::{
    http::{AppState, configure},
    n2yo::N2yoClient,
    open_meteo::OpenMeteoClient,
};

#[actix_web::test]
async fn health_returns_ok_without_calling_upstreams() {
    let n2yo = N2yoClient::new("https://api.n2yo.com/rest/v1/satellite", "test-key", 1)
        .expect("client should build");
    let open_meteo = OpenMeteoClient::new("https://api.open-meteo.com/v1/elevation", 1)
        .expect("client should build");
    let app_state = AppState::new(n2yo, open_meteo, env!("CARGO_PKG_VERSION").to_owned());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure),
    )
    .await;

    let request = test::TestRequest::get().uri("/health").to_request();
    let response = test::call_service(&app, request).await;

    assert!(response.status().is_success());
}

#[actix_web::test]
async fn elevation_rejects_invalid_coordinates_without_calling_open_meteo() {
    let n2yo = N2yoClient::new("https://api.n2yo.com/rest/v1/satellite", "test-key", 1)
        .expect("client should build");
    let open_meteo =
        OpenMeteoClient::new("http://127.0.0.1:9/elevation", 1).expect("client should build");
    let app_state = AppState::new(n2yo, open_meteo, env!("CARGO_PKG_VERSION").to_owned());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure),
    )
    .await;

    let request = test::TestRequest::get()
        .uri("/observer/elevation/91/13.41")
        .to_request();
    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
