//! Bracket management and visualiser library for admin dashboard
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]
#![forbid(unsafe_code)]

pub mod brackets;
pub mod health_check;
pub mod login;
pub mod logout;
pub mod registration;
pub mod resources;
pub mod test_utils;
pub mod users;

use crate::health_check::health_check;
use crate::login::login;
use crate::logout::logout;
use crate::registration::registration;
use crate::users::profile;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{
    routing::{delete, get, post},
    Router,
};
use brackets::{
    create_bracket, get_bracket, get_bracket_display, list_brackets, new_bracket, report_result,
    save_bracket,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::net::SocketAddr;
use time::Duration;
use tokio::net::TcpListener;
use totsugeki::player::Id;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tower_sessions::Session;
use tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use users::delete_user;

/// Name of the app
static APP: &str = "tournament organiser application";
/// Port to serve the app. By default, we set what flyio is expecting as
// default listening port
static PORT: u16 = 8080;

// FIXME do not panic when submitting score for match with missing player

/// Auth layer checking for presence of key `user_id` in session, set by login
/// endpoint.
///
/// NOTE: if you need multi auth layer, reach out for `axum_login` crate. Until
///       then, relying on a key being present in session is enough.
async fn auth_layer(
    // State(state): State<AppState>,
    session: Session,
    // you can add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request,
    next: Next,
) -> Response {
    let v: Option<Id> = session.get("user_id").await.expect("value from store");
    if v.is_none() {
        tracing::warn!(
            "unauthenticated request against protected route /api{} {}",
            request.uri(),
            request.method()
        );
        return (StatusCode::UNAUTHORIZED).into_response();
    };
    // do something with `request`...

    // do something with `response`...

    next.run(request).await
}

/// Router for non-user facing endpoints. Web page makes requests to API
/// (registration, updating bracket...)
fn api(pool: Pool<Postgres>, session_store: PostgresStore) -> Router {
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
            .route("/save", post(save_bracket))
            .route("/:bracket_id", get(get_bracket)),
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
            "/brackets",
            Router::new().route("/:bracket_id/display", get(get_bracket_display)),
        );
    let unprotected_routes = Router::new().merge(unprotected_bracket_routes);
    Router::new()
        .merge(unprotected_routes)
        .merge(protected_routes)
        .fallback_service(get(|| async { (StatusCode::NOT_FOUND, "Not found") }))
        .with_state(pool)
}

/// Serve web part of the application, using `tournament-organiser-web` build
/// For development, Cors rule are relaxed
pub fn app(pool: Pool<Postgres>, session_store: PostgresStore) -> Router {
    let web_build_path = std::env::var("BUILD_PATH_TOURNAMENT_ORGANISER_WEB").unwrap_or_else(|e| {
        tracing::info!("BUILD_PATH_TOURNAMENT_ORGANISER_WEB could not be parsed. Defaulting to relative path: {e}");
        "../tournament-organiser-web/dist".into()
    });

    let spa = ServeDir::new(web_build_path.clone())
        // .not_found_service will throw 404, which makes cypress test fail
        .fallback(ServeFile::new(format!("{web_build_path}/index.html")));
    Router::new()
        .nest("/api", api(pool, session_store))
        .nest_service("/dist", spa.clone())
        // Show vue app
        .fallback_service(spa)
}

/// Serve tournament organiser application. Listening address is:
/// * Developpement: 127.0.0.1:<PORT>
/// * Docker image : 0.0.0.0:<PORT>
async fn serve(app: Router, port: u16) {
    let addr = match std::env::var("DOCKER_BUILD") {
        Ok(_) => SocketAddr::from(([0, 0, 0, 0], port)),
        Err(_) => SocketAddr::from(([127, 0, 0, 1], port)),
    };
    let listener = TcpListener::bind(addr).await.expect("address to listen to");
    tracing::debug!("listening on {}", addr);
    axum::serve(
        listener,
        app.layer(TraceLayer::new_for_http()).into_make_service(),
    )
    .await
    .expect("running http server");
}

/// Run totsugeki application on `PORT`. Set up tracing and database connection
/// url (either from `DATABASE_URL` or `DB_USERNAME` + `DB_PASSWORD` +
/// `DB_NAME`)
///
/// # Panics
/// When database connection credentials are not set
pub async fn run() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                // example_static_file_server=debug,tower_http=debug
                // you can append this tower_http=debug to see more details
                // .unwrap_or_else(|_| "INFO,tower_http=debug".into()),
                .unwrap_or_else(|_| "INFO".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    tracing::debug!("setting up database...");
    let db_url = if let Ok(url) = std::env::var("DATABASE_URL") {
        url
    } else {
        let db_username =
            std::env::var("DB_USERNAME").expect("DB_USERNAME environment variable set");
        let db_password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD");
        let db_name = std::env::var("DB_NAME").expect("DB_NAME");

        format!("postgres://{db_username}:{db_password}@localhost/{db_name}")
    };
    // FIXME make db url connection optionnal, otherwise first time deploy
    // might be painful (like not provisionning right away a db deletes the
    // app from the hosting site for reason because it's stuck in crash loop)
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Database is unreachable: {e}");
            panic!("Database is unreachable: {e}");
        }
    };

    // If you want to test if any migrations were run, try to login and see if
    // any panic were logged

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.expect("session store");

    let _deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::hours(1)));

    let port = if let Ok(port) = std::env::var("PORT") {
        port.parse().expect("port")
    } else {
        tracing::warn!("PORT not set or error, defaulting to {PORT}");
        PORT
    };
    tracing::info!("Serving {APP} on http://localhost:{port}");
    serve(app(pool, session_store).layer(session_layer), port).await;
}

/// Standard error message
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    /// user-facing error message
    pub message: String,
}
