//! Register player in bracket

use crate::brackets::breakdown;
use crate::repositories::brackets::{BracketRepository, Error};
use crate::repositories::users::UserRepository;
use crate::users::session::Keys::UserId;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use sqlx::PgPool;
use totsugeki::bracket::Error as TotsugekiError;
use totsugeki::bracket::Id;
use totsugeki::player::Error as PlayerError;
use tower_sessions::Session;
use tracing::instrument;

#[instrument(name = "join_bracket", skip(session, pool))]
pub async fn join_bracket(
    session: Session,
    Path(bracket_id): Path<Id>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    tracing::debug!("bracket {bracket_id}");
    let user_id: totsugeki::player::Id = session
        .get(&UserId.to_string())
        .await
        .expect("key")
        .expect("user id");

    let mut transaction = pool.begin().await.unwrap(); // FIXME no unwrap
    let user = match UserRepository::read(&mut transaction, user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => return (StatusCode::NOT_FOUND).into_response(),
        Err(e) => {
            tracing::error!("{e:?}");
            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };
    let (bracket, is_tournament_organiser) =
        match BracketRepository::join(&mut transaction, bracket_id, user).await {
            Ok(Some(data)) => data,
            Ok(None) => return (StatusCode::NOT_FOUND).into_response(),
            Err(Error::Bracket(TotsugekiError::PlayerUpdate(PlayerError::AlreadyPresent))) => {
                return (StatusCode::BAD_REQUEST).into_response();
            }
            Err(e) => {
                tracing::error!("{e:?}");
                return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
            }
        };

    return breakdown(bracket, Some(user_id), is_tournament_organiser).into_response();
}
