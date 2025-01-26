//! Register player in bracket

use crate::http::ErrorSlug;
use crate::repositories::brackets::Error;
use crate::repositories::users::UserRepository;
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

/// Let user join bracket as a player
#[instrument(name = "join_bracket", skip(session, pool))]
pub(crate) async fn join_bracket<'a>(
    session: Session,
    Path(tournament_id): Path<Id>,
    State(pool): ConnectionPool,
) -> impl IntoResponse {
    tracing::debug!("tournament {tournament_id}");
    let user_id: ID = session
        .get(&UserId.to_string())
        .await
        .expect("ID")
        .expect("user ID");
    let user_id = ID::from(user_id);

    let mut transaction = pool.begin().await?;
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
            Err(Error::Sqlx(e)) => {
                return Err(e.into());
            }
        };

    Ok(breakdown(
        &tournament,
        bracket,
        Some(user_id),
        is_tournament_organiser,
    ))
}
