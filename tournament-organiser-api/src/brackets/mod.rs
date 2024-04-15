//! bracket management

use crate::resources::{
    PaginatedGenericResource, Pagination, PaginationResult, ValidatedQueryParams,
};
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
use totsugeki::player::{Id as PlayerId, Participants, Player};
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
pub struct CreateBracketForm {
    /// bracket names
    pub bracket_name: String,
    /// player names
    pub player_names: Vec<String>,
}

/// Result reported by player
///
/// FIXME there probably is a less computive intensive way to save steps of
/// match, like only saving relevant match ID to update. But it's not there.
/// Then this will do.
#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerMatchResultReport {
    /// high seed player
    pub p1_id: PlayerId,
    /// low seed player
    pub p2_id: PlayerId,
    /// score of player 1
    pub score_p1: i8,
    /// score of player 2
    pub score_p2: i8,
}

/// List of players from which a bracket can be created
#[derive(Deserialize, Serialize, Debug)]
pub struct BracketState {
    /// bracket names
    pub bracket_name: String,
    /// player names
    pub players: Vec<Player>,
    ///  results in order of replay
    pub results: Vec<PlayerMatchResultReport>,
}

/// Return a newly instanciated bracket from ordered (=seeded) player names for
/// display purposes
///
/// # Panics
/// When bracket cannot be converted to double elimination bracket
///
/// # Errors
/// May return 500 error when bracket cannot be parsed
#[instrument(name = "new_bracket")]
pub async fn new_bracket(AxumJson(form): AxumJson<CreateBracketForm>) -> impl IntoResponse {
    tracing::debug!("new bracket");

    let mut bracket = Bracket::default();
    bracket = bracket.update_name(form.bracket_name);
    for name in form.player_names {
        let Ok(tmp) = bracket.add_participant(name.as_str()) else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        bracket = tmp.0;
    }

    Ok(breakdown(bracket).into_response())
}

/// Save bracket replayed from player reports so in the event a guest actually
/// wants to save the resulting bracket, they can.
///
/// The server will not accept a JSON of a bracket just because it can be
/// parsed as that may lead to a malformed bracket. Then we do something a
/// little more intense computation wise that always yields a correct bracket.
#[instrument(name = "save_bracket")]
#[debug_handler]
pub async fn save_bracket(
    State(pool): State<PgPool>,
    AxumJson(bracket_state): AxumJson<BracketState>,
) -> impl IntoResponse {
    // NOTE: always pool before arguments. Otherwise:
    // error[E0277]: the trait bound `fn(axum::Json<BracketState>,
    // State<Pool<Postgres>>) -> impl std::future::Future<Output = impl
    // IntoResponse> {save_bracket}: Handler<_, _>` is not satisfied
    tracing::debug!("new bracket replayed from steps");

    let mut bracket = Bracket::default();
    bracket = bracket.update_name(bracket_state.bracket_name);
    let mut safe_player_mapping = vec![];
    // Do not rely on given ID, assign new IDs to players and map
    for player in bracket_state.players {
        let Ok(tmp) = bracket.add_participant(player.get_name().as_str()) else {
            tracing::warn!("oh no");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        bracket = tmp.0;
        safe_player_mapping.push((player, tmp.1));
    }
    let mut bracket = match bracket.start() {
        Ok(b) => b.0,
        Err(err) => {
            tracing::warn!("{err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    // let mut bracket = bracket.0;
    for r in bracket_state.results {
        let report = (r.score_p1, r.score_p2);
        let Some(p1_mapping) = safe_player_mapping.iter().find(|m| m.0.get_id() == r.p1_id) else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        let Some(p2_mapping) = safe_player_mapping.iter().find(|m| m.0.get_id() == r.p2_id) else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        bracket = match bracket.tournament_organiser_reports_result(
            p1_mapping.1.get_id(),
            report,
            p2_mapping.1.get_id(),
        ) {
            Ok(b) => b.0,
            Err(err) => {
                tracing::warn!("{err}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
    }

    let r = sqlx::query!(
        "INSERT INTO brackets (name, matches, participants) VALUES ($1, $2, $3) RETURNING id",
        bracket.get_name(),
        SqlxJson(bracket.get_matches()) as _,
        SqlxJson(bracket.get_participants()) as _,
    )
    .fetch_one(&pool)
    .await
    .expect("new bracket replayed from steps");
    // use auto-generated db id rather than from Bracket::default()
    bracket = bracket.set_id(r.id);

    tracing::info!("new bracket replayed from steps {}", bracket.get_id());
    tracing::debug!("new bracket replayed from steps {:?}", bracket);

    Ok((StatusCode::CREATED, breakdown(bracket)).into_response())
}

/// Returns existing bracket for display purposes
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
        r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at, participants as "participants: SqlxJson<Participants>"  from brackets WHERE id = $1"#,
        bracket_id,
    )
    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    .fetch_optional(&pool)
    .await
    .expect("fetch result") else {
        return (StatusCode::NOT_FOUND).into_response();
    };
    let bracket = Bracket::assemble(b.id, b.name, b.participants.0, b.matches.0 .0);

    breakdown(bracket).into_response()
}

/// Breaks down bracket in small parts to be presented by UI
fn breakdown(bracket: Bracket) -> impl IntoResponse {
    let dev: DoubleEliminationVariant = bracket.clone().try_into().expect("partition");

    // TODO test if tracing shows from which methods it was called
    let winner_bracket_matches = match dev.partition_winner_bracket() {
        Ok(wb) => wb,
        Err(e) => {
            tracing::error!("{e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let mut winner_bracket_rounds = vec![];
    for r in winner_bracket_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &bracket.get_participants()))
            .collect();
        winner_bracket_rounds.push(round);
    }

    reorder_winner_bracket(&mut winner_bracket_rounds);
    let Some(winner_bracket_lines) = winner_bracket_lines(&winner_bracket_rounds) else {
        tracing::error!("winner bracket connecting lines");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let Ok(lower_bracket_matches) = dev.partition_loser_bracket() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let mut loser_bracket_rounds: Vec<Vec<MinimalMatch>> = vec![];
    for r in lower_bracket_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &bracket.get_participants()))
            .collect();
        loser_bracket_rounds.push(round);
    }
    reorder_loser_bracket(&mut loser_bracket_rounds);
    let Some(loser_bracket_lines) = loser_bracket_lines(loser_bracket_rounds.clone()) else {
        tracing::error!("loser bracket connecting lines");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let Ok((gf, gf_reset)) = dev.grand_finals_and_reset() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let gf = from_participants(&gf, &bracket.get_participants());
    let gf_reset = from_participants(&gf_reset, &bracket.get_participants());

    let bracket = BracketDisplay {
        winner_bracket: winner_bracket_rounds,
        winner_bracket_lines,
        loser_bracket: loser_bracket_rounds,
        loser_bracket_lines,
        grand_finals: gf,
        grand_finals_reset: gf_reset,
        bracket,
    };
    tracing::info!("displaying bracket {}", bracket.bracket.get_id());
    tracing::debug!("displaying bracket {:?}", bracket);
    (StatusCode::OK, AxumJson(bracket)).into_response()
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
            //  of sync with database state and returns something to user
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let participants = bracket.get_participants();
    let dev: DoubleEliminationVariant = bracket.clone().try_into().expect("partition");

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
    /// participants
    pub participants: SqlxJson<Participants>,
}

/// Return a newly instanciated bracket from ordered (=seeded) player names
#[instrument(name = "create_bracket")]
pub async fn create_bracket(
    State(pool): State<PgPool>,
    AxumJson(player_list): AxumJson<CreateBracketForm>,
) -> impl IntoResponse {
    tracing::debug!("new bracket from players: {:?}", player_list.player_names);

    let mut bracket = Bracket::default();
    for name in player_list.player_names {
        let Ok(tmp) = bracket.add_participant(name.as_str()) else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        bracket = tmp.0;
    }

    let r = sqlx::query!(
        "INSERT INTO brackets (name, matches, participants) VALUES ($1, $2, $3) RETURNING id",
        bracket.get_name(),
        SqlxJson(bracket.get_matches()) as _,
        SqlxJson(bracket.get_participants()) as _,
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
        r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at, participants as "participants: SqlxJson<Participants>"  from brackets WHERE id = $1"#,
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
    let limit: i64 = pagination.limit.try_into().expect("ok");
    let offset: i64 = pagination.offset.try_into().expect("ok");

    let brackets = sqlx::query_as!(
        PaginatedGenericResource,
        r#"SELECT id, name, created_at, count(*) OVER() AS total from brackets
         LIMIT $1
         OFFSET $2"#,
        limit,
        offset
    )
    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    .fetch_all(&pool)
    .await
    .expect("fetch result");

    let data = brackets;
    let pagination_result = PaginationResult { total: 100, data };

    (StatusCode::OK, AxumJson(pagination_result)).into_response()
}

/// `/:user_id/brackets` GET to view brackets managed by user
#[instrument(name = "user_brackets", skip(pool))]
pub(crate) async fn user_brackets(
    Path(user_id): Path<Id>,
    State(pool): State<PgPool>,
    ValidatedQueryParams(pagination): ValidatedQueryParams<Pagination>,
) -> impl IntoResponse {
    let limit: i64 = pagination.limit.try_into().expect("ok");
    let offset: i64 = pagination.offset.try_into().expect("ok");

    // FIXME join with user table
    // paginated results with total count: https://stackoverflow.com/a/28888696
    // not optimal : each rows contains the total
    // not optimal : you have to extract total from first row if you want the
    // count to be separated from rows
    // weird: need Option<i64> for total otherwise does not compile
    // why keep : it might be nice for the consumer to access total rows in the
    // returned row. Also it works for the current use case (return all rows)
    // TODO: if this app scales hard, then this naive pagination won't hold I
    //  think (searching late page may become slow as offset forces all rows to
    //  be counted). But it's good enough for now
    // NOTE: ASC/DESC as param https://github.com/launchbadge/sqlx/issues/3020#issuecomment-1919930408
    let brackets = sqlx::query_as!(
        PaginatedGenericResource,
        r#"SELECT id, name, created_at, count(*) OVER() AS total from brackets
         ORDER BY 
           CASE WHEN $1 = 'ASC' THEN created_at END ASC,
           CASE WHEN $1 = 'DESC' THEN created_at END DESC
         LIMIT $2
         OFFSET $3
         "#,
        pagination.sort_order,
        limit,
        offset,
    )
    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    .fetch_all(&pool)
    .await
    .expect("fetch result");

    let total = if brackets.is_empty() {
        0
    } else {
        brackets[0].total.expect("total")
    };
    let total = total.try_into().expect("conversion");
    let data = brackets;
    let pagination_result = PaginationResult { total, data };

    (StatusCode::OK, AxumJson(pagination_result)).into_response()
}
