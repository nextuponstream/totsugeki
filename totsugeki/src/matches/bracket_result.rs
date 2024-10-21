//! Bracket result

use thiserror::Error;

/// Bracket result
#[derive(Debug)]
pub struct BracketResult((u8, u8));

/// Creating result
#[derive(Error, Debug)]
pub enum Error {
    /// Match result is invalid
    #[error("Invalid match result")]
    Invalid(u8, u8),
}

impl BracketResult {
    /// New bracket result
    pub fn new(r1: u8, r2: u8) -> Result<Self, Error> {
        if r1 == 0 && r2 == 0 {
            Err(Error::Invalid(r1, r2))
        } else {
            Ok(Self((r1, r2)))
        }
    }
}
