//! Save bracket from steps

use crate::brackets::{breakdown, BracketState};
use crate::http::{internal_error, ErrorSlug};
use crate::repositories::brackets::BracketRepository;
use crate::users::session::Keys;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use axum_macros::debug_handler;
use http::StatusCode;
use sqlx::PgPool;
use totsugeki::bracket::Bracket;
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

    let mut bracket = Bracket::default();
    bracket = bracket.update_name(bracket_state.bracket_name);
    let mut safe_player_mapping = vec![];
    // Do not rely on given ID, assign new IDs to players and map
    for player in bracket_state.players {
        let Ok(tmp) = bracket.add_participant(player.get_name().as_str()) else {
            tracing::warn!("oh no");
            return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
        };
        bracket = tmp.0;
        safe_player_mapping.push((player, tmp.1));
    }
    let mut bracket = match bracket.start() {
        Ok(b) => b.0,
        Err(err) => {
            tracing::warn!("{err}");
            return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };
    // let mut bracket = bracket.0;
    for r in bracket_state.results {
        let report = (r.score_p1, r.score_p2);
        let Some(p1_mapping) = safe_player_mapping.iter().find(|m| m.0.get_id() == r.p1_id) else {
            return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
        };
        let Some(p2_mapping) = safe_player_mapping.iter().find(|m| m.0.get_id() == r.p2_id) else {
            return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
        };
        bracket = match bracket.tournament_organiser_reports_result(
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
    if let Err(e) = BracketRepository::create(&mut transaction, &bracket, user_id).await {
        tracing::error!("{e:?}");
        return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
    };

    transaction.commit().await.map_err(internal_error)?;

    tracing::info!("new bracket replayed from steps {}", bracket.get_id());
    tracing::debug!("new bracket replayed from steps {:?}", bracket);

    Ok((StatusCode::CREATED, breakdown(bracket, None, true)))
}
