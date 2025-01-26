//! Reporting for double elimination bracket

use totsugeki_core::bracket::seeding::Seeding;
use totsugeki_core::double_elimination_bracket::reporting::MatchReportError;
use totsugeki_core::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki_core::matches::result::{MatchFormat, Score};
use totsugeki_core::validation::AutomaticMatchValidationMode;
use totsugeki_core::ID;

#[test]
#[should_panic]
fn panics_when_bracket_is_empty() {
    let deb = DoubleEliminationBracket::create(
        Seeding::default(),
        AutomaticMatchValidationMode::default(),
        MatchFormat::ft2(),
        None,
    );

    deb.tournament_organiser_reports_result(ID::new_v4(), ID::new_v4(), Score(2, 0), ID::new_v4())
        .unwrap();
}

#[test]
fn reporting_twice_for_the_same_match_throws_error() {
    let seeding = vec![ID::new_v4(), ID::new_v4(), ID::new_v4()];
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
        panic!("MatchReportError::AlreadyReported was not thrown")
    };
}
