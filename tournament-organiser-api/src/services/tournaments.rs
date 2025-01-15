// pub struct tou

use crate::repositories::brackets::Error;
use crate::repositories::matches::MatchRepository;
use crate::repositories::players::PlayerRepository;
use crate::resources::PaginatedGenericResource;
use crate::tournaments::{ParticipantError, Tournament, TournamentID, TournamentRecord, ID};
use crate::types::{SqlxError, SqlxTransaction};
use crate::users::registration::UserRecord;
use totsugeki_core::bracket::seeding::Seeding;
use totsugeki_core::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki_core::player::{Player, PlayerID};

pub struct TournamentService {}

impl TournamentService {
    /// Create bracket and set creator `user_id` as tournament organiser
    pub async fn create<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament: &Tournament,
        double_elimination_bracket: &DoubleEliminationBracket,
        user_id: ID,
    ) -> Result<(), SqlxError> {
        let _ = sqlx::query!(
            "INSERT INTO tournaments (id, name) VALUES ($1, $2)",
            tournament.get_id().get(), // FIXME this is not good syntax
            tournament.get_name(),
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

        MatchRepository::store_many(double_elimination_bracket.get_matches()).await?;

        Ok(())
    }

    /// User joins bracket
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

        todo!()
        // let Some(tournament_record) = sqlx::query_as!(
        //     TournamentRecord,
        //     r#"SELECT id, name, created_at"#,
        //     tournament_id,
        // )
        // // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        // .fetch_optional(&mut **transaction)
        // .await?
        // else {
        //     return Ok(None);
        // };
        // let is_tournament_organiser = sqlx::query!(
        //     r#"SELECT tournament_id, user_id from tournament_organisers WHERE user_id = $1 AND tournament_id = $2"#,
        //        user.id,
        //        tournament_id
        // ).fetch_optional(&mut **transaction).await?.is_some();
        //
        // let (mut tournament, _) = tournament_record.parse();
        //
        // if let Err(e) =
        //     tournament.add_participant(Player::from((PlayerID::new(user.id), user.name)))
        // {
        //     return match e {
        //         ParticipantError::AlreadyPresent => Err(Error::PlayerAlreadyPresent),
        //     };
        // };
        //
        // let bracket = DoubleEliminationBracket::create(
        //     Seeding::new(
        //         tournament
        //             .get_participants()
        //             .0
        //             .iter()
        //             .map(Player::get_id)
        //             .collect(),
        //     )
        //     .expect("should update seeding of bracket with tournament valid seeding"),
        //     AutomaticMatchValidationMode::Flexible,
        //     MatchFormat::ft2(),
        //     None,
        // );
        //
        // Ok(Some((tournament, bracket, is_tournament_organiser)))
    }

    /// List all brackets belonging to `user_id`
    pub async fn list<'a>(
        transaction: SqlxTransaction<'a, '_>,
        sort_order: String,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PaginatedGenericResource>, SqlxError> {
        // let brackets = sqlx::query_as!(
        //     PaginatedGenericResource,
        //     r#"SELECT id, name, created_at, count(*) OVER() AS total from tournaments
        //  ORDER BY
        //    CASE WHEN $1 = 'ASC' THEN created_at END ASC,
        //    CASE WHEN $1 = 'DESC' THEN created_at END DESC
        //  LIMIT $2
        //  OFFSET $3
        //  "#,
        //     sort_order,
        //     limit,
        //     offset
        // )
        // // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        // .fetch_all(&mut **transaction)
        // .await?;
        // Ok(brackets)
        todo!()
    }
}
