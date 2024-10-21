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
pub mod http;
mod middlewares;
pub(crate) mod repositories;
pub mod resources;
mod router;
pub mod test_utils;
pub mod users;

use crate::router::api;
use axum::Router;
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
use tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Name of the app
static APP: &str = "tournament organiser application";
/// Port to serve the app. By default, we set what flyio is expecting as
/// default listening port
static PORT: u16 = 8080;

/// Expected env value for boolean configuration variable
static ENABLED: &str = "true";

// FIXME do not panic when submitting score for match with missing player

/// Serve web part of the application, using `tournament-organiser-web` build
pub fn app(pool: Pool<Postgres>, session_store: PostgresStore) -> Router {
    let web_build_path = std::env::var("BUILD_PATH_TOURNAMENT_ORGANISER_WEB").unwrap_or_else(|e| {
        tracing::info!("BUILD_PATH_TOURNAMENT_ORGANISER_WEB could not be parsed. Defaulting to relative path: {e}");
        "../tournament-organiser-web/dist".into()
    });

    let spa = ServeDir::new(web_build_path.clone())
        // .not_found_service will throw 404, which makes cypress test fail
        .fallback(ServeFile::new(format!("{web_build_path}/index.html")));
    if std::env::var("DEVELOPMENT").is_ok_and(|v| v == ENABLED) {
        // Vite dev server is used. This is to avoid confusion when app is
        // served at default port AND on the vite dev server
        Router::new().nest("/api", api(pool, session_store))
    } else {
        Router::new()
            .nest("/api", api(pool, session_store))
            .nest_service("/dist", spa.clone())
            // Show vue app
            .fallback_service(spa)
    }
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
                // TODO set session to its own layer at trace level
                // TODO set pool to its own layer at trace level
                // use https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/index.html#per-layer-filtering
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

    // could be a utility crate with more features but this suffice
    // Currently returning 401 only
    let session_duration = if let Ok(duration) = std::env::var("SESSION_DURATION") {
        let duration = duration.parse().expect("session duration in seconds");
        tracing::info!("Session duration {duration}s");
        duration
    } else {
        let default_session_duration = 3600;
        tracing::warn!("Default session duration set ({default_session_duration})");
        default_session_duration
    };
    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(session_duration)));

    let port = if let Ok(port) = std::env::var("PORT") {
        port.parse().expect("port")
    } else {
        tracing::warn!("PORT not set or error, defaulting to {PORT}");
        PORT
    };
    if std::env::var("DEVELOPMENT").is_ok_and(|v| v == ENABLED) {
        tracing::warn!(
            "Development server used for hosting SPA. Checkout tournament-organiser-web..."
        );
    } else {
        tracing::info!("Serving {APP} on http://localhost:{port}");
    }
    tracing::debug!("Serving {APP} API on http://localhost:{port}/api");

    serve(app(pool, session_store).layer(session_layer), port).await;
}

/// Standard error message
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    /// user-facing error message
    pub message: String,
}
