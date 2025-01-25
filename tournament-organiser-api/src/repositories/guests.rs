//! Guests persistence

use crate::guests::GuestID;
use crate::types::{SqlxError, SqlxTransaction};

pub struct GuestRepository {}

impl GuestRepository {
    pub async fn add_many<'a>(
        transaction: SqlxTransaction<'a, '_>,
        guest_names: Vec<String>,
    ) -> Result<Vec<GuestID>, SqlxError> {
        let mut guest_ids = vec![];
        for guest_name in guest_names {
            let guest_id = sqlx::query!(
                r#"
INSERT INTO guests (name) VALUES ($1) RETURNING id;
           "#,
                guest_name
            )
            .fetch_one(&mut **transaction)
            .await?
            .id;
            guest_ids.push(GuestID::new(guest_id));
        }
        Ok(guest_ids)
    }
}
