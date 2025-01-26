//! bracket management
//! Tournament description for players on how to participate

mod create;
mod join;
mod list;
mod new;
mod report_result;
mod save_bracket_from_steps;
mod show;
pub(crate) mod tournament_players;
pub(crate) mod update_with_result;
mod user_tournaments;

// Flatten exports when reusing
pub(crate) use crate::tournaments::create::*;
pub(crate) use crate::tournaments::join::*;
pub(crate) use crate::tournaments::list::*;
pub(crate) use crate::tournaments::new::*;
pub(crate) use crate::tournaments::report_result::*;
pub(crate) use crate::tournaments::save_bracket_from_steps::*;
pub(crate) use crate::tournaments::show::*;
use crate::tournaments::tournament_players::TournamentPlayer;
pub(crate) use crate::tournaments::update_with_result::*;
pub(crate) use crate::tournaments::user_tournaments::*;
use crate::ID;
use axum::{response::IntoResponse, Json as AxumJson};
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use totsugeki_core::bracket::seeding::Seeding;
use totsugeki_core::bracket::Id;
use totsugeki_core::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki_core::format::Format;
use totsugeki_core::matches::result::MatchFormat;
use totsugeki_core::matches::Match;
use totsugeki_core::player::Player;
use totsugeki_core::validation::AutomaticMatchValidationMode;
use totsugeki_display::loser_bracket::lines as loser_bracket_lines;
use totsugeki_display::loser_bracket::reorder as reorder_loser_bracket;
use totsugeki_display::winner_bracket::lines as winner_bracket_lines;
use totsugeki_display::winner_bracket::reorder as reorder_winner_bracket;
use totsugeki_display::{from_participants, BoxElement, MinimalMatch};
use validator::Validate;

/// List of players from which a bracket can be created
#[derive(Debug, Deserialize)]
pub struct ReportResultInput {
    /// current state of the bracket
    pub bracket: DoubleEliminationBracket,
    /// tournament
    pub tournament: Tournament,
    /// First player
    pub player1_id: ID,
    /// Second player
    pub player2_id: ID,
    /// player 1 score
    pub score_p1: u8,
    /// player 2 score
    pub score_p2: u8,
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
    /// Participants (seeding + names)
    pub participants: Participants,
}

/// List of players from which a bracket can be created
#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct CreateTournamentForm {
    #[validate(length(min = 1))]
    /// tournament name
    pub tournament_name: String,
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
    pub player1_id: ID,
    /// low seed player
    pub player2_id: ID,
    /// score of player 1
    pub score_p1: u8,
    /// score of player 2
    pub score_p2: u8,
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
    tournament: &Tournament,
    bracket: DoubleEliminationBracket,
    user_id: Option<ID>,
    is_tournament_organiser: bool,
) -> impl IntoResponse {
    // TODO test if tracing shows from which methods it was called
    let winner_bracket_rounds = match bracket.partition_winner_bracket() {
        Ok(winner_bracket_matches) => {
            let mut winner_bracket_rounds = vec![];
            for r in winner_bracket_matches {
                let round = r
                    .iter()
                    .map(|m| {
                        from_participants(
                            m,
                            &tournament
                                .get_players()
                                .into_iter()
                                .map(|v| v.into())
                                .collect::<Vec<Player>>(),
                        )
                    })
                    .collect();
                winner_bracket_rounds.push(round);
            }

            reorder_winner_bracket(&mut winner_bracket_rounds);
            Some(winner_bracket_rounds)
        }
        Err(totsugeki_core::bracket::PartitionError::NotEnoughPlayersInBracket) => None,
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
                    .map(|m| {
                        from_participants(
                            m,
                            &tournament
                                .get_players()
                                .into_iter()
                                .map(|tp| tp.into())
                                .collect::<Vec<Player>>(),
                        )
                    })
                    .collect();
                loser_bracket_rounds.push(round);
            }
            reorder_loser_bracket(&mut loser_bracket_rounds);
            Some(loser_bracket_rounds)
        }
        Err(totsugeki_core::bracket::PartitionError::NotEnoughPlayersInBracket) => None,
    };
    let maybe_loser_bracket_lines = match loser_bracket_rounds.clone() {
        Some(loser_bracket_rounds) => loser_bracket_lines(loser_bracket_rounds),
        None => None,
    };

    let (gf, gf_reset) = match bracket.grand_finals_and_reset() {
        Ok((gf, gf_reset)) => {
            let gf = from_participants(
                &gf,
                &tournament
                    .get_players()
                    .into_iter()
                    .map(|tp| tp.into())
                    .collect::<Vec<Player>>(),
            );
            let gf_reset = from_participants(
                &gf_reset,
                &tournament
                    .get_players()
                    .into_iter()
                    .map(|tp| tp.into())
                    .collect::<Vec<Player>>(),
            );
            (Some(gf), Some(gf_reset))
        }
        Err(totsugeki_core::bracket::PartitionError::NotEnoughPlayersInBracket) => (None, None),
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
        participants: Participants(
            tournament
                .get_players()
                .into_iter()
                .map(|tp| tp.into())
                .collect::<Vec<Player>>(),
        ),
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
    pub created_at: OffsetDateTime,
    /// Format of tournament
    pub format: totsugeki_core::format::Format,
}

/// Deserialize in tournament information and double elimination bracket
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub(crate) struct TournamentAugmentedRecord {
    /// bracket ID
    pub id: ID,
    /// name
    pub name: String,
    /// creation date
    pub created_at: OffsetDateTime,
    /// Format of tournament
    pub format: Format,
    /// Players of tournament
    // FIXME alway return array and not null for empty array
    pub players: Option<Vec<PlayerRecord>>,
    /// Matches
    pub matches: Option<Vec<MatchRecord>>,
}

/// Represents a tournament player from the `players` database table
#[derive(sqlx::Type, Deserialize, Serialize, Clone)]
pub(crate) struct PlayerRecord {
    /// User ID of player
    pub id: ID,
    /// Name of player
    // option because it's a result of coalesce
    pub name: Option<String>,
    /// User ID if any
    pub user_id: Option<ID>,
    /// Guest ID if any
    pub guest_id: Option<ID>,
}

impl From<PlayerRecord> for TournamentPlayer {
    fn from(value: PlayerRecord) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            guest_id: value.guest_id,
            name: value.name.expect("name"),
        }
    }
}

/// Represents a match retrieved from the `matches` database table.
#[derive(sqlx::Type, Deserialize, Serialize, Clone)]
pub(crate) struct MatchRecord {
    /// Index of match, useful for display purpose
    pub pos: i16,
    /// Match ID
    pub match_id: ID,
    /// Match format (example: first to X)
    pub format: String,
    /// Additionnal match format information (example: first to 3)
    pub format_n: i16,
    /// left seed (highest seed) for the presumed strongest predicted player
    pub high_seed: i16,
    /// presumed stronger player
    pub high_seed_player: Option<ID>,
    /// right seed (lowest seed) for the presumed weakest predicted player
    pub low_seed: i16,
    /// presumed weakest player
    pub low_seed_player: Option<ID>,
}

impl From<MatchRecord> for Match {
    fn from(value: MatchRecord) -> Self {
        let players = [value.high_seed_player.into(), value.low_seed_player.into()];
        let seeds = [
            value.high_seed.to_usize().expect("high seed"),
            value.low_seed.to_usize().expect("low seed"),
        ];
        let format =
            MatchFormat::new(value.format_n.to_u8().expect("format n")).expect("match format");

        Match::new(Some(value.match_id), players, seeds, format).expect("match from match data")
    }
}

impl From<PlayerRecord> for Player {
    fn from(value: PlayerRecord) -> Self {
        Player::from((value.id, value.name.expect("name").as_str()))
    }
}

impl TournamentAugmentedRecord {
    /// Retrieve data from tournament record
    pub fn parse(self) -> (Tournament, DoubleEliminationBracket) {
        // let players = self.players.clone().into_iter().map(|v| v.into()).collect();
        let tournament: Tournament = self.into();
        // let bracket = DoubleEliminationBracket::new(
        //     self.matches.into_iter().map(|v| v.into()).collect(),
        //     Seeding::new(self.players.into_iter().map(|v| PlayerID(v.id)).collect())
        //         .expect("use seeding from database record"),
        //     AutomaticMatchValidationMode::Flexible, // FIXME should be in tournament record
        // );
        let bracket = DoubleEliminationBracket::new(
            vec![],
            Seeding::new(vec![]).expect("use seeding from database record"),
            AutomaticMatchValidationMode::Flexible, // FIXME should be in tournament record
        );

        (tournament, bracket)
    }
}

impl From<TournamentAugmentedRecord> for Tournament {
    fn from(value: TournamentAugmentedRecord) -> Self {
        Self {
            id: value.id,
            name: value.name,
            start_time: None, // FIXME
            end_time: None,   // FIXME
            format: value.format,
            players: value.players.map_or(vec![], |players| {
                players.into_iter().map(|p| p.into()).collect()
            }),
        }
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
#[allow(unused)]
pub struct Tournament {
    /// Identifier of tournament
    id: ID,
    /// Name of tournament
    name: String,
    /// Advertised start time
    start_time: Option<DateTime<Utc>>,
    /// Advertised end time
    end_time: Option<DateTime<Utc>>,
    /// Format
    format: Format,
    /// Participants
    players: Vec<TournamentPlayer>,
}

impl Default for Tournament {
    fn default() -> Self {
        Self {
            id: ID::new_v4(),
            name: String::new(),
            start_time: None,
            end_time: None,
            format: Format::default(),
            players: vec![],
        }
    }
}

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
    pub fn add_player(&mut self, player: TournamentPlayer) -> Result<(), ParticipantError> {
        if self.players.iter().any(|p| p.get_id() == player.get_id()) {
            Err(ParticipantError::AlreadyPresent)
        } else {
            self.players.push(player);
            Ok(())
        }
    }

    /// Create new guests from `guest_names`
    pub fn add_guests(&mut self, guest_names: &[String]) {
        for (index, guest_name) in guest_names.iter().enumerate() {
            let tournament_player = TournamentPlayer::new_guest(guest_name.clone());
            self.players.push(tournament_player);
        }
    }

    /// Get ID
    #[must_use]
    pub fn get_id(&self) -> ID {
        self.id
    }

    /// Get name
    #[must_use]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Get participants
    #[must_use]
    pub fn get_players(&self) -> Vec<TournamentPlayer> {
        self.players.clone()
    }

    /// Set name of tournament
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }
}
