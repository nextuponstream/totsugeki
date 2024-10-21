//! Generic resources

use chrono::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use totsugeki::bracket::Id;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
/// Pagination
pub struct Pagination {
    /// How much per page
    #[validate(custom = "validate_limit")]
    pub limit: usize,
    /// Display from bracket X
    pub offset: usize,
    /// Sort by criteria: ASC or DESC
    #[validate(custom = "sort_order")]
    pub sort_order: String,
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

/// Sort ascending or descending
fn sort_order(sort_order: &str) -> Result<(), ValidationError> {
    match sort_order {
        "ASC" | "DESC" => Ok(()),
        _ => Err(ValidationError::new("either ASC or DESC expected")),
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

/// Generic resource description
#[derive(Deserialize, Serialize)]
pub struct PaginatedGenericResource {
    /// ID of resource
    pub id: Id,
    /// name of resource
    pub name: String,
    /// creation date
    pub created_at: DateTime<Utc>,
    /// pagination helper
    pub total: Option<i64>,
}

/// List of generic resources
#[derive(Deserialize, Serialize)]
pub struct GenericResourcesList(pub Vec<GenericResource>);
/// List of generic resources
#[derive(Deserialize, Serialize)]
pub struct PaginationResult {
    /// total data
    pub total: usize,
    /// resources
    pub data: Vec<PaginatedGenericResource>,
}
