use http::StatusCode;
use sqlx::PgPool;
use totsugeki_core::matches::result::Score;
use totsugeki_core::ID;
use tournament_organiser_api::test_utils::{spawn_app, TestApp};
use uuid::uuid;

const TOURNAMENT_ID: &ID = &uuid!("62aefc4c-d6ec-4c2f-98f0-b639688cbe0c");
const DIEGO_PLAYER_ID: &ID = &uuid!("d245b66d-b2cb-4a91-bab4-ce326146a7ad");
const PINK_PLAYER_ID: &ID = &uuid!("dab8c793-f134-44f5-aa7e-87fa644ba9c4");
const JOHN_MID_PLAYER_ID: &ID = &uuid!("75e3cf69-f89b-40f1-a047-00b55701e1b0");

#[sqlx::test(fixtures("3_players_tournament"))]
async fn report_scores_for_3_man_tournament(db: PgPool) {
    let app = spawn_app(db).await;
    app.login_as_test_user().await;

    send_score(
        &app,
        TOURNAMENT_ID,
        Score(2, 0),
        PINK_PLAYER_ID,
        JOHN_MID_PLAYER_ID,
    )
    .await;
}

#[sqlx::test(fixtures("3_players_tournament"))]
async fn report_scores_for_3_man_tournament_full(db: PgPool) {
    let app = spawn_app(db).await;
    app.login_as_test_user().await;

    // Playing to get into winner finals
    send_score(
        &app,
        TOURNAMENT_ID,
        Score(3, 0),
        PINK_PLAYER_ID,
        JOHN_MID_PLAYER_ID,
    )
    .await;

    // winner finals
    send_score(
        &app,
        TOURNAMENT_ID,
        Score(3, 1),
        DIEGO_PLAYER_ID,
        PINK_PLAYER_ID,
    )
    .await;

    // loser finals
    send_score(
        &app,
        TOURNAMENT_ID,
        Score(3, 2),
        PINK_PLAYER_ID,
        JOHN_MID_PLAYER_ID,
    )
    .await;

    // Grand finals, with upset!
    send_score(
        &app,
        TOURNAMENT_ID,
        Score(0, 3),
        DIEGO_PLAYER_ID,
        PINK_PLAYER_ID,
    )
    .await;

    // Grand finals reset, Diego is too good
    send_score(
        &app,
        TOURNAMENT_ID,
        Score(3, 1),
        DIEGO_PLAYER_ID,
        PINK_PLAYER_ID,
    )
    .await;
}

async fn send_score(
    app: &TestApp,
    tournament_id: &ID,
    score: Score,
    high_seed_player: &ID,
    low_seed_player: &ID,
) {
    let response = app
        .report_score_for_tournament(&tournament_id, score, high_seed_player, low_seed_player)
        .await;
    let status = response.status();
    assert_eq!(
        status,
        StatusCode::OK,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
}
