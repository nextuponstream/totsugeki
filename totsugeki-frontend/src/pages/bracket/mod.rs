//! Bracket related pages

use totsugeki_api_request::RequestError;

pub mod many;
pub mod single;

/// States of a bracket list fetch request
#[derive(Debug)]
pub enum FetchState<T> {
    /// Page is not fetching brackets
    NotFetching,
    /// Page is fetching brackets
    Fetching,
    /// Page has successfully fetched
    Success(T),
    /// Failure to fetch
    Failed(RequestError),
}
