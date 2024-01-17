//! Bracket management and visualiser library for admin dashboard
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
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

/// /health_check endpoint for health check
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
    let wb_rounds_matches = match dev.partition_winner_bracket() {
        Ok(wb) => wb,
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let mut wb_rounds = vec![];
    for r in wb_rounds_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        wb_rounds.push(round);
    }

    reorder_winner_bracket(&mut wb_rounds);
    let Some(winner_bracket_lines) = winner_bracket_lines(&wb_rounds) else {
        tracing::error!("winner bracket connecting lines");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(lb_rounds_matches) = dev.partition_loser_bracket() else {
        // TODO log error
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let mut lb_rounds: Vec<Vec<MinimalMatch>> = vec![];
    for r in lb_rounds_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        lb_rounds.push(round);
    }
    reorder_loser_bracket(&mut lb_rounds);
    let Some(loser_bracket_lines) = loser_bracket_lines(lb_rounds.clone()) else {
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
        winner_bracket: wb_rounds,
        winner_bracket_lines,
        loser_bracket: lb_rounds,
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
    p1_id: PlayerId,
    p2_id: PlayerId,
    score_p1: i8,
    score_p2: i8,
}

/// Returns updated bracket with result. Because there is no persistence, it's
/// obviously limited in that TO can manipulate localStorage to change the
/// bracket but we are not worried about that right now. For now, the goal is
/// that it just works for normal use cases
pub async fn report_result_for_bracket(
    Json(report): Json<ReportResultForBracket>,
) -> impl IntoResponse {
    tracing::debug!("new reported result");
    let mut bracket = report.bracket;

    // FIXME deal with error
    bracket = bracket
        .tournament_organiser_reports_result(
            report.p1_id,
            (report.score_p1, report.score_p2),
            report.p2_id,
        )
        .unwrap()
        .0;

    let participants = bracket.get_participants();
    let dev: DoubleEliminationVariant = bracket.clone().try_into().expect("partition");

    // TODO test if tracing shows from which methods it was called
    let wb_rounds_matches = match dev.partition_winner_bracket() {
        Ok(wb) => wb,
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let mut wb_rounds = vec![];
    for r in wb_rounds_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        wb_rounds.push(round);
    }

    reorder_winner_bracket(&mut wb_rounds);
    let Some(winner_bracket_lines) = winner_bracket_lines(&wb_rounds) else {
        tracing::error!("winner bracket connecting lines");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(lb_rounds_matches) = dev.partition_loser_bracket() else {
        // TODO log error
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let mut lb_rounds: Vec<Vec<MinimalMatch>> = vec![];
    for r in lb_rounds_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        lb_rounds.push(round);
    }
    reorder_loser_bracket(&mut lb_rounds);
    let Some(loser_bracket_lines) = loser_bracket_lines(lb_rounds.clone()) else {
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
        winner_bracket: wb_rounds,
        winner_bracket_lines,
        loser_bracket: lb_rounds,
        loser_bracket_lines,
        grand_finals: gf,
        grand_finals_reset: gf_reset,
        bracket,
    };
    tracing::debug!("created bracket {:?}", bracket);
    Ok(Json(bracket))
}

fn api() -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/bracket-from-players", post(new_bracket_from_players))
        .route(
            "/report-result-for-bracket",
            post(report_result_for_bracket),
        )
        .fallback_service(get(|| async { (StatusCode::NOT_FOUND, "Not found") }))
}

/// Serve web part of the application, using `tournament-organiser-web` build
/// For development, Cors rule are relaxed
pub fn app() -> Router {
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
        .nest("/api", api())
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
        .unwrap()
}

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

    tracing::info!("Serving {APP} on http://localhost:{PORT}");
    serve(app(), PORT).await
}
