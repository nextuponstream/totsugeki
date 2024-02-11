//! bracket management

use reqwest::StatusCode;
use sqlx::PgPool;
use tournament_organiser_api::bracket::GenericResourceCreated;
use tournament_organiser_api::test_utils::spawn_app;

#[sqlx::test]
async fn create_bracket(db: PgPool) {
    let app = spawn_app(db).await;
    app.login_as_test_user().await;

    let players = vec![];

    let response = app.create_bracket(players).await;

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::CREATED,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
    let _id: GenericResourceCreated = response.json().await.unwrap();
}

#[sqlx::test]
async fn cannot_create_bracket_when_unauthenticated(db: PgPool) {
    let app = spawn_app(db).await;

    let players = vec![];

    let response = app.create_bracket(players).await;

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
}
