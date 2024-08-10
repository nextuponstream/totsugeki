//! all API routes

use axum::routing::{delete, get, post};
use axum::Router;
use http::StatusCode;
use sqlx::{Pool, Postgres};
use tower_sessions_sqlx_store::PostgresStore;

use crate::brackets::join::join_bracket;
use crate::brackets::{
    create_bracket, get_bracket, list_brackets, new_bracket, report_result,
    save_bracket_from_steps, update_with_result, user_brackets,
};
use crate::health_check::health_check;
use crate::middlewares::authentication::{auth_layer, maybe_auth_layer};
use crate::users::login::login;
use crate::users::logout::logout;
use crate::users::registration::registration;
use crate::users::{delete_user, profile};

/// Router for non-user facing endpoints. Web page makes requests to API
/// (registration, updating bracket...)
pub(crate) fn api(pool: Pool<Postgres>, session_store: PostgresStore) -> Router {
    // TODO declare that router in a new dashboard folder and import
    let user_routes = Router::new().nest(
        "/users",
        Router::new()
            .route("/", delete(delete_user))
            .route("/profile", get(profile)),
    );

    // TODO declare that router in brackets and import
    let bracket_routes = Router::new().nest(
        "/brackets",
        Router::new()
            .route("/", get(list_brackets))
            .route("/", post(create_bracket))
            .route("/save", post(save_bracket_from_steps))
            .route("/:bracket_id/report-result", post(update_with_result))
            .route("/:bracket_id/join", post(join_bracket)),
    );
    let protected_routes = Router::new()
        .merge(user_routes)
        .merge(bracket_routes)
        .layer(axum::middleware::from_fn_with_state(
            session_store.clone(),
            auth_layer,
        ));
    let unprotected_bracket_routes = Router::new()
        .route("/health_check", get(health_check))
        // TODO declare an auth router and merge routes
        .route("/register", post(registration))
        .route("/login", post(login))
        .route("/logout", post(logout))
        // TODO declare brackets_guest router and merge
        // FIXME naming is unclear, just say dry-run
        .route("/report-result", post(report_result))
        .nest(
            "/guest",
            Router::new().route("/brackets", post(new_bracket)),
        )
        .nest(
            "/user",
            Router::new().route("/:id/brackets", get(user_brackets)),
        );
    let maybe_logged_in_routes = Router::new()
        .nest(
            "/brackets",
            Router::new().route("/:bracket_id", get(get_bracket)),
        )
        .layer(axum::middleware::from_fn_with_state(
            session_store,
            maybe_auth_layer,
        ));

    let unprotected_routes = Router::new().merge(unprotected_bracket_routes);
    Router::new()
        .merge(unprotected_routes)
        .merge(maybe_logged_in_routes)
        .merge(protected_routes)
        .fallback_service(get(|| async { (StatusCode::NOT_FOUND, "Not found") }))
        .with_state(pool)
}
