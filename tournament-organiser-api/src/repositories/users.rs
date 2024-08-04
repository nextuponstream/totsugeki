//! User repository

use crate::users::registration::User;
use sqlx::{Postgres, Transaction};
use totsugeki::player::Id;

pub(crate) struct UserRepository<'a> {
    /// transaction connection to use
    transaction: &'a mut Transaction<'a, Postgres>,
}
#[derive(Debug)]
pub(crate) enum Error {}

impl<'a> UserRepository<'a> {
    pub fn new(transaction: &'a mut Transaction<'a, Postgres>) -> UserRepository<'a> {
        Self { transaction }
    }

    pub async fn read(self, user_id: Id) -> Result<User, Error> {
        todo!()
    }
}
