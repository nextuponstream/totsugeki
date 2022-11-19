//! Disqualification of player in bracket and all side-effects

use crate::{
    bracket::{Bracket, Error},
    matches::Match,
    player::Id as PlayerId,
};

/// Returns new matches when comparing old bracket and new bracket
pub(crate) fn get_new_matches(old_bracket: &[Match], new_bracket: &[Match]) -> Vec<Match> {
    new_bracket
        .iter()
        .filter(|new_m| {
            !old_bracket
                .iter()
                .any(|old_m| old_m.get_id() == new_m.get_id())
        })
        .cloned()
        .collect::<Vec<Match>>()
}

impl Bracket {
    /// Disqualify player from bracket, advance opponent in bracket and returns
    /// updated bracket
    ///
    /// # Errors
    /// thrown when referred player does not belong in current bracket, bracket
    /// has not started/is over or participant has already been disqualified
    pub fn disqualify_participant(
        self,
        player_id: PlayerId,
    ) -> Result<(Bracket, Vec<Match>), Error> {
        if !self.accept_match_results {
            return Err(Error::NotStarted(
                self.bracket_id,
                ". Cannot disqualify at this time.".into(),
            ));
        }

        let p = self.format.get_progression(
            self.get_matches(),
            self.get_participants(),
            self.automatic_match_progression,
        );
        match p.disqualify_participant(player_id) {
            Ok(el) => Ok((
                Self {
                    matches: el.0,
                    ..self
                },
                el.1,
            )),
            Err(e) => Err(Error::Progression(self.bracket_id, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bracket::{raw::Raw, Id as BracketId},
        format::Format,
        player::Participants,
        seeding::{
            single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored,
            Method as SeedingMethod,
        },
    };
    use chrono::prelude::*;

    #[test]
    fn cannot_disqualify_player_before_bracket_starts() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id];
        let player_names = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let players = Participants::from_raw_id(
            player_ids
                .iter()
                .zip(player_names.iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let bracket_id = BracketId::new_v4();
        let bracket: Bracket = Raw {
            bracket_id,
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");
        match bracket.disqualify_participant(p1_id) {
            Ok((b, _)) => panic!("Expected error, bracket: {b}"),
            Err(e) => match e {
                Error::NotStarted(id, _) => assert_eq!(id, bracket_id),
                _ => panic!("Expected Started error, got {e}"),
            },
        }
    }
}
