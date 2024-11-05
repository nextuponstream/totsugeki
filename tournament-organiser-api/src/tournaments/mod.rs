//! bracket management
//! Tournament description for players on how to participate

mod create;
mod join;
mod list;
mod new;
mod report_result;
mod save_bracket_from_steps;
mod show;
mod update_with_result;
mod user_tournaments;

// Flatten exports when reusing
use crate::repositories::brackets::MatchesRaw;
pub(crate) use crate::tournaments::create::*;
pub(crate) use crate::tournaments::join::*;
pub(crate) use crate::tournaments::list::*;
pub(crate) use crate::tournaments::new::*;
pub(crate) use crate::tournaments::report_result::*;
pub(crate) use crate::tournaments::save_bracket_from_steps::*;
pub(crate) use crate::tournaments::show::*;
pub(crate) use crate::tournaments::update_with_result::*;
pub(crate) use crate::tournaments::user_tournaments::*;
use axum::{response::IntoResponse, Json as AxumJson};
use chrono::{DateTime, Utc};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::types::Json as SqlxJson;
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use totsugeki::bracket::seeding::Seeding;
use totsugeki::bracket::Id;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::format::Format;
use totsugeki::player::{Id as PlayerId, Participants as TotsugekiParticipants, Player};
use totsugeki::validation::AutomaticMatchValidationMode;
use totsugeki_display::loser_bracket::lines as loser_bracket_lines;
use totsugeki_display::loser_bracket::reorder as reorder_loser_bracket;
use totsugeki_display::winner_bracket::lines as winner_bracket_lines;
use totsugeki_display::winner_bracket::reorder as reorder_winner_bracket;
use totsugeki_display::{from_participants, BoxElement, MinimalMatch};
use uuid::Uuid;
use validator::Validate;

/// List of players from which a bracket can be created
#[derive(Debug, Deserialize)]
pub struct ReportResultInput {
    /// current state of the bracket
    pub bracket: DoubleEliminationBracket,
    /// tournament
    pub tournament: Tournament,
    /// First player
    pub p1_id: PlayerId,
    /// Second player
    pub p2_id: PlayerId,
    /// player 1 score
    pub score_p1: i8,
    /// player 2 score
    pub score_p2: i8,
}

/// Bracket to display. When there is less than 3 players, then there is nothing
/// to display
#[derive(Serialize, Debug, Deserialize)]
pub struct BracketDisplay {
    /// Winner bracket matches and lines to draw
    pub winner_bracket: Option<Vec<Vec<MinimalMatch>>>,
    /// Lines to draw between winner bracket matches
    pub winner_bracket_lines: Option<Vec<Vec<BoxElement>>>,
    /// Loser bracket matches and lines to draw
    pub loser_bracket: Option<Vec<Vec<MinimalMatch>>>,
    /// Lines to draw between loser bracket matches
    pub loser_bracket_lines: Option<Vec<Vec<BoxElement>>>,
    /// Grand finals
    pub grand_finals: Option<MinimalMatch>,
    /// Grand finals reset
    pub grand_finals_reset: Option<MinimalMatch>,
    /// Bracket object to update
    pub bracket: DoubleEliminationBracket,
    /// true if user requesting the data is also a TO
    pub is_tournament_organiser: bool,
    /// true if user requesting the data participates
    pub is_participant: bool,
}

/// List of players from which a bracket can be created
#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct CreateBracketForm {
    #[validate(length(min = 1))]
    /// bracket names
    pub bracket_name: String,
    /// player names
    pub player_names: Vec<String>,
}

/// Result reported by player
///
/// FIXME there probably is a less computive intensive way to save steps of
/// match, like only saving relevant match ID to update. But it's not there.
/// Then this will do.
#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerMatchResultReport {
    /// high seed player
    pub p1_id: PlayerId,
    /// low seed player
    pub p2_id: PlayerId,
    /// score of player 1
    pub score_p1: i8,
    /// score of player 2
    pub score_p2: i8,
}

/// List of players from which a bracket can be created
#[derive(Deserialize, Serialize, Debug)]
pub struct BracketState {
    /// bracket names
    pub bracket_name: String,
    /// player names
    pub players: Vec<Player>,
    ///  results in order of replay
    pub results: Vec<PlayerMatchResultReport>,
}

/// Breaks down bracket in small parts to be presented by UI
fn breakdown(
    tournament: Tournament,
    bracket: DoubleEliminationBracket,
    user_id: Option<totsugeki::player::Id>,
    is_tournament_organiser: bool,
) -> impl IntoResponse {
    // TODO test if tracing shows from which methods it was called
    let winner_bracket_rounds = match bracket.partition_winner_bracket() {
        Ok(winner_bracket_matches) => {
            let mut winner_bracket_rounds = vec![];
            for r in winner_bracket_matches {
                let round = r
                    .iter()
                    .map(|m| from_participants(m, &tournament.get_participants().0))
                    .collect();
                winner_bracket_rounds.push(round);
            }

            reorder_winner_bracket(&mut winner_bracket_rounds);
            Some(winner_bracket_rounds)
        }
        Err(totsugeki::bracket::PartitionError::NotEnoughPlayersInBracket) => None,
    };
    let maybe_winner_bracket_lines = match winner_bracket_rounds.clone() {
        Some(winner_bracket_rounds) => winner_bracket_lines(&winner_bracket_rounds),
        None => None,
    };

    let loser_bracket_rounds = match bracket.partition_loser_bracket() {
        Ok(lower_bracket_matches) => {
            let mut loser_bracket_rounds: Vec<Vec<MinimalMatch>> = vec![];
            for r in lower_bracket_matches {
                let round = r
                    .iter()
                    .map(|m| from_participants(m, &tournament.get_participants().0))
                    .collect();
                loser_bracket_rounds.push(round);
            }
            reorder_loser_bracket(&mut loser_bracket_rounds);
            Some(loser_bracket_rounds)
        }
        Err(totsugeki::bracket::PartitionError::NotEnoughPlayersInBracket) => None,
    };
    let maybe_loser_bracket_lines = match loser_bracket_rounds.clone() {
        Some(loser_bracket_rounds) => loser_bracket_lines(loser_bracket_rounds),
        None => None,
    };

    let (gf, gf_reset) = match bracket.grand_finals_and_reset() {
        Ok((gf, gf_reset)) => {
            let gf = from_participants(&gf, &tournament.get_participants().0);
            let gf_reset = from_participants(&gf_reset, &tournament.get_participants().0);
            (Some(gf), Some(gf_reset))
        }
        Err(totsugeki::bracket::PartitionError::NotEnoughPlayersInBracket) => (None, None),
    };

    let is_participant = match user_id {
        Some(participant_id) => bracket.get_seeding().contains(participant_id),
        None => false,
    };

    let bracket = BracketDisplay {
        winner_bracket: winner_bracket_rounds,
        winner_bracket_lines: maybe_winner_bracket_lines,
        loser_bracket: loser_bracket_rounds,
        loser_bracket_lines: maybe_loser_bracket_lines,
        grand_finals: gf,
        grand_finals_reset: gf_reset,
        bracket,
        is_participant,
        is_tournament_organiser,
    };
    tracing::info!("displaying bracket {}", tournament.get_id());
    tracing::debug!("displaying bracket {:?}", bracket);
    (StatusCode::OK, AxumJson(bracket)).into_response()
}

#[derive(Serialize, Deserialize)]
/// 201 response
pub struct GenericResourceCreated {
    /// Resource ID
    pub id: Id,
}

/// Deserialize in tournament information and double elimination bracket
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub(crate) struct TournamentRecord {
    /// bracket ID
    pub id: Id,
    /// name
    pub name: String,
    /// creation date
    pub created_at: DateTime<Utc>,
    /// matches (agnostic to tournament format)
    pub matches: SqlxJson<MatchesRaw>,
    /// participants
    pub participants: SqlxJson<TotsugekiParticipants>,
}

impl TournamentRecord {
    /// Retrieve data from tournament record
    pub fn parse(self) -> (Tournament, DoubleEliminationBracket) {
        let tournament = Tournament::new_from_database_record(
            self.id,
            self.name,
            self.participants.0.get_players_list(),
        );
        let bracket = DoubleEliminationBracket::new(
            self.matches.0 .0,
            Seeding::new(self.participants.0.get_seeding()).unwrap(),
            AutomaticMatchValidationMode::Flexible, // FIXME should be in tournament record
        );

        (tournament, bracket)
    }
}

/// Identifier
pub type ID = Uuid;

/// ID format for tournament
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct TournamentID(pub ID);

impl Display for TournamentID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tournament . Mostly common information such as
/// * bracket name
/// * start+end time
/// * location
///
/// These information may not be necessary to running the bracket, but they are
/// necessary for player
#[derive(Clone, Debug, Deserialize)]
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

impl Tournament {
    /// New tournament from database record
    pub fn new_from_database_record(id: ID, name: String, participants: Vec<Player>) -> Self {
        Self {
            id: TournamentID(id),
            name,
            start_time: None,
            end_time: None,
            format: Default::default(),
            participants: Participants(participants),
        }
    }
}

/// Player ID
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Participants(pub Vec<Player>);

impl Participants {
    /// Ordered list for seeding
    pub fn get_seeding(&self) -> Vec<ID> {
        self.0.iter().map(Player::get_id).collect()
    }
}

impl Tournament {
    /// Add player to tournament
    /// # Errors
    /// when player is duplicate
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
    #[must_use]
    pub fn get_id(&self) -> TournamentID {
        self.id
    }

    /// Get name
    #[must_use]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Get participants
    #[must_use]
    pub fn get_participants(&self) -> Participants {
        self.participants.clone()
    }

    /// Set name of tournament
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }
}
