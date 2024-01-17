//! tests/health_check.rs
use http::StatusCode;
use reqwest::Client;
use std::net::{SocketAddr, TcpListener};
use tournament_organiser_api::{app, HealthCheck};

/// Returns address to connect to new application (with random available port)
///
/// Example: http://0.0.0.0:43222
fn spawn_app() -> String {
    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app().into_make_service())
            .await
            .unwrap();
    });

    format!("http://{addr}")
}

// zero to prod is ok but let's follow tokio testing material
// https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/testing/src/main.rs#L133
#[tokio::test]
async fn health_check() {
    let addr = spawn_app();
    let client = Client::new();

    let response = client
        .get(format!("{addr}/api/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();
    assert!(
        status.is_success(),
        "status: {status}, {addr}/api/health_check"
    );
    let json_response: HealthCheck = response.json().await.unwrap();
    assert!(json_response.ok)
}

// TODO Add cypress test to test redirecting to 404 page when it's a unknown
// url not nested in /api

#[tokio::test]
async fn redirect_on_bad_api_url() {
    let addr = spawn_app();
    let client = Client::new();

    let response = client
        .get(format!("{addr}/api/health_checkkkk"))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();
    assert_eq!(status, StatusCode::NOT_FOUND, "status: {status}, {addr}");
}
