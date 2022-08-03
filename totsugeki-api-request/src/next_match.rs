//! Get next match to play

use crate::{RequestError, HTTP_PREFIX};
use totsugeki::{matches::NextMatchGET as GET, DiscussionChannel};

/// Get next match to play
///
/// # Errors
/// Returns error if there is an error with the network
pub async fn next_match<T: DiscussionChannel>(
    client: reqwest::Client,
    tournament_server_url: &str,
    player_internal_id: &str,
    discussion_channel: T,
) -> Result<GET, RequestError> {
    let res = client
        .get(format!(
            "{HTTP_PREFIX}{tournament_server_url}/next_match/{player_internal_id}/{}",
            discussion_channel.get_internal_id().to_string()
        ))
        .send()
        .await?;
    let next_match: GET = res.json().await?;
    Ok(next_match)
}
