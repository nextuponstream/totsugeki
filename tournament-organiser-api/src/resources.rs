//! Generic structs for routine api work like pagination

use async_trait::async_trait;
use axum::{
    extract::{rejection::FormRejection, Form, FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::prelude::*;
use serde::Serialize;
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;
use totsugeki::bracket::Id;
use validator::{Validate, ValidationError};

// axum struct validation https://github.com/tokio-rs/axum/blob/d703e6f97a0156177466b6741be0beac0c83d8c7/examples/validator/src/main.rs

#[derive(Debug, Clone, Copy, Default, Deserialize)]
/// Validate structs with defined validation rules of the generic type T or
/// return 400 response when unvalid
pub struct ValidatedQueryParams<T>(pub T);

#[derive(Debug, Error)]
/// Server cannot process request
pub enum ServerError {
    #[error(transparent)]
    /// Validation error, user must provide different values
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    /// Error not related to validation error
    AxumFormRejection(#[from] FormRejection),
}

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedQueryParams<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedQueryParams(value))
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
        }
        .into_response()
    }
}

#[derive(Debug, Deserialize, Validate)]
/// Pagination
pub struct Pagination {
    /// How much per page
    #[validate(custom = "validate_limit")]
    pub limit: usize,
    /// Display from bracket X
    pub offset: usize,
}

/// Limit for pagination is either 10, 25, 50 or 100
fn validate_limit(l: usize) -> Result<(), ValidationError> {
    match l {
        10 | 25 | 50 | 100 => Ok(()),
        _ => Err(ValidationError::new(
            "Must be equal to either 10, 25, 50 or 100",
        )),
    }
}

/// Generic resource description
#[derive(Deserialize, Serialize)]
pub struct GenericResource {
    /// ID of resource
    pub id: Id,
    /// name of resource
    pub name: String,
    /// creation date
    pub created_at: DateTime<Utc>,
}

/// List of generic resources
#[derive(Deserialize, Serialize)]
pub struct GenericResourcesList(pub Vec<GenericResource>);
