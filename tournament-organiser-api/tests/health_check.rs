//! tests/health_check.rs
use reqwest::Client;
use reqwest::StatusCode;
use sqlx::PgPool;
use tournament_organiser_api::{health_check::HealthCheck, test_utils::spawn_app};

// NOTE: zero2prod uses tokio stuff but let's use sqlx to assert db state

#[sqlx::test]
async fn health_check(db: PgPool) {
    let app = spawn_app(db).await;
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

#[sqlx::test]
async fn redirect_on_bad_api_url(db: PgPool) {
    let app = spawn_app(db).await;
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
