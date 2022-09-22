//! Query state of bracket

use crate::{
    bracket::{Bracket, Error},
    format::Format::{DoubleElimination, SingleElimination},
    matches::{Id as MatchId, Match},
    opponent::Opponent,
    player::{Id as PlayerId, Player},
};

impl Bracket {
    /// Returns true if player has been disqualified from bracket
    #[must_use]
    pub(super) fn is_disqualified(&self, player_id: PlayerId) -> bool {
        self.matches
            .iter()
            .any(|m| m.is_automatic_looser_by_disqualification(player_id))
    }

    /// Returns true if bracket is over
    fn bracket_is_over(bracket_matches: &[Match]) -> bool {
        !bracket_matches.iter().any(|m| !m.is_over())
    }

    /// Returns true if bracket is over (all matches are played)
    #[must_use]
    pub(super) fn is_over(&self) -> bool {
        match self.format {
            SingleElimination => Self::bracket_is_over(&self.matches),
            DoubleElimination => {
                let (winner_bracket, looser_bracket, gf, gfr) =
                    Match::partition_double_elimination_matches(
                        &self.matches,
                        self.participants.len(),
                    )
                    .expect("partition");
                Self::bracket_is_over(&winner_bracket)
                    && Self::bracket_is_over(&looser_bracket)
                    && gf.is_over()
                    && (gf.stronger_seed_wins() || gfr.is_over())
            }
        }
    }

    /// Return next opponent for `player_id`, relevant match and player name
    ///
    /// # Errors
    /// Thrown when matches have yet to be generated or player has won/been
    /// eliminated
    pub fn next_opponent(&self, player_id: PlayerId) -> Result<(Opponent, MatchId, String), Error> {
        if !self
            .participants
            .clone()
            .get_players_list()
            .iter()
            .map(Player::get_id)
            .any(|id| id == player_id)
        {
            return Err(Error::PlayerIsNotParticipant(player_id, self.bracket_id));
        }
        if self.matches.is_empty() {
            return Err(Error::NoGeneratedMatches(self.bracket_id));
        }

        if self.is_disqualified(player_id) {
            return Err(Error::DisqualifiedPlayerHasNoNextOpponent(
                self.bracket_id,
                player_id,
            ));
        }

        let player = Opponent::Player(player_id);
        let next_match = self.matches.iter().find(|m| {
            (m.get_players()[0] == player || m.get_players()[1] == player)
                && m.get_winner() == Opponent::Unknown
        });
        let relevant_match = match next_match {
            Some(m) => m,
            None => {
                let last_match = self.matches.iter().last().expect("last match");
                if let Opponent::Player(id) = last_match.get_winner() {
                    if id == player_id {
                        return Err(Error::NoNextMatch(player_id, self.bracket_id));
                    }
                }
                return Err(Error::EliminatedFromBracket(player_id, self.bracket_id));
            }
        };

        let opponent = if relevant_match.get_players()[0] == player {
            relevant_match.get_players()[1]
        } else {
            relevant_match.get_players()[0]
        };
        let player_name = match opponent {
            Opponent::Player(opponent_id) => self
                .participants
                .clone()
                .get_players_list()
                .iter()
                .find(|p| p.id == opponent_id)
                .map_or_else(|| Opponent::Unknown.to_string(), Player::get_name),
            Opponent::Unknown => Opponent::Unknown.to_string(),
        };

        Ok((opponent, relevant_match.get_id(), player_name))
    }
}
