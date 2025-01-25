//! wrap every call to sqlx to abstract away
//! * transactions
//! * connection pool
//! * reused functions
//! * authorize user to update their resource
//!
//! Fundamentally, repositories do not represent business logic: look into
//! services

pub(crate) mod brackets;
pub(crate) mod guests;
pub(crate) mod matches;
pub(crate) mod players;
pub(crate) mod tournaments;
pub(crate) mod users;
