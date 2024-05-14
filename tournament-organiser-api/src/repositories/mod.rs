//! wrap every call to sqlx to abstract away
//! * transactions
//! * connection pool
//! * reused functions
//! * authorize user to update their resource

pub(crate) mod brackets;
