//! Register player in bracket

use crate::http::{internal_error, ErrorSlug};
use crate::repositories::brackets::{Error, TournamentService};
use crate::repositories::users::UserRepository;
use crate::tournaments::breakdown;
use crate::users::session::Keys::UserId;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use http::StatusCode;
use sqlx::PgPool;
use totsugeki::bracket::Id;
use totsugeki::player::PlayerID;
use totsugeki::ID;
use tower_sessions::Session;
use tracing::instrument;

/// Let user join bracket as a player
#[instrument(name = "join_bracket", skip(session, pool))]
pub(crate) async fn join_bracket(
    session: Session,
    Path(tournament_id): Path<Id>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    tracing::debug!("tournament {tournament_id}");
    let user_id: ID = session
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
    let (tournament, bracket, is_tournament_organiser) =
        // FIXME make all errors from totsugeki library simple to parse and not a big enum when some
        //  enum variants are simply irrelevant for some methods
        match TournamentService::join(&mut transaction, tournament_id, user).await {
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

    Ok(breakdown(
        &tournament,
        bracket,
        Some(PlayerID::new(user_id)),
        is_tournament_organiser,
    ))
}
