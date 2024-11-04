//! Tournament description for players on how to participate

use chrono::{DateTime, Utc};
use std::cmp::PartialEq;
use totsugeki::format::Format;
use totsugeki::player::Player;
use uuid::Uuid;

/// Identifier
pub type ID = Uuid;

/// ID format for tournament
#[derive(Default, Debug, Copy, Clone)]
pub struct TournamentID(pub ID);

/// Tournament . Mostly common information such as
/// * bracket name
/// * start+end time
/// * location
///
/// These information may not be necessary to running the bracket, but they are
/// necessary for player
#[derive(Clone, Debug)]
pub struct Tournament {
    /// Identifier of this bracket
    id: TournamentID,
    /// Name of tournament
    name: String,
    /// Advertised start time
    start_time: Option<DateTime<Utc>>,
    /// Advertised end time
    end_time: Option<DateTime<Utc>>,
    /// Format
    format: Format,
    /// Participants
    participants: Participants,
}

impl Default for Tournament {
    fn default() -> Self {
        Self {
            id: TournamentID(ID::new_v4()),
            name: "".into(),
            start_time: None,
            end_time: None,
            format: Format::default(),
            participants: Participants::default(),
        }
    }
}

/// Player ID
#[derive(Default, Debug, Clone, PartialEq)]
struct PlayerID(pub ID);

/// Error
#[derive(Debug)]
pub enum ParticipantError {
    /// Player is already present
    AlreadyPresent,
}

/// Participants of tournament
///
/// Participants are ordered by seeding position from strongest to weakest
#[derive(Default, Debug, Clone)]
pub struct Participants(pub Vec<Player>);

impl Tournament {
    /// Add player to tournament
    pub fn add_participant(&mut self, player: Player) -> Result<(), ParticipantError> {
        if self
            .participants
            .0
            .iter()
            .any(|p| p.get_id() == player.get_id())
        {
            Err(ParticipantError::AlreadyPresent)
        } else {
            self.participants.0.push(player);
            Ok(())
        }
    }

    /// Get ID
    pub fn get_id(&self) -> TournamentID {
        self.id
    }

    /// Get name
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Get participants
    pub fn get_participants(&self) -> Participants {
        self.participants.clone()
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }
}
