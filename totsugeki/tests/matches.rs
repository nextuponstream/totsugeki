use totsugeki::matches::{BracketResult, BracketResultGenerationError};

#[test]
fn create_bracket_result() {
    BracketResult::new(2, 0).unwrap();
    BracketResult::new(0, 2).unwrap();
}
#[test]
fn invalid_bracket_result() {
    let Err(BracketResultGenerationError::Invalid(0, 0)) = BracketResult::new(0, 0) else {
        panic!("Expected error")
    };
}
