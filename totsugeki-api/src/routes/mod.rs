//! routes for tournament server

pub mod bracket;
pub mod test_utils;

use crate::persistence::Database;
use poem::web::Data;
use std::boxed::Box;
use std::sync::{Arc, RwLock};

/// Instance of shared database
pub type SharedDb<'a> = Data<&'a Arc<RwLock<Box<dyn Database + Send + Sync>>>>;
