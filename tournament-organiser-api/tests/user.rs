//! User management

use reqwest::StatusCode;
use sqlx::PgPool;
use tournament_organiser_api::test_utils::{spawn_app, FormUserInput, LoginForm};

#[sqlx::test]
async fn cannot_delete_user_without_a_valid_session(db: PgPool) {
    let app = spawn_app(db).await;

    // create a user to delete
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

    let response = app.delete_user().await;

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
}

#[sqlx::test]
async fn delete_user(db: PgPool) -> sqlx::Result<()> {
    let mut conn = db.acquire().await?;
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

    let records = sqlx::query("SELECT * FROM users")
        // wtf dereference?
        .fetch_all(&mut *conn)
        .await?;

    assert_eq!(records.len(), 1);

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

    let response = app.delete_user().await;

    let status = response.status();
    assert!(
        status.is_success(),
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );

    let records = sqlx::query("SELECT * FROM users")
        .fetch_all(&mut *conn)
        .await?;

    assert!(
        records.is_empty(),
        "user not deleted: remaining user {}",
        records.len()
    );

    Ok(())
}
