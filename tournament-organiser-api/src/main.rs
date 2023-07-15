#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]
#![forbid(unsafe_code)]

use axum::{response::IntoResponse, routing::get, Json, Router};
use http::Method;
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                // example_static_file_server=debug,tower_http=debug
                // you can append this tower_http=debug to see more details
                .unwrap_or_else(|_| "INFO,".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting tournament organiser application...");
    tokio::join!(serve(using_serve_dir_with_assets_fallback(), 3000),);
}

/// Serve web part of the application, using `tournament-organiser-web` build
/// For development, Cors rule are relaxed
fn using_serve_dir_with_assets_fallback() -> Router {
    let web_build_path = match std::env::var("BUILD_PATH_TOURNAMENT_ORGANISER_WEB") {
        Ok(s) => s,
        Err(e) => {
            tracing::info!("BUILD_PATH_TOURNAMENT_ORGANISER_WEB could not be parsed. Defaulting to relative path: {e}");
            "../tournament-organiser-web/dist".into()
        }
    };

    let serve_dir =
        ServeDir::new(web_build_path).not_found_service(ServeFile::new("dist/index.html"));

    Router::new()
        .route("/foo", get(|| async { "Hi from /foo" }))
        .route("/health", get(health))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers([http::header::CONTENT_TYPE])
                .allow_methods([Method::GET]),
        )
        .nest_service("/dist", serve_dir.clone())
        .fallback_service(serve_dir)
}

/// Health check response
#[derive(Serialize)]
struct HealthCheck {
    /// true if api is health
    ok: bool,
}

/// /health endpoint for health check
async fn health() -> impl IntoResponse {
    Json(HealthCheck { ok: true })
}

/// Serve tournament organiser application
async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await
        .expect("server serving tournament-organiser application");
}
