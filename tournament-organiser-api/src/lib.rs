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

use crate::registration::user_registration;
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::net::SocketAddr;
use totsugeki::bracket::Bracket;
use totsugeki::{
    bracket::double_elimination_variant::Variant as DoubleEliminationVariant,
    player::Id as PlayerId,
};
use totsugeki_display::loser_bracket::lines as loser_bracket_lines;
use totsugeki_display::loser_bracket::reorder as reorder_loser_bracket;
use totsugeki_display::winner_bracket::lines as winner_bracket_lines;
use totsugeki_display::winner_bracket::reorder as reorder_winner_bracket;
use totsugeki_display::{from_participants, BoxElement, MinimalMatch};
use tower_http::cors::CorsLayer;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod registration;

/// Name of the app
static APP: &str = "tournament organiser application";
/// Port to serve the app
static PORT: u16 = 3000;

// FIXME do not panic when submitting score for match with missing player

/// Health check response
#[derive(Serialize, Deserialize, Debug)]
pub struct HealthCheck {
    /// true if api is health
    pub ok: bool,
}

/// `/health_check` endpoint for health check
async fn health_check() -> impl IntoResponse {
    Json(HealthCheck { ok: true })
}

/// Bracket to display
#[derive(Serialize, Debug)]
struct BracketDisplay {
    /// Winner bracket matches and lines to draw
    winner_bracket: Vec<Vec<MinimalMatch>>,
    /// Lines to draw between winner bracket matches
    winner_bracket_lines: Vec<Vec<BoxElement>>,
    /// Loser bracket matches and lines to draw
    loser_bracket: Vec<Vec<MinimalMatch>>,
    /// Lines to draw between loser bracket matches
    loser_bracket_lines: Vec<Vec<BoxElement>>,
    /// Grand finals
    grand_finals: MinimalMatch,
    /// Grand finals reset
    grand_finals_reset: MinimalMatch,
    /// Bracket object to update
    bracket: Bracket,
}

/// List of players from which a bracket can be created
#[derive(Deserialize)]
pub struct PlayerList {
    /// player names
    names: Vec<String>,
}

/// Return a newly instanciated bracket from ordered (=seeded) player names
///
/// # Panics
/// When bracket cannot be converted to double elimination bracket
///
/// # Errors
/// May return 500 error when bracket cannot be parsed
pub async fn new_bracket_from_players(Json(player_list): Json<PlayerList>) -> impl IntoResponse {
    tracing::debug!("new bracket from players: {:?}", player_list.names);
    let mut bracket = Bracket::default();
    for name in player_list.names {
        let Ok(tmp) = bracket.add_participant(name.as_str()) else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        bracket = tmp;
    }
    let participants = bracket.get_participants();
    let dev: DoubleEliminationVariant = bracket.clone().try_into().expect("partition");

    // TODO test if tracing shows from which methods it was called
    let winner_bracket_matches = match dev.partition_winner_bracket() {
        Ok(wb) => wb,
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let mut winner_bracket_rounds = vec![];
    for r in winner_bracket_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        winner_bracket_rounds.push(round);
    }

    reorder_winner_bracket(&mut winner_bracket_rounds);
    let Some(winner_bracket_lines) = winner_bracket_lines(&winner_bracket_rounds) else {
        tracing::error!("winner bracket connecting lines");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(lower_bracket_matches) = dev.partition_loser_bracket() else {
        // TODO log error
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let mut loser_bracket_rounds: Vec<Vec<MinimalMatch>> = vec![];
    for r in lower_bracket_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        loser_bracket_rounds.push(round);
    }
    reorder_loser_bracket(&mut loser_bracket_rounds);
    let Some(loser_bracket_lines) = loser_bracket_lines(loser_bracket_rounds.clone()) else {
        tracing::error!("loser bracket connecting lines");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let (gf, gf_reset) = match dev.grand_finals_and_reset() {
        Ok((gf, bracket_reset)) => (gf, bracket_reset),
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let gf = from_participants(&gf, &participants);
    let gf_reset = from_participants(&gf_reset, &participants);

    let bracket = BracketDisplay {
        winner_bracket: winner_bracket_rounds,
        winner_bracket_lines,
        loser_bracket: loser_bracket_rounds,
        loser_bracket_lines,
        grand_finals: gf,
        grand_finals_reset: gf_reset,
        bracket,
    };
    tracing::debug!("updated bracket {:?}", bracket);
    Ok(Json(bracket))
}

/// List of players from which a bracket can be created
#[derive(Deserialize)]
pub struct ReportResultForBracket {
    /// current state of the bracket
    bracket: Bracket,
    /// First player
    p1_id: PlayerId,
    /// Second player
    p2_id: PlayerId,
    /// player 1 score
    score_p1: i8,
    /// player 2 score
    score_p2: i8,
}

/// Returns updated bracket with result. Because there is no persistence, it's
/// obviously limited in that TO can manipulate localStorage to change the
/// bracket but we are not worried about that right now. For now, the goal is
/// that it just works for normal use cases
///
/// # Panics
/// May panic if I fucked up
///
/// # Errors
/// Error 500 if a user gets out of sync with the bracket in the database and
/// the one displayed in the web page
pub async fn report_result_for_bracket(
    Json(report): Json<ReportResultForBracket>,
) -> impl IntoResponse {
    tracing::debug!("new reported result");
    let mut bracket = report.bracket;

    bracket = match bracket.tournament_organiser_reports_result(
        report.p1_id,
        (report.score_p1, report.score_p2),
        report.p2_id,
    ) {
        Ok((bracket, _, _)) => bracket,
        Err(e) => {
            // TODO deal with corner case where UI shows a bracket that is out
            // of sync with database state and returns something to user
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let participants = bracket.get_participants();
    let dev: DoubleEliminationVariant = bracket.clone().try_into().expect("partition");

    // TODO test if tracing shows from which methods it was called
    let winner_bracket_matches = match dev.partition_winner_bracket() {
        Ok(wb) => wb,
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let mut winner_bracket_rounds = vec![];
    for r in winner_bracket_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        winner_bracket_rounds.push(round);
    }

    reorder_winner_bracket(&mut winner_bracket_rounds);
    let Some(winner_bracket_lines) = winner_bracket_lines(&winner_bracket_rounds) else {
        tracing::error!("winner bracket connecting lines");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(lower_bracket_matches) = dev.partition_loser_bracket() else {
        // TODO log error
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let mut lower_bracket_rounds: Vec<Vec<MinimalMatch>> = vec![];
    for r in lower_bracket_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        lower_bracket_rounds.push(round);
    }
    reorder_loser_bracket(&mut lower_bracket_rounds);
    let Some(loser_bracket_lines) = loser_bracket_lines(lower_bracket_rounds.clone()) else {
        tracing::error!("loser bracket connecting lines");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let (gf, gf_reset) = match dev.grand_finals_and_reset() {
        Ok((gf, bracket_reset)) => (gf, bracket_reset),
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let gf = from_participants(&gf, &participants);
    let gf_reset = from_participants(&gf_reset, &participants);

    let bracket = BracketDisplay {
        winner_bracket: winner_bracket_rounds,
        winner_bracket_lines,
        loser_bracket: lower_bracket_rounds,
        loser_bracket_lines,
        grand_finals: gf,
        grand_finals_reset: gf_reset,
        bracket,
    };
    tracing::debug!("created bracket {:?}", bracket);
    Ok(Json(bracket))
}

/// Router for non-user facing endpoints. Web page makes requests to API (registration,
/// updating bracket...)
fn api(pool: Pool<Postgres>) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/register", post(user_registration))
        .route("/bracket-from-players", post(new_bracket_from_players))
        .route(
            "/report-result-for-bracket",
            post(report_result_for_bracket),
        )
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

/// Serve tournament organiser application
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

/// Run totsugeki application on `PORT`
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

/// Utility function for integration testing
/// NOTE: other people also have the same idea, see [link](https://stackoverflow.com/a/59090848)
pub mod test_utils {

    /// Test app with convenience methods to avoid test boilerplate
    ///
    /// IDEA: [link](https://youtu.be/_VB1fLLtZfQ?t=961)
    pub struct TestApp {
        /// http address of test app
        pub addr: String,
    }

    use crate::registration::FormInput;
    use reqwest::{Client, Response};

    use super::*;
    use std::net::TcpListener;
    /// Returns address to connect to new application (with random available port)
    ///
    /// Example: `http://0.0.0.0:43222`
    #[must_use]
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn spawn_app(db: PgPool) -> TestApp {
        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app(db).into_make_service())
                .await
                .unwrap();
        });

        TestApp {
            addr: format!("http://{addr}"),
        }
    }

    impl TestApp {
        /// register user through `/api/register` endpoint with a POST request
        #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
        pub async fn register(&self, request: &FormInput) -> Response {
            let client = Client::new();
            client
                .post(format!("{}/api/register", self.addr))
                .json(request)
                .send()
                .await
                .expect("request done")
        }
    }
}
