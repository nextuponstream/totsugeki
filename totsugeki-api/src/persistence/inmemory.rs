//! In-memory database
use crate::persistence::{DBAccessor, Error};
use std::sync::{Arc, RwLock};
use totsugeki::{Bracket, Organiser};

/// In-memory database
#[derive(Default)]
pub struct InMemoryDBAccessor {
    db: Arc<RwLock<InMemoryDatabase>>,
}

/// In memory database
#[derive(Default)]
pub struct InMemoryDatabase {
    next_id: i64,
    brackets: Vec<Bracket>,
    organisers: Vec<Organiser>,
}

impl DBAccessor for InMemoryDBAccessor {
    fn init(&self) -> Result<(), Error> {
        Ok(())
    }

    fn create_bracket<'a, 'b, 'c>(&'a self, bracket_name: &'b str) -> Result<(), Error<'c>> {
        let mut db = self.db.write().expect("database");
        let b = Bracket::new(db.next_id, bracket_name.to_string());
        db.next_id += 1;
        db.brackets.push(b);
        Ok(())
    }

    fn list_brackets<'a, 'b>(&'a self, _offset: i64) -> Result<Vec<Bracket>, Error<'b>> {
        let db = self.db.read().expect("database");
        Ok(db.brackets.clone())
    }

    fn find_brackets<'a, 'b, 'c>(
        &'a self,
        bracket_name: &'b str,
        _offset: i64,
    ) -> Result<Vec<Bracket>, Error<'c>> {
        let db = self.db.read().expect("database");
        Ok(db
            .brackets
            .clone()
            .into_iter()
            .filter(|b| b.clone().get_bracket_name() == bracket_name)
            .collect())
    }

    fn clean<'a, 'b>(&'a self) -> Result<(), Error<'b>> {
        let mut db = self.db.write().expect("database");
        db.next_id = 0;
        db.brackets = vec![];
        Ok(())
    }

    fn create_organiser<'a, 'b, 'c>(&'a self, organiser_name: &'b str) -> Result<(), Error<'c>> {
        let mut db = self.db.write().expect("database");
        db.organisers
            .push(Organiser::new(organiser_name.to_string()));
        Ok(())
    }
}
