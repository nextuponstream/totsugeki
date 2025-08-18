//! Guests persistence

use crate::types::{SqlxError, SqlxTransaction};
use crate::ID;

/// Persist and retrieve guest data
pub struct GuestRepository {}

impl GuestRepository {
    /// Create many guests from `names` and returns created guest ID's
    pub async fn create_many(
        transaction: SqlxTransaction<'_, '_>,
        names: Vec<String>,
    ) -> Result<Vec<ID>, SqlxError> {
        let mut guest_ids = vec![];
        for name in names {
            let guest_id = sqlx::query!(
                r#"
INSERT INTO guests (name) VALUES ($1) RETURNING id;
           "#,
                name
            )
            .fetch_one(&mut **transaction)
            .await?
            .id;
            guest_ids.push(guest_id);
        }
        Ok(guest_ids)
    }
}
