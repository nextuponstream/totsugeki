//! registration tests

use http::StatusCode;
use sqlx::PgPool;
use tournament_organiser_api::test_utils::{spawn_app, FormUserInput};
use tournament_organiser_api::ErrorResponse;

// Use sqlx macro to create (and teardown) database on the fly to enforce test
// isolation
// https://docs.rs/sqlx/latest/sqlx/attr.test.html
// https://github.com/launchbadge/sqlx/blob/31e541ac7a9c7d18ee2b3b91c58349e77eac28f7/examples/postgres/axum-social-with-tests/tests/post.rs#L17
#[sqlx::test]
async fn registration(db: PgPool) {
    let app = spawn_app(db).await;

    let response = app
        .register(&FormUserInput {
            name: "jean".into(),
            email: "jean@bon.ch".into(),
            password: "verySecurePassword#123456789?".into(),
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
    let request = FormUserInput {
        name: "jean".into(),
        email: "jean@bon.ch".into(),
        password: "verySecurePassword#123456789?".into(),
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

#[sqlx::test]
async fn weak_password_ask_for_stronger_password(db: PgPool) {
    let app = spawn_app(db).await;

    let response = app
        .register(&FormUserInput {
            name: "jean".into(),
            email: "jean@bon.ch".into(),
            password: "weakpw".into(),
        })
        .await;

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );

    let json: ErrorResponse = response.json().await.unwrap();
    assert!(json.message.contains("weak_password"));
}
