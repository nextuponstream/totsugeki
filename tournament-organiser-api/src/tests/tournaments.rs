#[cfg(test)]
mod tests {
    use crate::repositories::tournaments::*;
    use sqlx::PgPool;

    #[sqlx::test(fixtures("3_players_tournament.sql"))]
    async fn deleting_tournaments_deletes_related_data(db: PgPool) {
        let mut transaction = db.begin().await.unwrap();
        let tournaments = TournamentRepository::all(&mut transaction).await.unwrap();
        assert_eq!(1, tournaments.len());
        // TODO refactor to repository
        let matches = sqlx::query("SELECT * FROM matches")
            .fetch_all(&mut *transaction)
            .await
            .unwrap();
        assert_eq!(5, matches.len());

        let tournament_id = "62aefc4c-d6ec-4c2f-98f0-b639688cbe0c".parse().unwrap();
        TournamentRepository::delete(&mut transaction, tournament_id)
            .await
            .unwrap();

        // TODO refactor to repository
        let tournaments = TournamentRepository::all(&mut transaction).await.unwrap();
        assert_eq!(0, tournaments.len());
        // TODO refactor to repository
        let matches = sqlx::query("SELECT * FROM matches")
            .fetch_all(&mut *transaction)
            .await
            .unwrap();
        assert_eq!(0, matches.len());
        transaction.commit().await.unwrap();
    }
}
