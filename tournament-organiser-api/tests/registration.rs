//! tests/health_check.rs
use http::StatusCode;
use sqlx::PgPool;
use tournament_organiser_api::registration::{ErrorResponse, FormInput};
use tournament_organiser_api::test_utils::spawn_app;

// Use sqlx macro to create (and teardown) database on the fly to enforce test
// isolation
// https://docs.rs/sqlx/latest/sqlx/attr.test.html
// https://github.com/launchbadge/sqlx/blob/31e541ac7a9c7d18ee2b3b91c58349e77eac28f7/examples/postgres/axum-social-with-tests/tests/post.rs#L17
#[sqlx::test]
async fn registration(db: PgPool) {
    let app = spawn_app(db).await;

    let response = app
        .register(&FormInput {
            name: "jean".into(),
            email: "jean@bon.ch".into(),
            password: "verySecurePassword#123456789?".into(),
            created_at: None,
        })
        .await;

    let status = response.status();
    assert!(
        status.is_success(),
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
}

#[sqlx::test]
async fn registration_fails_when_another_user_already_exists(db: PgPool) {
    let app = spawn_app(db).await;
    let request = FormInput {
        name: "jean".into(),
        email: "jean@bon.ch".into(),
        password: "verySecurePassword#123456789?".into(),
        created_at: None,
    };

    let response = app.register(&request).await;

    let status = response.status();
    assert!(
        status.is_success(),
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );

    let response = app.register(&request).await;

    let status = response.status();

    // why 409: https://stackoverflow.com/a/3826024
    assert_eq!(status, StatusCode::CONFLICT);
    let details: ErrorResponse = response.json().await.unwrap();
    assert_eq!(
        details.message,
        "Another user has already registered with provided mail"
    )
}
