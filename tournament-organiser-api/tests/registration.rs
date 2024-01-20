//! tests/health_check.rs
use reqwest::Client;
use sqlx::PgPool;
use tournament_organiser_api::registration::FormInput;
use tournament_organiser_api::test_utils::spawn_app;

// Use sqlx macro to create (and teardown) database on the fly to enforce test
// isolation
// https://docs.rs/sqlx/latest/sqlx/attr.test.html
// https://github.com/launchbadge/sqlx/blob/31e541ac7a9c7d18ee2b3b91c58349e77eac28f7/examples/postgres/axum-social-with-tests/tests/post.rs#L17
#[sqlx::test]
async fn registration(db: PgPool) {
    let app = spawn_app(db).await;
    let client = Client::new();

    let response = client
        .post(format!("{}/api/register", app.addr))
        .json(&FormInput {
            name: "jean".into(),
            email: "jean@bon.ch".into(),
            password: "verySecurePassword#123456789?".into(),
            created_at: None,
        })
        .send()
        .await
        .expect("request done");

    let status = response.status();
    assert!(
        status.is_success(),
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
}
