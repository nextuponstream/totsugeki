//! Register player in bracket

use crate::users::session::Keys::UserId;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use sqlx::PgPool;
use totsugeki::bracket::Id;
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
    todo!();
    // let userRepo = UserRepository::new(&mut transaction);
    // let user = match userRepo.read(user_id).await {
    //     Ok(user) => user,
    //     Err(e) => todo!(),
    // };
    todo!()
    // let repo = BracketRepository::for_transaction(&mut transaction);
    // let (bracket, is_tournament_organiser) = match repo.join(bracket_id, user).await {
    //     Ok(Some(data)) => data,
    //     Ok(None) => return (StatusCode::NOT_FOUND).into_response(),
    //     Err(e) => {
    //         tracing::error!("{e:?}");
    //         return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    //     }
    // };
    //
    // return breakdown(bracket, Some(user_id), is_tournament_organiser).into_response();
}
