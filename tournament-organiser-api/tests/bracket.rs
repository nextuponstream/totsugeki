//! bracket management

use reqwest::StatusCode;
use sqlx::PgPool;
use totsugeki_core::matches::Match;
use totsugeki_core::player::Player;
use totsugeki_core::ID;
use tournament_organiser_api::resources::PaginationResult;
use tournament_organiser_api::services::tournaments::PaginatedTournamentResource;
use tournament_organiser_api::test_utils::spawn_app;
use tournament_organiser_api::tournaments::{
    BracketDisplay, BracketState, GenericResourceCreated, PlayerMatchResultReport,
};

#[sqlx::test]
async fn tournament_is_searchable(db: PgPool) {
    let app = spawn_app(db).await;
    app.login_as_test_user().await;

    let players = vec![];

    let request = tournament_organiser_api::tournaments::CreateTournamentForm {
        tournament_name: "".into(),
        player_names: players,
    };
    let response = app
        .http_client
        .post(format!("{}/api/tournaments", app.addr))
        .json(&request)
        .send()
        .await
        .expect("request done");

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
}

#[sqlx::test]
async fn create_tournament(db: PgPool) {
    let app = spawn_app(db).await;
    app.login_as_test_user().await;

    let players = vec![];

    // FIXME rename double elimination
    let response = app.create_tournament(players).await;

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
async fn cannot_create_tournament_without_data(db: PgPool) {
    let app = spawn_app(db).await;
    app.login_as_test_user().await;

    let response = app
        .http_client
        .post(format!("{}/api/tournaments", app.addr))
        .send()
        .await
        .expect("response");

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
}

#[sqlx::test]
async fn cannot_create_tournament_when_unauthenticated(db: PgPool) {
    let app = spawn_app(db).await;

    let players = vec![];

    let response = app.create_tournament(players).await;

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
}

#[sqlx::test]
async fn get_tournament(db: PgPool) {
    let app = spawn_app(db).await;
    app.login_as_test_user().await;

    let players = vec![];

    let response = app.create_tournament(players).await;

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::CREATED,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
    let r: GenericResourceCreated = response.json().await.unwrap();

    let response = app.get_tournament(r.id).await;

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::OK,
        "status: {status}, response: \"{}\", {}",
        response.text().await.unwrap(),
        r.id
    );

    let bracket: BracketDisplay = response.json().await.unwrap();
    // panic!("{:?}", response.text().await.unwrap());
    let matches: Vec<Match> = bracket.bracket.get_matches();
    assert!(matches.is_empty());
}

#[sqlx::test(fixtures("tournaments"))]
async fn list_tournaments(db: PgPool) {
    let app = spawn_app(db).await;
    app.login_as_test_user().await;

    let response = app.list_brackets(10, 0).await;

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::OK,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap(),
    );

    let response = app.list_brackets(100, 100).await;

    let status = response.status();
    assert_eq!(status, StatusCode::OK);

    let brackets: PaginationResult<PaginatedTournamentResource> = response.json().await.unwrap();
    assert_eq!(brackets.total, 100);
}
#[sqlx::test]
async fn save_tournament(db: PgPool) {
    let app = spawn_app(db).await;
    app.login_as_test_user().await;

    let p1 = Player::new("p1".into());
    let p2 = Player::new("p2".into());
    let p3 = Player::new("p3".into());
    let state = BracketState {
        bracket_name: "test bracket".to_string(),
        players: vec![p1.clone(), p2.clone(), p3.clone()],
        results: vec![
            PlayerMatchResultReport {
                p1_id: p2.clone().get_id(),
                p2_id: p3.clone().get_id(),
                score_p1: 2,
                score_p2: 0,
            },
            PlayerMatchResultReport {
                p1_id: p1.clone().get_id(),
                p2_id: p2.clone().get_id(),
                score_p1: 2,
                score_p2: 0,
            },
            PlayerMatchResultReport {
                p1_id: p2.clone().get_id(),
                p2_id: p3.clone().get_id(),
                score_p1: 0,
                score_p2: 2,
            },
            PlayerMatchResultReport {
                p1_id: p1.clone().get_id(),
                p2_id: p3.clone().get_id(),
                score_p1: 0,
                score_p2: 2,
            },
            PlayerMatchResultReport {
                p1_id: p1.clone().get_id(),
                p2_id: p3.clone().get_id(),
                score_p1: 0,
                score_p2: 2,
            },
        ],
    };

    let response = app.save_bracket(state).await;

    let status = response.status();
    assert_eq!(
        status,
        StatusCode::CREATED,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
}
#[sqlx::test(fixtures("3_players_tournament"))]
async fn join_tournament(mut db: PgPool) {
    let app = spawn_app(db.clone()).await;
    app.login_as_test_user().await;

    let mut transaction = db.begin().await.unwrap();

    // FIXME use api calls only
    let id: ID = "62aefc4c-d6ec-4c2f-98f0-b639688cbe0c".try_into().unwrap();
    let response = app.join_tournament(id).await;
    let status = response.status();
    assert_eq!(
        status,
        StatusCode::OK,
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
}
