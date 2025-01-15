//! Store tournament matches

use crate::types::SqlxError;
use totsugeki_core::matches::Match;

#[derive(Debug)]
pub(crate) struct MatchRepository {}

impl MatchRepository {
    /// Save matches to database
    pub async fn store_many(matches: Vec<Match>) -> Result<(), SqlxError> {
        todo!()
    }
}
