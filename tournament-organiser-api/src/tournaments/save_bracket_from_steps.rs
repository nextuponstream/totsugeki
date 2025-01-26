//! Save bracket from steps

use crate::guests::Guest;
use crate::http::{internal_error, ErrorSlug};
use crate::repositories::guests::GuestRepository;
use crate::repositories::players::PlayerRepository;
use crate::repositories::tournaments::TournamentRepository;
use crate::services::tournaments::TournamentService;
use crate::tournaments::tournament_players::TournamentPlayer;
use crate::tournaments::Tournament;
use crate::tournaments::{breakdown, BracketState};
use crate::users::session::Keys;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use axum_macros::debug_handler;
use http::StatusCode;
use sqlx::PgPool;
use totsugeki_core::bracket::seeding::Seeding;
use totsugeki_core::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki_core::matches::result::{MatchFormat, Score};
use totsugeki_core::validation::AutomaticMatchValidationMode;
use totsugeki_core::ID;
use tower_sessions::Session;
use tracing::instrument;

/// Save bracket replayed from player reports so in the event a guest actually
/// wants to save the resulting bracket, they can.
///
/// The server will not accept a JSON of a bracket just because it can be
/// parsed as that may lead to a malformed bracket. Then we do something a
/// little more intense computation wise that always yields a correct bracket.
///
/// When creating a bracket this way, we assume EVERY player created this way is
/// a guest and not an actual user
#[instrument(name = "save_bracket_from_steps")]
#[debug_handler]
pub async fn save_tournament_from_steps(
    session: Session,
    State(pool): State<PgPool>,
    Json(bracket_state): Json<BracketState>,
) -> impl IntoResponse {
    // NOTE: always pool before arguments. Otherwise:
    // error[E0277]: the trait bound `fn(axum::Json<BracketState>,
    // State<Pool<Postgres>>) -> impl std::future::Future<Output = impl
    // IntoResponse> {save_bracket}: Handler<_, _>` is not satisfied
    tracing::debug!("new tournament replayed from steps");
    let mut transaction = pool.begin().await?;

    let mut tournament = Tournament::default();
    tournament.set_name(bracket_state.bracket_name);
    TournamentRepository::create(&mut transaction, &tournament).await?;

    let mut safe_player_mapping = vec![];
    // Do not rely on given ID, assign new IDs to players and map
    let tournament_players = TournamentService::add_guests_as_tournament_players(
        &mut transaction,
        tournament.id,
        bracket_state
            .players
            .clone()
            .into_iter()
            .map(|p| p.get_name())
            .collect(),
    )
    .await?;
    for (index, tournament_player) in tournament_players.iter().enumerate() {
        // regen player ID server side, don't trust
        let Ok(()) = tournament.add_player(tournament_player.clone()) else {
            tracing::warn!("oh no");
            return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
        };
        safe_player_mapping.push((bracket_state.players[index].clone(), tournament_player));
    }
    let mut bracket = DoubleEliminationBracket::create(
        Seeding::new(
            tournament
                .get_players()
                .into_iter()
                .map(|tp| tp.get_id())
                .collect(),
        )
        .expect("should use seeding from tournament organiser input"),
        AutomaticMatchValidationMode::Flexible, // FIXME get from form
        MatchFormat::ft2(),                     // FIXME get from form
        None,
    );
    for r in bracket_state.results {
        let report = Score(r.score_p1, r.score_p2);
        let Some(p1_mapping) = safe_player_mapping
            .iter()
            .find(|m| m.0.get_id() == r.player1_id)
        else {
            return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
        };
        let Some(p2_mapping) = safe_player_mapping
            .iter()
            .find(|m| m.0.get_id() == r.player2_id)
        else {
            return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
        };
        let bracket_copy = bracket.clone();
        bracket = match bracket_copy.tournament_organiser_reports_result_dangerous(
            p1_mapping.1.get_id(),
            report,
            p2_mapping.1.get_id(),
        ) {
            Ok(b) => b.0,
            Err(err) => {
                tracing::warn!("{err}");
                return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
            }
        };
    }

    // FIXME remove
    let players = PlayerRepository::read_for_tournament(&mut transaction, tournament.id).await?;
    for player in players {
        println!("{} {}", player.name, player.id,);
    }

    let user_id: ID = session
        .get(&Keys::UserId.to_string())
        .await
        .expect("value from store")
        .expect("user id");
    TournamentService::create_tournament_organiser_and_matches(
        &mut transaction,
        &tournament,
        &bracket,
        user_id,
    )
    .await?;

    transaction.commit().await?;

    tracing::info!("new tournament replayed from steps {}", tournament.get_id());
    tracing::debug!("new tournament replayed from steps {:?}", bracket);

    Ok((
        StatusCode::CREATED,
        breakdown(&tournament, bracket, None, true),
    ))
}
