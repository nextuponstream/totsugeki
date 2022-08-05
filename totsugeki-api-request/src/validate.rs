//! Validate match

use crate::{RequestError, HTTP_PREFIX};
use totsugeki::matches::Id as MatchId;

/// Send message to api to validate match with given `match_id`
///
/// # Errors
/// Returns an error if bracket could not be updated
pub async fn send(
    client: reqwest::Client,
    tournament_server_url: &str,
    authorization_header: &str,
    match_id: MatchId,
) -> Result<(), RequestError> {
    let _res = client
        .post(format!(
            "{HTTP_PREFIX}{tournament_server_url}/bracket/validate/{match_id}"
        ))
        .header("X-API-Key", authorization_header)
        .send()
        .await?;
    Ok(())
}
