//! Sqlite database
use crate::persistence::{Database, Error};
use crate::Bracket;
use sqlite::State;
use std::path::Path;

/// Sqlite database
pub struct Sqlite {
    savefile: String,
}

const LIMIT: i64 = 25;

impl Sqlite {
    /// Return a default sqlite database
    #[must_use]
    pub fn default() -> Self {
        Sqlite {
            savefile: "./tournament_server.db".to_string(),
        }
    }

    /// Connect to database which is a file in the current directory named
    /// `tournament_server.db`
    fn connection_string(&self) -> String {
        self.savefile.clone()
    }
}

impl Database for Sqlite {
    fn init(&self) -> Result<(), Error> {
        if !Path::exists(Path::new(self.connection_string().as_str())) {
            let connection = sqlite::open(self.connection_string())?;
            connection.execute(
                "
            CREATE TABLE brackets (id INTEGER primary key, bracket_name TEXT);
            ",
            )?;
        }
        Ok(())
    }

    fn create_bracket<'a, 'b, 'c>(&'a mut self, bracket_name: &'b str) -> Result<(), Error<'c>> {
        let connection = sqlite::open(self.connection_string())?;
        let mut statement = connection.prepare(
            "
        INSERT INTO brackets (bracket_name) VALUES (?)
        ",
        )?;
        statement.bind(1, bracket_name)?;
        statement.next()?;
        Ok(())
    }

    fn list_brackets<'a, 'b>(&'a self, offset: i64) -> Result<Vec<Bracket>, Error<'b>> {
        let connection = sqlite::open(self.connection_string())?;
        let mut statement = connection.prepare(
            "
        SELECT * FROM brackets
        ORDER BY id ASC
        LIMIT ?
        OFFSET ?
        ",
        )?;
        statement.bind(1, LIMIT)?;
        statement.bind(2, offset)?;

        let mut brackets = vec![];

        while let State::Row = statement.next()? {
            let id = statement.read::<i64>(0)?;
            let bracket_name = statement.read::<String>(1)?;
            brackets.push(Bracket::new(id, bracket_name));
        }

        Ok(brackets)
    }

    fn find_brackets<'a, 'b, 'c>(
        &'a self,
        bracket_name: &'b str,
        offset: i64,
    ) -> Result<Vec<Bracket>, Error<'c>> {
        let connection = sqlite::open(self.connection_string())?;
        let mut statement = connection.prepare(
            "
        SELECT * FROM brackets
        WHERE bracket_name = ?
        ORDER BY id ASC
        LIMIT ?
        OFFSET ?
        ",
        )?;
        statement.bind(1, bracket_name)?;
        statement.bind(2, LIMIT)?;
        statement.bind(3, offset)?;

        let mut brackets = vec![];

        while let State::Row = statement.next()? {
            let id = statement.read::<i64>(0)?;
            let bracket_name = statement.read::<String>(1)?;
            brackets.push(Bracket::new(id, bracket_name));
        }

        Ok(brackets)
    }

    // NOTE: does nothing
    fn clean<'a, 'b>(&'a mut self) -> Result<(), Error<'b>> {
        Ok(())
    }
}

impl<'a> From<sqlite::Error> for Error<'a> {
    fn from(e: sqlite::Error) -> Self {
        // NOTE: all sqlite error codes https://www.sqlite.org/c3ref/c_abort.html
        match e.code {
            Some(code) => match code {
                3 | 23 => Error::Denied(),
                _ => Error::Code(e.to_string()),
            },
            None => Error::Code(e.to_string()),
        }
    }
}
