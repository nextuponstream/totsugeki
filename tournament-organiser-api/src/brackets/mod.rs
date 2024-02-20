//! bracket management

use crate::resources::{GenericResource, GenericResourcesList, Pagination, ValidatedQueryParams};
use axum::extract::{Path, State};
use axum::{debug_handler, response::IntoResponse, Json as AxumJson};
use chrono::prelude::*;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::types::Json as SqlxJson;
use sqlx::PgPool;
use totsugeki::bracket::{
    double_elimination_variant::Variant as DoubleEliminationVariant, Bracket, Id,
};
use totsugeki::matches::Match;
use totsugeki::player::Id as PlayerId;
use totsugeki_display::loser_bracket::lines as loser_bracket_lines;
use totsugeki_display::loser_bracket::reorder as reorder_loser_bracket;
use totsugeki_display::winner_bracket::lines as winner_bracket_lines;
use totsugeki_display::winner_bracket::reorder as reorder_winner_bracket;
use totsugeki_display::{from_participants, BoxElement, MinimalMatch};
use tracing::instrument;

/// List of players from which a bracket can be created
#[derive(Deserialize)]
pub struct ReportResultInput {
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
#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerList {
    /// player names
    pub names: Vec<String>,
}

/// Return a newly instanciated bracket from ordered (=seeded) player names
///
/// # Panics
/// When bracket cannot be converted to double elimination bracket
///
/// # Errors
/// May return 500 error when bracket cannot be parsed
#[instrument(name = "get_bracket_display")]
pub async fn get_bracket_display(
    Path(bracket_id): Path<Id>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    tracing::debug!("bracket {bracket_id}");

    let Some(b) = sqlx::query_as!(
        BracketRecord,
        r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at  from brackets WHERE id = $1"#,
        bracket_id,
    )
    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    .fetch_optional(&pool)
    .await
    .expect("fetch result") else {
        return (StatusCode::NOT_FOUND).into_response();
    };
    todo!();
    // let bracket: Bracket = b.into();
    // let dev: DoubleEliminationVariant = bracket.clone().try_into().expect("partition");

    // // TODO test if tracing shows from which methods it was called
    // let winner_bracket_matches = match dev.partition_winner_bracket() {
    //     Ok(wb) => wb,
    //     Err(e) => {
    //         tracing::error!("{e:?}");
    //         return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    //     }
    // };
    // let mut winner_bracket_rounds = vec![];
    // for r in winner_bracket_matches {
    //     let round = r
    //         .iter()
    //         .map(|m| from_participants(m, &participants))
    //         .collect();
    //     winner_bracket_rounds.push(round);
    // }

    // reorder_winner_bracket(&mut winner_bracket_rounds);
    // let Some(winner_bracket_lines) = winner_bracket_lines(&winner_bracket_rounds) else {
    //     tracing::error!("winner bracket connecting lines");
    //     return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    // };

    // let Ok(lower_bracket_matches) = dev.partition_loser_bracket() else {
    //     // TODO log error
    //     return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    // };
    // let mut loser_bracket_rounds: Vec<Vec<MinimalMatch>> = vec![];
    // for r in lower_bracket_matches {
    //     let round = r
    //         .iter()
    //         .map(|m| from_participants(m, &participants))
    //         .collect();
    //     loser_bracket_rounds.push(round);
    // }
    // reorder_loser_bracket(&mut loser_bracket_rounds);
    // let Some(loser_bracket_lines) = loser_bracket_lines(loser_bracket_rounds.clone()) else {
    //     tracing::error!("loser bracket connecting lines");
    //     return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    // };

    // let (gf, gf_reset) = match dev.grand_finals_and_reset() {
    //     Ok((gf, bracket_reset)) => (gf, bracket_reset),
    //     Err(e) => {
    //         tracing::error!("{e:?}");
    //         return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
    //     }
    // };
    // let gf = from_participants(&gf, &participants);
    // let gf_reset = from_participants(&gf_reset, &participants);

    // let bracket = BracketDisplay {
    //     winner_bracket: winner_bracket_rounds,
    //     winner_bracket_lines,
    //     loser_bracket: loser_bracket_rounds,
    //     loser_bracket_lines,
    //     grand_finals: gf,
    //     grand_finals_reset: gf_reset,
    //     bracket,
    // };
    // tracing::info!("new bracket {}", bracket.bracket.get_id());
    // tracing::debug!("new bracket {:?}", bracket);
    // (StatusCode::OK, AxumJson(bracket)).into_response()
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
#[instrument(name = "report_result", skip(report))]
pub async fn report_result(AxumJson(report): AxumJson<ReportResultInput>) -> impl IntoResponse {
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

    let lower_bracket_matches = match dev.partition_loser_bracket() {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
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
    tracing::info!("updated bracket {}", bracket.bracket.get_id());
    tracing::debug!("updated bracket {:?}", bracket);
    Ok(AxumJson(bracket))
}

#[derive(Serialize, Deserialize)]
/// 201 response
pub struct GenericResourceCreated {
    /// Resource ID
    pub id: Id,
}

/// Matches raw value
#[derive(Deserialize, Serialize)]
pub struct MatchesRaw(pub Vec<Match>);

/// Bracket in database
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct BracketRecord {
    /// bracket ID
    pub id: Id,
    /// name
    pub name: String,
    /// creation date
    pub created_at: DateTime<Utc>,
    /// matches
    pub matches: SqlxJson<MatchesRaw>,
}

/// Return a newly instanciated bracket from ordered (=seeded) player names
#[instrument(name = "create_bracket")]
pub async fn create_bracket(
    State(pool): State<PgPool>,
    AxumJson(player_list): AxumJson<PlayerList>,
) -> impl IntoResponse {
    tracing::debug!("new bracket from players: {:?}", player_list.names);

    let mut bracket = Bracket::default();
    for name in player_list.names {
        let Ok(tmp) = bracket.add_participant(name.as_str()) else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        bracket = tmp;
    }

    let r = sqlx::query!(
        "INSERT INTO brackets (name, matches) VALUES ($1, $2) RETURNING id",
        bracket.get_name(),
        SqlxJson(bracket.get_matches()) as _,
    )
    .fetch_one(&pool)
    .await
    .expect("user insert");
    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    tracing::info!("new bracket {}", r.id);

    tracing::info!("new bracket {}", bracket.get_id());
    tracing::debug!("new bracket {:?}", bracket);
    Ok((
        StatusCode::CREATED,
        AxumJson(GenericResourceCreated { id: r.id }),
    )
        .into_response())
}

/// Return a newly instanciated bracket from ordered (=seeded) player names
#[instrument(name = "get_bracket", skip(pool))]
#[debug_handler]
pub async fn get_bracket(
    Path(bracket_id): Path<Id>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    tracing::debug!("bracket {bracket_id}");

    let Some(b) = sqlx::query_as!(
        BracketRecord,
        r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at  from brackets WHERE id = $1"#,
        bracket_id,
    )
    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    .fetch_optional(&pool)
    .await
    .expect("fetch result") else {
        return (StatusCode::NOT_FOUND).into_response();
    };

    (StatusCode::OK, AxumJson(b)).into_response()
}

/// Return a newly instanciated bracket from ordered (=seeded) player names
#[instrument(name = "list_brackets", skip(pool))]
#[debug_handler]
pub async fn list_brackets(
    // NOTE pool before validated query params for some reason???
    State(pool): State<PgPool>,
    ValidatedQueryParams(pagination): ValidatedQueryParams<Pagination>,
) -> impl IntoResponse {
    tracing::debug!("bracket {pagination:?}");
    let limit: i64 = pagination.limit.try_into().expect("ok");
    let offset: i64 = pagination.offset.try_into().expect("ok");

    let brackets = sqlx::query_as!(
        GenericResource,
        r#"SELECT id, name, created_at from brackets LIMIT $1 OFFSET $2"#,
        limit,
        offset
    )
    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    .fetch_all(&pool)
    .await
    .expect("fetch result");

    let list = GenericResourcesList(brackets);

    (StatusCode::OK, AxumJson(list)).into_response()
}
