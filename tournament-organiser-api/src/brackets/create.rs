//! Create brackets

use crate::brackets::{CreateBracketForm, GenericResourceCreated};
use crate::http::internal_error;
use crate::middlewares::validation::ValidatedJson;
use crate::repositories::brackets::BracketRepository;
use crate::users::session::Keys::UserId;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json as AxumJson;
use axum_macros::debug_handler;
use http::StatusCode;
use sqlx::PgPool;
use totsugeki::bracket::Bracket;
use tower_sessions::Session;
use tracing::instrument;

/// Return a newly instanciated bracket from ordered (=seeded) player names
#[instrument(name = "create_bracket", skip(pool, session))]
#[debug_handler]
pub(crate) async fn create_bracket(
    session: Session,
    State(pool): State<PgPool>,
    ValidatedJson(form): ValidatedJson<CreateBracketForm>,
) -> impl IntoResponse {
    tracing::debug!("new bracket from players: {:?}", form.player_names);

    let mut transaction = pool.begin().await.map_err(internal_error).unwrap();
    // TODO refactor user_id key in SESSION_KEY enum
    let user_id: totsugeki::player::Id =
        session.get(&UserId.to_string()).await.expect("").expect("");
    let mut bracket = Bracket::default();
    for name in form.player_names {
        let tmp = bracket.add_participant(name.as_str()).unwrap();
        bracket = tmp.0;
    }
    let bracket = bracket.update_name(form.bracket_name);

    BracketRepository::create(&mut transaction, &bracket, user_id)
        .await
        .unwrap();

    transaction.commit().await.map_err(internal_error).unwrap();

    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    tracing::info!("new bracket {}", bracket.get_id());

    tracing::debug!("new bracket {:?}", bracket);
    (
        StatusCode::CREATED,
        AxumJson(GenericResourceCreated {
            id: bracket.get_id(),
        }),
    )
}
