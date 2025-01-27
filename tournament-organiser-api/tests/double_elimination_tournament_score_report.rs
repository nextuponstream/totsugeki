use http::StatusCode;
use sqlx::PgPool;
use totsugeki_core::matches::result::Score;
use totsugeki_core::ID;
use tournament_organiser_api::test_utils::spawn_app;

#[sqlx::test(fixtures("3_players_tournament"))]
async fn report_scores_for_3_man_tournament(db: PgPool) {
    let app = spawn_app(db).await;
    app.login_as_test_user().await;

    let tournament_id: ID = "62aefc4c-d6ec-4c2f-98f0-b639688cbe0c".try_into().unwrap();
    let response = app
        .report_score_for_tournament(tournament_id, Score(2, 0))
        .await;
    let status = response.status();
    assert_eq!(
        status,
        StatusCode::OK,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
}
