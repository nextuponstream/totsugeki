//! all API routes

use axum::routing::{delete, get, post};
use axum::Router;
use http::StatusCode;
use sqlx::{Pool, Postgres};
use tower_sessions_sqlx_store::PostgresStore;

use crate::health_check::health_check;
use crate::middlewares::authentication::{auth_layer, maybe_auth_layer};
use crate::tournaments::join_bracket;
use crate::tournaments::{
    create_tournament, list_brackets, new_bracket, save_tournament_from_steps, score, show_bracket,
    update_with_result, user_tournaments,
};
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
        "/tournaments",
        Router::new()
            .route("/", get(list_brackets))
            .route("/", post(create_tournament))
            .route("/save", post(save_tournament_from_steps))
            .route("/{tournament_id}/score", post(update_with_result))
            .route("/{tournament_id}/join", post(join_bracket)),
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
        .route("/report-result", post(score))
        .nest(
            "/guest",
            Router::new().route("/tournaments", post(new_bracket)),
        )
        .nest(
            "/user",
            Router::new().route("/{id}/tournaments", get(user_tournaments)),
        );
    let maybe_logged_in_routes = Router::new()
        .nest(
            "/tournaments",
            Router::new().route("/{tournament_id}", get(show_bracket)),
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
