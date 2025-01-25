//! Create brackets

use crate::http::internal_error;
use crate::http::Error;
use crate::middlewares::validation::ValidatedJson;
use crate::repositories::tournaments::TournamentRepository;
use crate::services::tournaments::TournamentService;
use crate::tournaments::tournament_players::TournamentPlayer;
use crate::tournaments::Tournament;
use crate::tournaments::{CreateTournamentForm, GenericResourceCreated};
use crate::users::session::Keys::UserId;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json as AxumJson;
use axum_macros::debug_handler;
use http::StatusCode;
use sqlx::PgPool;
use totsugeki_core::bracket::seeding::Seeding;
use totsugeki_core::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki_core::matches::result::MatchFormat;
use totsugeki_core::player::{Player, PlayerID};
use totsugeki_core::validation::AutomaticMatchValidationMode;
use totsugeki_core::ID;
use tower_sessions::Session;
use tracing::instrument;

/// Return a newly instanciated bracket from ordered (=seeded) player names
#[instrument(name = "create_tournament", skip(pool, session))]
#[debug_handler]
pub(crate) async fn create_tournament(
    session: Session,
    State(pool): State<PgPool>, // FIXME cannot use ConnectionPool
    ValidatedJson(form): ValidatedJson<CreateTournamentForm>,
) -> impl IntoResponse {
    tracing::debug!("new bracket from players: {:?}", form.player_names);

    let mut transaction = pool.begin().await.map_err(internal_error)?;
    // TODO refactor user_id key in SESSION_KEY enum
    let user_id: ID = session.get(&UserId.to_string()).await.expect("").expect("");
    let mut tournament = Tournament::default();
    tournament.set_name(form.tournament_name);
    TournamentRepository::create(&mut transaction, &tournament).await?;
    let _guest_ids = TournamentService::add_guests_as_tournament_players(
        &mut transaction,
        tournament.id,
        form.player_names,
    )
    .await?;
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(
            tournament
                .get_players()
                .into_iter()
                .map(|tp| PlayerID::new(tp.get_id()))
                .collect(),
        )
        .expect("should form seeding with new player"),
        AutomaticMatchValidationMode::default(),
        MatchFormat::ft2(),
        None,
    );

    TournamentService::create_tournament_organiser_and_matches(
        &mut transaction,
        &tournament,
        &bracket,
        user_id,
    )
    .await?;

    transaction.commit().await?;
    println!("{}", tournament.get_id());

    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    tracing::info!("new tournament {}", tournament.get_id().0);

    tracing::debug!("new tournament {:?}", bracket);
    Ok::<(StatusCode, axum::Json<GenericResourceCreated>), Error>((
        StatusCode::CREATED,
        AxumJson(GenericResourceCreated {
            id: tournament.get_id().0,
        }),
    ))
}
