//! tests/health_check.rs
use http::StatusCode;
use reqwest::Client;
use tournament_organiser_api::{test_utils::spawn_app, HealthCheck};

// zero to prod is ok but let's follow tokio testing material
// https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/testing/src/main.rs#L133
#[tokio::test]
async fn health_check() {
    let app = spawn_app().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/api/health_check", app.addr))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();
    assert!(
        status.is_success(),
        "status: {status}, {}/api/health_check",
        app.addr
    );
    let json_response: HealthCheck = response.json().await.unwrap();
    assert!(json_response.ok)
}

// TODO Add cypress test to test redirecting to 404 page when it's a unknown
// url not nested in /api

#[tokio::test]
async fn redirect_on_bad_api_url() {
    let app = spawn_app().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/api/health_checkkkk", app.addr))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "status: {status}, {}",
        app.addr
    );
}