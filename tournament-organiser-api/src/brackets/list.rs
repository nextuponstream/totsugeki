//! List brackets

use crate::middlewares::validation::ValidatedRequest;
use crate::repositories::brackets::BracketRepository;
use crate::resources::{Pagination, PaginationResult};
use axum::extract::State;
use axum::Json;
use sqlx::PgPool;
use tracing::instrument;

/// Return a newly instanciated bracket from ordered (=seeded) player names
#[instrument(name = "list_brackets", skip(pool))]
pub(crate) async fn list_brackets(
    // NOTE pool before validated query params for some reason???
    State(pool): State<PgPool>,
    ValidatedRequest(pagination): ValidatedRequest<Pagination>,
) -> crate::http::Result<Json<PaginationResult>> {
    let limit: i64 = pagination.limit.try_into().expect("ok");
    let offset: i64 = pagination.offset.try_into().expect("ok");

    let mut transaction = pool.begin().await?;
    let brackets =
        BracketRepository::list(&mut transaction, pagination.sort_order, limit, offset).await?;
    let data = brackets;
    let pagination_result = PaginationResult { total: 100, data };
    transaction.commit().await?;

    Ok(Json(pagination_result))
}
