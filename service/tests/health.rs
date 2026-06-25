use actix_web::{App, test, web};
use service::{
    http::{AppState, configure},
    n2yo::N2yoClient,
};

#[actix_web::test]
async fn health_returns_ok_without_calling_n2yo() {
    let n2yo = N2yoClient::new("https://api.n2yo.com/rest/v1/satellite", "test-key", 1)
        .expect("client should build");
    let app_state = AppState::new(n2yo, env!("CARGO_PKG_VERSION").to_owned());
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
