//! In-memory database
use crate::persistence::{Database, Error};
use totsugeki::Bracket;

/// In-memory database
#[derive(Default)]
pub struct InMemory {
    next_id: i64,
    brackets: Vec<Bracket>,
}

impl Database for InMemory {
    fn init(&self) -> Result<(), Error> {
        Ok(())
    }

    fn create_bracket<'a, 'b, 'c>(&'a mut self, bracket_name: &'b str) -> Result<(), Error<'c>> {
        let b = Bracket::new(self.next_id, bracket_name.to_string());
        self.next_id += 1;
        self.brackets.push(b);
        Ok(())
    }

    fn list_brackets<'a, 'b>(&'a self, _offset: i64) -> Result<Vec<Bracket>, Error<'b>> {
        Ok(self.brackets.clone())
    }

    fn find_brackets<'a, 'b, 'c>(
        &'a self,
        bracket_name: &'b str,
        _offset: i64,
    ) -> Result<Vec<Bracket>, Error<'c>> {
        Ok(self
            .brackets
            .clone()
            .into_iter()
            .filter(|b| b.clone().get_bracket_name() == bracket_name)
            .collect())
    }

    fn clean<'a, 'b>(&'a mut self) -> Result<(), Error<'b>> {
        self.next_id = 0;
        self.brackets = vec![];
        Ok(())
    }
}
