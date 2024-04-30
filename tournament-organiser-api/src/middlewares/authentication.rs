//! User authentication middleware

use crate::users::session::Keys;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use totsugeki::player::Id;
use tower_sessions::Session;

/// Auth layer checking for presence of key `user_id` in session, set by login
/// endpoint.
///
/// NOTE: if you need multi auth layer, reach out for `axum_login` crate. Until
///       then, relying on a key being present in session is enough.
pub(crate) async fn auth_layer(
    // State(state): State<AppState>,
    session: Session,
    // you can add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request,
    next: Next,
) -> Response {
    let v: Option<Id> = session
        .get(&Keys::UserId.to_string())
        .await
        .expect("value from store");
    if v.is_none() {
        tracing::warn!(
            "unauthenticated request against protected route /api{} {}",
            request.uri(),
            request.method()
        );
        return (StatusCode::UNAUTHORIZED).into_response();
    };
    // do something with `request`...

    // do something with `response`...

    next.run(request).await
}
