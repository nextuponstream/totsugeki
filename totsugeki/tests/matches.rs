use totsugeki::matches::result::{MatchFormat, Score};
use totsugeki::matches::MatchScore;

#[test]
fn create_bracket_result() {
    MatchScore::new(Score(2, 0), MatchFormat::ft2()).unwrap();
    MatchScore::new(Score(0, 2), MatchFormat::ft2()).unwrap();
}
#[test]
fn intermediate_bracket_result() {
    let result = MatchScore::new(Score(0, 0), MatchFormat::ft2()).unwrap();
    assert!(!result.finished())
}
