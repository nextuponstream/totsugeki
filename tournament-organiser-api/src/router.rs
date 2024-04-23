//! all API routes

use crate::auth_layer;
use crate::brackets::{
    create_bracket, get_bracket, list_brackets, new_bracket, report_result, save_bracket,
    user_brackets,
};
use crate::health_check::health_check;
use crate::login::login;
use crate::logout::logout;
use crate::registration::registration;
use crate::users::{delete_user, profile};
use axum::routing::{delete, get, post};
use axum::Router;
use http::StatusCode;
use sqlx::{Pool, Postgres};
use tower_sessions_sqlx_store::PostgresStore;

/// Router for non-user facing endpoints. Web page makes requests to API
/// (registration, updating bracket...)
pub(crate) fn api(pool: Pool<Postgres>, session_store: PostgresStore) -> Router {
    // TODO declare that router in a new users folder and import
    let user_routes = Router::new().nest(
        "/users",
        Router::new()
            .route("/", get(profile))
            .route("/", delete(delete_user)),
    );

    // TODO declare that router in brackets and import
    let bracket_routes = Router::new().nest(
        "/brackets",
        Router::new()
            .route("/", post(create_bracket))
            .route("/", get(list_brackets))
            .route("/save", post(save_bracket)),
    );
    let protected_routes = Router::new()
        .merge(user_routes)
        .merge(bracket_routes)
        .layer(axum::middleware::from_fn_with_state(
            session_store,
            auth_layer,
        ));
    let unprotected_bracket_routes = Router::new()
        .route("/health_check", get(health_check))
        // TODO declare an auth router and merge routes
        .route("/register", post(registration))
        .route("/login", post(login))
        .route("/logout", post(logout))
        // TODO declare brackets_guest router and merge
        .route("/report-result-for-bracket", post(report_result))
        .nest(
            "/guest",
            Router::new().route("/brackets", post(new_bracket)),
        )
        .nest(
            "/user",
            Router::new().route("/:id/brackets", get(user_brackets)),
        )
        .nest(
            "/brackets",
            Router::new().route("/:bracket_id", get(get_bracket)),
        );
    let unprotected_routes = Router::new().merge(unprotected_bracket_routes);
    Router::new()
        .merge(unprotected_routes)
        .merge(protected_routes)
        .fallback_service(get(|| async { (StatusCode::NOT_FOUND, "Not found") }))
        .with_state(pool)
}
