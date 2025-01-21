//!

use crate::services::traits::user_trait::UserTrait;
use crate::tournaments::TournamentID;
use crate::types::{SqlxError, SqlxTransaction};
use crate::users::registration::UserID;

/// Complex database queries for users
pub struct UserService;

impl UserTrait for UserService {}
