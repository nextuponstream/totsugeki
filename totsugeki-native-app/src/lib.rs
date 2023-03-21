pub mod components;
pub mod single_elimination;

use totsugeki::{
    matches::{Id as MatchId, Match},
    player::{Id as PlayerId, Participants},
};

#[derive(Debug, Clone, Copy)]
// #[derive(Clone, Copy)]
pub struct DisplayableMatch {
    id: MatchId,
    players: [[u8; 256]; 2],
    score: (i8, i8),
    seeds: [usize; 2],
    row_hint: Option<usize>,
}

impl Default for DisplayableMatch {
    fn default() -> Self {
        let mut v: Vec<u8> = vec![];
        v.resize(256, 0);
        DisplayableMatch {
            id: MatchId::new_v4(),
            players: [v.clone().try_into().unwrap(), v.try_into().unwrap()],
            score: (0, 0),
            seeds: [0, 0],
            row_hint: None,
        }
    }
}

// impl std::fmt::Debug for DisplayableMatch {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?} {:?}", self.seeds, self.row_hint)
//     }
// }

impl DisplayableMatch {
    #[cfg(test)]
    fn new(seeds: [usize; 2]) -> Self {
        Self {
            seeds,
            ..Self::default()
        }
    }

    fn player(&self, is_player1: bool) -> &str {
        let id = if is_player1 { 0 } else { 1 };
        std::str::from_utf8(&self.players[id]).unwrap()
    }

    fn player1(&self) -> &str {
        self.player(true)
    }

    fn player2(&self) -> &str {
        self.player(false)
    }

    fn score1(&self) -> String {
        self.score.0.to_string()
    }

    fn score2(&self) -> String {
        self.score.1.to_string()
    }
}

fn convert(m: &Match, participants: &Participants) -> DisplayableMatch {
    let list = participants.get_players_list();
    let players: Vec<(PlayerId, String)> =
        list.iter().map(|p| (p.get_id(), p.get_name())).collect();
    let player_name_size = 256;
    let mut player1_name = m.get_players()[0]
        .get_name(&players)
        .into_bytes()
        .into_iter()
        .take(256)
        .collect::<Vec<u8>>();
    player1_name.resize(player_name_size, 0); // '\0' null byte
    let player1 = player1_name.try_into().unwrap();
    let mut player2_name = m.get_players()[1]
        .get_name(&players)
        .into_bytes()
        .into_iter()
        .take(256)
        .collect::<Vec<u8>>();
    player2_name.resize(player_name_size, 0); // '\0' null byte
    let player2 = player2_name.try_into().unwrap();
    DisplayableMatch {
        id: m.get_id(),
        players: [player1, player2],
        score: m.get_score(),
        seeds: m.get_seeds(),
        row_hint: None,
    }
}
