//! Register player in bracket

use crate::brackets::breakdown;
use crate::http::{internal_error, ErrorSlug};
use crate::repositories::brackets::{BracketRepository, Error};
use crate::repositories::users::UserRepository;
use crate::users::session::Keys::UserId;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use sqlx::PgPool;
use totsugeki::bracket::Id;
use tower_sessions::Session;
use tracing::instrument;

/// Let user join bracket as a player
#[instrument(name = "join_bracket", skip(session, pool))]
pub(crate) async fn join_bracket(
    session: Session,
    Path(bracket_id): Path<Id>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    tracing::debug!("bracket {bracket_id}");
    let user_id: totsugeki::player::Id = session
        .get(&UserId.to_string())
        .await
        .map_err(internal_error)?
        .ok_or_else(|| {
            tracing::error!("missing user id");
            ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    let mut transaction = pool.begin().await.map_err(internal_error)?;
    let user = match UserRepository::read(&mut transaction, user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(ErrorSlug::from(StatusCode::NOT_FOUND)),
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };
    let (bracket, is_tournament_organiser) =
        // FIXME make all errors from totsugeki library simple to parse and not a big enum when some
        //  enum variants are simply irrelevant for some methods
        match BracketRepository::join(&mut transaction, bracket_id, user).await {
            Ok(Some(data)) => data,
            Ok(None) => return Err(ErrorSlug::from(StatusCode::NOT_FOUND)),
            Err(Error::PlayerAlreadyPresent)=> {
                return Err(ErrorSlug::new(StatusCode::CONFLICT, "player-already-present"));
            }
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
            }
        };

    return Ok(breakdown(bracket, Some(user_id), is_tournament_organiser));
}
