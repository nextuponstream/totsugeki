//! Show bracket

use crate::http::ErrorSlug;
use crate::services::tournaments::TournamentService;
use crate::tournaments::breakdown;
use crate::types::ConnectionPool;
use crate::users::session::Keys::UserId;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use totsugeki_core::bracket::Id;
use totsugeki_core::ID;
use tower_sessions::Session;
use tracing::instrument;

/// Returns existing bracket for display purposes
///
/// # Panics
/// When bracket cannot be converted to double elimination bracket
///
/// # Errors
/// May return 500 error when bracket cannot be parsed
#[instrument(name = "show_bracket", skip(session, pool))]
pub async fn show_bracket(
    session: Session,
    Path(tournament_id): Path<Id>,
    State(pool): ConnectionPool,
) -> impl IntoResponse {
    tracing::debug!("tournament {tournament_id}");
    let user_id: Option<ID> = session
        .get(&UserId.to_string())
        .await
        .expect("maybe id of user");

    let mut transaction = pool.begin().await?;
    let (tournament, bracket, is_tournament_organiser) =
    // FIXME wrong error type
        match TournamentService::read_for_user(&mut transaction, tournament_id, user_id).await {
            Ok(Some(data)) => data,
            Ok(None) => return Err(ErrorSlug::from(StatusCode::NOT_FOUND)),
            Err(e) => {
                return Err(e.into());
            }
        };

    transaction.commit().await?;
    Ok((
        StatusCode::OK,
        breakdown(&tournament, bracket, user_id, is_tournament_organiser),
    ))
}
