use totsugeki::matches::result::{MatchFormat, Score};
use totsugeki::matches::MatchResult;

#[test]
fn create_bracket_result() {
    MatchResult::new(Score(2, 0), MatchFormat::ft2()).unwrap();
    MatchResult::new(Score(0, 2), MatchFormat::ft2()).unwrap();
}
#[test]
fn intermediate_bracket_result() {
    let result = MatchResult::new(Score(0, 0), MatchFormat::ft2()).unwrap();
    assert!(!result.finished())
}
