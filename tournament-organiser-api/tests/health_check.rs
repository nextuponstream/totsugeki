//! tests/health_check.rs
use std::net::{SocketAddr, TcpListener};
use tournament_organiser_api::{app, HealthCheck};

fn spawn_app() -> SocketAddr {
    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app().into_make_service())
            .await
            .unwrap();
    });
    addr
}

// zero to prod is ok but let's follow tokio testing material
// https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/testing/src/main.rs#L133
#[tokio::test]
async fn health_check() {
    let addr = spawn_app();
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(format!("http://{addr}/api/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    let status = response.status();
    assert!(
        status.is_success(),
        "status: {status}, http://{addr}/api/health_check"
    );
    // let json_response = response.text().await.unwrap();
    let json_response: HealthCheck = response.json().await.unwrap();
    assert!(json_response.ok)
}

// TODO Add cypress test to test redirecting to 404 page when it's a unknown
// url not nested in /api

#[tokio::test]
async fn redirect_on_bad_api_url() {
    let addr = spawn_app();

    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(format!("http://{addr}/api/health_checkkkk"))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    let status = response.status();
    assert!(!status.is_success(), "status: {status}, http://{addr}");
}
