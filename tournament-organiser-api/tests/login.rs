//! login tests

use http::StatusCode;
use sqlx::PgPool;
use tournament_organiser_api::test_utils::{spawn_app, FormUserInput, LoginForm};

#[sqlx::test]
async fn login(db: PgPool) {
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

    let response = app
        .login(&LoginForm {
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
async fn bad_login(db: PgPool) {
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

    let response = app
        .login(&LoginForm {
            email: "jean@bon.ch".into(),
            // slight typo
            password: "vrySecurePassword#123456789?".into(),
        })
        .await;

    let status = response.status();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
