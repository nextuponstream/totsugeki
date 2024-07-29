//! Validator middleware

use async_trait::async_trait;
use axum::extract::rejection::JsonDataError;
use axum::{
    extract::{
        rejection::FormRejection, rejection::JsonRejection, Form, FromRequest, Json, Request,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;
use validator::Validate;

// axum struct validation https://github.com/tokio-rs/axum/blob/d703e6f97a0156177466b6741be0beac0c83d8c7/examples/validator/src/main.rs

#[derive(Debug, Clone, Copy, Default, Deserialize)]
/// Validate structs with defined validation rules of the generic type T or
/// return 400 response when invalid
///
/// NOTE: works for query params. Just derive `Validate` on your struct!
pub struct ValidatedRequest<T>(pub T);

/// Validated json
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct ValidatedJson<T>(pub T);

#[derive(Debug, Error)]
/// Server cannot process request
pub enum ServerError {
    #[error(transparent)]
    /// Validation error, user must provide different values
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    /// Error not related to validation error
    AxumFormRejection(#[from] FormRejection),

    #[error(transparent)]
    /// Error not related to validation error
    AxumJsonRejection(#[from] JsonRejection),
}

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedRequest<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedRequest(value))
    }
}

// https://docs.rs/axum/0.7.5/axum/extract/index.html#implementing-fromrequest
#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;

        Ok(ValidatedJson(value))
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(_) => {
                let message = format!("Input validation error: [{self}]").replace('\n', ", ");
                (StatusCode::BAD_REQUEST, message)
            }
            ServerError::AxumFormRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::AxumJsonRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        }
        .into_response()
    }
}

impl From<JsonDataError> for ServerError {
    fn from(value: JsonDataError) -> Self {
        ServerError::AxumJsonRejection(JsonRejection::JsonDataError(value))
    }
}
