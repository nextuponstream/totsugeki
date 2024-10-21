//! Show bracket

use crate::brackets::breakdown;
use crate::http::{internal_error, ErrorSlug};
use crate::repositories::brackets::BracketRepository;
use crate::users::session::Keys::UserId;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use sqlx::PgPool;
use totsugeki::bracket::Id;
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
    Path(bracket_id): Path<Id>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    tracing::debug!("bracket {bracket_id}");
    let user_id: Option<totsugeki::player::Id> = session
        .get(&UserId.to_string())
        .await
        .expect("maybe id of user");

    let mut transaction = pool.begin().await.map_err(internal_error)?;
    let (bracket, is_tournament_organiser) =
    // FIXME wrong error type
        match BracketRepository::read_for_user(&mut transaction, bracket_id, user_id).await {
            Ok(Some(data)) => data,
            Ok(None) => return Err(ErrorSlug::from(StatusCode::NOT_FOUND)),
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
            }
        };

    transaction.commit().await.map_err(internal_error)?;
    Ok((
        StatusCode::OK,
        breakdown(bracket, user_id, is_tournament_organiser),
    ))
}
