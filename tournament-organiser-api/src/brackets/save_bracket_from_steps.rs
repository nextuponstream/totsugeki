//! Save bracket from steps

use crate::brackets::{breakdown, BracketState};
use crate::http::{internal_error, ErrorSlug};
use crate::repositories::brackets::TournamentService;
use crate::tournaments::Tournament;
use crate::users::session::Keys;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use axum_macros::debug_handler;
use http::StatusCode;
use sqlx::PgPool;
use totsugeki::bracket::seeding::Seeding;
use totsugeki::double_elimination_bracket::progression::ProgressionDEB;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::player::Player;
use totsugeki::validation::AutomaticMatchValidationMode;
use tower_sessions::Session;
use tracing::instrument;

/// Save bracket replayed from player reports so in the event a guest actually
/// wants to save the resulting bracket, they can.
///
/// The server will not accept a JSON of a bracket just because it can be
/// parsed as that may lead to a malformed bracket. Then we do something a
/// little more intense computation wise that always yields a correct bracket.
#[instrument(name = "save_bracket_from_steps")]
#[debug_handler]
pub async fn save_bracket_from_steps(
    session: Session,
    State(pool): State<PgPool>,
    Json(bracket_state): Json<BracketState>,
) -> impl IntoResponse {
    // NOTE: always pool before arguments. Otherwise:
    // error[E0277]: the trait bound `fn(axum::Json<BracketState>,
    // State<Pool<Postgres>>) -> impl std::future::Future<Output = impl
    // IntoResponse> {save_bracket}: Handler<_, _>` is not satisfied
    tracing::debug!("new bracket replayed from steps");

    let mut tournament = Tournament::default();
    tournament.set_name(bracket_state.bracket_name);
    let mut safe_player_mapping = vec![];
    // Do not rely on given ID, assign new IDs to players and map
    for player in bracket_state.players {
        // regen player ID server side, don't trust
        let safe_player = Player::new(player.get_name());
        let Ok(()) = tournament.add_participant(safe_player.clone()) else {
            tracing::warn!("oh no");
            return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
        };
        safe_player_mapping.push((player, safe_player));
    }
    let mut bracket = DoubleEliminationBracket::create(
        Seeding::new(tournament.get_participants().get_seeding()).unwrap(),
        AutomaticMatchValidationMode::Flexible, // FIXME get from form
    );
    for r in bracket_state.results {
        let report = (r.score_p1, r.score_p2);
        let Some(p1_mapping) = safe_player_mapping.iter().find(|m| m.0.get_id() == r.p1_id) else {
            return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
        };
        let Some(p2_mapping) = safe_player_mapping.iter().find(|m| m.0.get_id() == r.p2_id) else {
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

    let mut transaction = pool.begin().await.map_err(internal_error)?;
    let user_id: totsugeki::player::Id = session
        .get(&Keys::UserId.to_string())
        .await
        .expect("value from store")
        .expect("user id");
    if let Err(e) =
        TournamentService::create(&mut transaction, &tournament, &bracket, user_id).await
    {
        tracing::error!("{e:?}");
        return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
    };

    transaction.commit().await.map_err(internal_error)?;

    tracing::info!("new bracket replayed from steps {}", tournament.get_id().0);
    tracing::debug!("new bracket replayed from steps {:?}", bracket);

    Ok((
        StatusCode::CREATED,
        breakdown(tournament, bracket, None, true),
    ))
}
