//! Reporting for double elimination bracket

use totsugeki::bracket::seeding::Seeding;
use totsugeki::double_elimination_bracket::reporting::MatchReportError;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::matches::result::{MatchFormat, Score};
use totsugeki::matches::MatchID;
use totsugeki::player::PlayerID;
use totsugeki::validation::AutomaticMatchValidationMode;

#[test]
#[should_panic]
fn panics_when_bracket_is_empty() {
    let deb = DoubleEliminationBracket::create(
        Seeding::default(),
        AutomaticMatchValidationMode::default(),
        MatchFormat::ft2(),
        None,
    );

    deb.tournament_organiser_reports_result(
        MatchID::default(),
        PlayerID::create(),
        Score(2, 0),
        PlayerID::create(),
    )
    .unwrap();
}

#[test]
fn reporting_twice_for_the_same_match_throws_error() {
    let seeding = vec![PlayerID::create(), PlayerID::create(), PlayerID::create()];
    let deb = DoubleEliminationBracket::create(
        Seeding::new(seeding).unwrap(),
        AutomaticMatchValidationMode::Flexible,
        MatchFormat::ft2(),
        None,
    );
    let match_id_seed_2_vs_seed_3 = deb.get_matches()[0].get_id();

    let players = deb.get_seeding().get();
    let s2 = players[1];
    let s3 = players[2];
    assert!(!deb.get_matches()[0].is_over());
    let (deb, _n) = deb
        .tournament_organiser_reports_result(match_id_seed_2_vs_seed_3, s2, Score(2, 0), s3)
        .unwrap();
    assert!(deb.get_matches()[0].is_over());
    let Err(MatchReportError::AlreadyReported) =
        deb.tournament_organiser_reports_result(match_id_seed_2_vs_seed_3, s2, Score(2, 0), s3)
    else {
        panic!("MatchReportError::AlreadyReported was not throwned")
    };
}
