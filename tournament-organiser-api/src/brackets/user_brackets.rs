//! Brackets from user

use crate::middlewares::validation::ValidatedRequest;
use crate::repositories::brackets::BracketRepository;
use crate::resources::{Pagination, PaginationResult};
use axum::extract::{Path, State};
use axum::Json;
use axum_macros::debug_handler;
use sqlx::PgPool;
use totsugeki::bracket::Id;
use tracing::instrument;

/// `/:user_id/brackets` GET to view brackets managed by user
#[instrument(name = "user_brackets", skip(pool))]
#[debug_handler]
pub(crate) async fn user_brackets(
    Path(user_id): Path<Id>,
    State(pool): State<PgPool>,
    ValidatedRequest(pagination): ValidatedRequest<Pagination>,
) -> crate::http::Result<Json<PaginationResult>> {
    let limit: i64 = pagination.limit.try_into().expect("ok");
    let offset: i64 = pagination.offset.try_into().expect("ok");

    let mut transaction = pool.begin().await?;
    let brackets = BracketRepository::user_brackets(
        &mut transaction,
        pagination.sort_order,
        limit,
        offset,
        user_id,
    )
    .await?;

    let total = if brackets.is_empty() {
        0
    } else {
        brackets[0].total.expect("total")
    };
    let total = total.try_into().expect("conversion");
    let data = brackets;
    let pagination_result = PaginationResult { total, data };

    transaction.commit().await?;

    Ok(Json(pagination_result))
}
