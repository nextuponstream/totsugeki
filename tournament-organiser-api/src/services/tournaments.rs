//! Tournament lifecycle

use crate::guests::{Guest, GuestID};
use crate::repositories::brackets::Error;
use crate::repositories::guests::GuestRepository;
use crate::repositories::matches::MatchRepository;
use crate::repositories::players::PlayerRepository;
use crate::repositories::tournaments::TournamentRepository;
use crate::repositories::users::UserRepository;
use crate::services::traits::match_trait::MatchTrait;
use crate::services::traits::user_trait::UserTrait;
use crate::tournaments::tournament_players::TournamentPlayer;
use crate::tournaments::PlayerData;
use crate::tournaments::{MatchData, ReportResultInput};
use crate::tournaments::{
    ParticipantError, Tournament, TournamentAugmentedRecord, TournamentID, TournamentRecord, ID,
};
use crate::types::{SqlxError, SqlxTransaction};
use crate::users::registration::{UserID, UserRecord};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use totsugeki_core::bracket::seeding::Seeding;
use totsugeki_core::bracket::Id;
use totsugeki_core::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki_core::matches::result::MatchFormat;
use totsugeki_core::player::{Player, PlayerID};
use totsugeki_core::validation::AutomaticMatchValidationMode;

/// Create and manage tournaments
pub struct TournamentService {}
impl UserTrait for TournamentService {}
impl MatchTrait for TournamentService {}

/// Generic resource description
#[derive(Deserialize, Serialize)]
pub struct PaginatedTournamentResource {
    /// ID of resource
    pub id: Id,
    /// name of resource
    pub name: String,
    /// tournament format
    pub format: String,
    /// creation date
    pub created_at: OffsetDateTime,
    /// pagination helper
    pub total: Option<i64>,
}

impl TournamentService {
    /// Create tournament and set creator `user_id` as tournament organiser
    pub async fn create_tournament_organiser_and_matches<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament: &Tournament,
        double_elimination_bracket: &DoubleEliminationBracket,
        user_id: ID,
    ) -> Result<(), SqlxError> {
        // TODO instantiate players and guests from service... so create tournament create
        //  everything associated. Readability
        let _ = sqlx::query!(
            "INSERT INTO tournament_organisers (tournament_id, user_id) VALUES ($1, $2)",
            tournament.get_id().get(),
            user_id,
        )
        .execute(&mut **transaction)
        .await?;

        // FIXME high seed player fk... probably a mapping error
        MatchRepository::store_many(transaction, double_elimination_bracket.get_matches()).await?;

        Ok(())
    }

    /// Get tournament with matches and players
    pub async fn get_tournament<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament_id: TournamentID,
    ) -> Result<Option<TournamentAugmentedRecord>, SqlxError> {
        Ok(sqlx::query_as!(
            TournamentAugmentedRecord,
            r#"
SELECT
    tournaments.id,
    tournaments.name,
    tournaments.format,
    tournaments.created_at,
    ARRAY_AGG(DISTINCT (
                        ordered_tournament_matches.pos,
                        ordered_tournament_matches.match_id,
                        M.format,
                        M.format_n,
                        M.high_seed,
                        M.high_seed_player,
                        M.low_seed,
                        M.low_seed_player
        )) 
    filter ( where ordered_tournament_matches.match_id IS NOT NULL ) as "matches: Vec<MatchData>",
    ARRAY_AGG(DISTINCT (P.id, COALESCE(U.name, G.name), U.id, G.id)) 
    filter ( where P.id IS NOT NULL ) as "players: Vec<PlayerData>"
FROM tournaments
         LEFT JOIN (SELECT tournament_id,
                      match_id,
                      tournament_matches.pos
               from tournament_matches
               ORDER BY pos) as ordered_tournament_matches
              ON tournaments.id = ordered_tournament_matches.tournament_id

         LEFT JOIN players P ON ordered_tournament_matches.tournament_id = P.tournament_id
         LEFT JOIN users U ON P.user_id = U.id
         LEFT JOIN guests G ON P.guest_id = G.id
         LEFT JOIN matches M ON M.id = ordered_tournament_matches.match_id
WHERE tournaments.id = $1
GROUP BY tournaments.id
            "#,
            tournament_id.get(),
        )
        // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        .fetch_optional(&mut **transaction)
        .await?)
    }

    /// User joins tournament
    pub async fn join<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament_id: TournamentID,
        user: UserRecord,
    ) -> Result<Option<(Tournament, DoubleEliminationBracket, bool)>, Error> {
        let user_is_player_of_tournament =
            PlayerRepository::is_user_a_player_in_tournament(transaction, user.id, tournament_id)
                .await?;

        if user_is_player_of_tournament {
            todo!("send proper error to user");
        }

        // test https://stackoverflow.com/a/76476596/29197309
        let Some(tournament_record) =
            TournamentService::get_tournament(transaction, tournament_id).await?
        else {
            return Ok(None);
        };

        let is_tournament_organiser =
            Self::is_tournament_organiser(transaction, user.id, tournament_id).await?;

        let (mut tournament, _): (Tournament, _) = tournament_record.parse();

        let tournament_player = TournamentPlayer::new(Some(user.id), None, user.name)
            .expect("tournament player from user");
        Self::create_player(transaction, tournament_id, tournament_player.clone()).await?;
        if let Err(e) = tournament.add_player(tournament_player) {
            return match e {
                ParticipantError::AlreadyPresent => Err(Error::PlayerAlreadyPresent),
            };
        };

        let bracket = DoubleEliminationBracket::create(
            Seeding::new(
                tournament
                    .get_players()
                    .iter()
                    .map(|tp| PlayerID::new(tp.get_id()))
                    .collect(),
            )
            .expect("should update seeding of bracket with tournament valid seeding"),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(), // FIXME
            None,
        );

        Ok(Some((tournament, bracket, is_tournament_organiser)))
    }

    /// List all tournaments belonging to `user_id`
    pub async fn list<'a>(
        transaction: SqlxTransaction<'a, '_>,
        sort_order: String,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PaginatedTournamentResource>, SqlxError> {
        let tournaments = sqlx::query_as!(
            PaginatedTournamentResource,
            r#"
SELECT id, name, created_at, format, count(*) OVER() AS total
FROM tournaments
ORDER BY
    CASE WHEN $1 = 'ASC' THEN created_at END ASC,
    CASE WHEN $1 = 'DESC' THEN created_at END DESC
LIMIT $2
OFFSET $3
            "#,
            sort_order,
            limit,
            offset
        )
        // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        .fetch_all(&mut **transaction)
        .await?;
        Ok(tournaments)
    }

    /// Returns tournament in database and boolean if user is a tournament
    /// organiser of that tournament
    pub async fn read_for_user<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament_id: ID,
        user_id: Option<ID>,
    ) -> Result<Option<(Tournament, DoubleEliminationBracket, bool)>, SqlxError> {
        let Some(tournament_record) =
            TournamentService::get_tournament(transaction, TournamentID::from(tournament_id))
                .await?
        else {
            return Ok(None);
        };
        //             sqlx::query_as!(
        //             TournamentRecord,
        //             r#"
        // SELECT
        //     id,
        //     name,
        //     format,
        //     created_at
        // FROM tournaments
        // WHERE id = $1
        //             "#,
        //             tournament_id,
        //         )
        //         // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        //         .fetch_optional(&mut **transaction)
        //         .await?
        //         else {
        //             return Ok(None);
        //         };
        let is_tournament_organiser = if let Some(user_id) = user_id {
            TournamentService::is_tournament_organiser(
                transaction,
                user_id.into(),
                tournament_id.into(),
            )
            .await?
        } else {
            false
        };

        let (tournament, bracket) = tournament_record.parse();

        Ok(Some((tournament, bracket, is_tournament_organiser)))
    }

    /// List all brackets belonging to `user_id`
    pub async fn user_tournaments<'a>(
        transaction: SqlxTransaction<'a, '_>,
        sort_order: String,
        limit: i64,
        offset: i64,
        user_id: ID,
    ) -> Result<Vec<PaginatedTournamentResource>, SqlxError> {
        // paginated results with total count: https://stackoverflow.com/a/28888696
        // not optimal : each rows contains the total
        // not optimal : you have to extract total from first row if you want the
        // count to be separated from rows
        // weird: need Option<i64> for total otherwise does not compile
        // why keep : it might be nice for the consumer to access total rows in
        // the returned row. Also, it works for the current use case (return all
        // rows)
        // NOTE: ASC/DESC as param https://github.com/launchbadge/sqlx/issues/3020#issuecomment-1919930408
        let tournaments = sqlx::query_as!(
            PaginatedTournamentResource,
            r#"SELECT 
            id, 
            name,
            format,
            tournaments.created_at,
            count(*) OVER() AS total from tournaments
            LEFT JOIN tournament_organisers ON tournaments.id = tournament_organisers.tournament_id
            WHERE tournament_organisers.user_id = $4
            ORDER BY
                CASE WHEN $1 = 'ASC' THEN tournaments.created_at END ASC,
                CASE WHEN $1 = 'DESC' THEN tournaments.created_at END DESC
            LIMIT $2
            OFFSET $3
            "#,
            sort_order,
            limit,
            offset,
            user_id,
        )
        // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        .fetch_all(&mut **transaction)
        .await?;

        Ok(tournaments)
    }

    /// Update bracket with result
    pub async fn update_with_result<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament_id: ID,
        report: &ReportResultInput,
    ) -> Result<
        Option<(Tournament, DoubleEliminationBracket)>,
        crate::tournaments::update_with_result::Error,
    > {
        // let Some(tournament_record) = sqlx::query_as!(
        // TournamentRecord,
        // r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at, participants as "participants: SqlxJson<Participants>" from tournaments WHERE id = $1"#,
        // tournament_id,
        // )
        //     // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        //     .fetch_optional(&mut **transaction)
        //     .await?
        //     else {
        //         return Ok(None);
        //     };
        // let (tournament, bracket) = tournament_record.parse();
        //
        // // FIXME actual error handling
        // let (bracket, _, _) = bracket.tournament_organiser_reports_result_dangerous(
        //     report.p1_id,
        //     Score(report.score_p1, report.score_p2),
        //     report.p2_id,
        // )?;
        // let _r = sqlx::query!(
        //     r#"
        // UPDATE tournaments
        //     SET matches = $1
        // WHERE id = $2
        // "#,
        //     SqlxJson(bracket.get_matches()) as _,
        //     tournament.get_id().0,
        // )
        // .execute(&mut **transaction)
        // .await?;
        // Ok(Some((tournament, bracket)))
        todo!()
    }

    async fn create_player<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament_id: TournamentID,
        tournament_player: TournamentPlayer,
    ) -> Result<(), SqlxError> {
        assert!(tournament_player.get_user().is_some() ^ tournament_player.get_guest().is_some());
        match (tournament_player.get_user(), tournament_player.get_guest()) {
            (Some(user_id), None) => {
                sqlx::query!(
                    r#"
INSERT INTO players (id, tournament_id, user_id) VALUES ($1, $2, $3);             
                "#,
                    tournament_player.get_id(),
                    tournament_id.get(),
                    user_id.0
                )
                .execute(&mut **transaction)
                .await?;
                Ok(())
            }
            (None, Some(guest)) => todo!(),
            _ => unreachable!(),
        }
    }

    /// Add guests from names
    pub async fn add_guests_as_tournament_players<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament_id: TournamentID,
        guest_names: Vec<String>,
    ) -> Result<Vec<TournamentPlayer>, SqlxError> {
        let guest_ids = GuestRepository::add_many(transaction, guest_names.clone()).await?;
        let guests = guest_ids
            .iter()
            .zip(guest_names)
            .map(|g| Guest(*g.0, g.1))
            .collect();
        let tournament_players =
            PlayerRepository::add_guests(transaction, tournament_id, guests).await?;
        Ok(tournament_players)
    }
}
