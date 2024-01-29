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

mod bracket;
pub mod health_check;
pub mod login;
pub mod registration;
pub mod test_utils;

use crate::health_check::health_check;
use crate::registration::registration;
use crate::login::login;
use axum::{
    routing::{get, post},
    Router,
};
use bracket::{new_bracket_from_players, report_result};
use http::StatusCode;
use sqlx::PgPool;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Name of the app
static APP: &str = "tournament organiser application";
/// Port to serve the app
static PORT: u16 = 3000;

// FIXME do not panic when submitting score for match with missing player

/// Router for non-user facing endpoints. Web page makes requests to API
/// (registration, updating bracket...)
fn api(pool: Pool<Postgres>) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/register", post(registration))
        .route("/login", post(login))
        .route("/bracket-from-players", post(new_bracket_from_players))
        .route("/report-result-for-bracket", post(report_result))
        .fallback_service(get(|| async { (StatusCode::NOT_FOUND, "Not found") }))
        .with_state(pool)
}

/// Serve web part of the application, using `tournament-organiser-web` build
/// For development, Cors rule are relaxed
pub fn app(pool: Pool<Postgres>) -> Router {
    let web_build_path = match std::env::var("BUILD_PATH_TOURNAMENT_ORGANISER_WEB") {
        Ok(s) => s,
        Err(e) => {
            tracing::info!("BUILD_PATH_TOURNAMENT_ORGANISER_WEB could not be parsed. Defaulting to relative path: {e}");
            "../tournament-organiser-web/dist".into()
        }
    };

    let spa = ServeDir::new(web_build_path.clone())
        // .not_found_service will throw 404, which makes cypress test fail
        .fallback(ServeFile::new(format!("{web_build_path}/index.html")));

    Router::new()
        .nest("/api", api(pool))
        .nest_service("/dist", spa.clone())
        // Show vue app
        .fallback_service(spa)
        // CORS after route declaration https://github.com/tokio-rs/axum/issues/1330#issue-1351827022
        // this allows npm run dev from localhost:5173 to work with api at
        // localhost:3000
        .layer(
            // FIXME remove after end of development
            CorsLayer::very_permissive(),
        )
}

/// Serve tournament organiser application. Listening address is:
/// * Developpement: 127.0.0.1:<PORT>
/// * Docker image : 0.0.0.0:<PORT>
async fn serve(app: Router, port: u16) {
    let addr = match std::env::var("DOCKER_BUILD") {
        Ok(_) => SocketAddr::from(([0, 0, 0, 0], port)),
        Err(_) => SocketAddr::from(([127, 0, 0, 1], port)),
    };
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
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
    let db_url = if let Ok(url) = std::env::var("DATABASE_URL") {
        url
    } else {
        let db_username =
            std::env::var("DB_USERNAME").expect("DB_USERNAME environment variable set");
        let db_password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD");
        let db_name = std::env::var("DB_NAME").expect("DB_NAME");

        format!("postgres://{db_username}:{db_password}@localhost/{db_name}")
    };
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("database connection pool");

    tracing::info!("Serving {APP} on http://localhost:{PORT}");
    serve(app(pool), PORT).await;
}
