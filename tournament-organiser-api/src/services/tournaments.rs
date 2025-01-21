//! Tournament lifecycle

use crate::repositories::brackets::Error;
use crate::repositories::matches::MatchRepository;
use crate::repositories::players::PlayerRepository;
use crate::services::traits::user_trait::UserTrait;
use crate::tournaments::MatchData;
use crate::tournaments::PlayerData;
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
    /// Create bracket and set creator `user_id` as tournament organiser
    pub async fn create<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament: &Tournament,
        double_elimination_bracket: &DoubleEliminationBracket,
        user_id: ID,
    ) -> Result<(), SqlxError> {
        let _ = sqlx::query!(
            "INSERT INTO tournaments (id, name, format) VALUES ($1, $2, $3)",
            tournament.get_id().get(), // FIXME this is not good syntax
            tournament.get_name(),
            "double_elimination".to_string(),
        )
        .execute(&mut **transaction)
        .await?;
        let _ = sqlx::query!(
            "INSERT INTO tournament_organisers (tournament_id, user_id) VALUES ($1, $2)",
            tournament.get_id().get(),
            user_id,
        )
        .execute(&mut **transaction)
        .await?;

        MatchRepository::store_many(transaction, double_elimination_bracket.get_matches()).await?;

        Ok(())
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

        let Some(tournament_record) = sqlx::query_as!(
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
        ))                                                             as "matches!: Vec<MatchData>",
    COALESCE(NULLIF(ARRAY_AGG((P.player_id, U.name)), '{NULL}'), '{}') as "players!: Vec<PlayerData>"
FROM tournaments

JOIN (SELECT tournament_id,
             match_id,
             tournament_matches.pos
      from tournament_matches
      ORDER BY pos) as ordered_tournament_matches
            ON tournaments.id = ordered_tournament_matches.tournament_id

         JOIN players P ON ordered_tournament_matches.tournament_id = P.tournament_id
         JOIN users U ON player_id = U.id
         JOIN matches M ON M.id = ordered_tournament_matches.match_id
WHERE tournaments.id = $1
GROUP BY tournaments.id
            "#,
            tournament_id.get(),
        )
        // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        .fetch_optional(&mut **transaction)
        .await?
        else {
            return Ok(None);
        };

        let is_tournament_organiser =
            Self::is_tournament_organiser(transaction, user.id, tournament_id).await?;

        let (mut tournament, _): (Tournament, _) = tournament_record.parse();

        if let Err(e) =
            tournament.add_participant(Player::from((PlayerID::new(user.id.0), user.name)))
        {
            return match e {
                ParticipantError::AlreadyPresent => Err(Error::PlayerAlreadyPresent),
            };
        };

        let bracket = DoubleEliminationBracket::create(
            Seeding::new(
                tournament
                    .get_participants()
                    .0
                    .iter()
                    .map(Player::get_id)
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
            r#"SELECT id, name, created_at, format, count(*) OVER() AS total
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
    ) -> Result<Option<(Tournament, DoubleEliminationBracket, bool)>, Error> {
        let Some(tournament_record) = sqlx::query_as!(
            TournamentRecord,
            r#"SELECT id, name, format, created_at
            FROM tournaments
            WHERE id = $1
            "#,
            tournament_id,
        )
        // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        .fetch_optional(&mut **transaction)
        .await?
        else {
            return Ok(None);
        };
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

        let matches = MatchRepository::get_for_tournament(transaction, tournament_id).await?;
        let players = vec![];

        let (tournament, bracket) = tournament_record.parse(matches, players);

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
}
