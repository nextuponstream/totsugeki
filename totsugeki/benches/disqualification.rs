use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use mycrate::fibonacci;
use totsugeki::{bracket::Bracket, player::Player};

// run with `cargo bench`

// TODO optimise disqualifying and create benchmark for 8k player
// disqualification bracket

fn disqualify_many_people(n: usize) {
    let mut bracket = Bracket::default();

    for i in 0..n {
        let p = Player::new(format!("p{i}"));
        bracket = bracket
            .unchecked_join_skip_matches_generation(p)
            .expect("new bracket");
    }

    bracket = bracket.generate_matches().expect("bracket with matches");
    bracket = bracket.start().expect("bracket started").0;

    let players = bracket.get_participants().get_players_list();

    for p in players {
        if !bracket.is_over() {
            bracket = bracket
                .disqualify_participant(p.get_id())
                .expect("updated bracket")
                .0;
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("disqualifying");
    // Configure Criterion.rs to detect smaller differences and increase sample size to improve
    // precision and counteract the resulting noise.
    // FIXME tweak significance level until there is very few "regression/
    // improvement" when benchmarking the same code
    group.significance_level(0.02).sample_size(12);
    group.bench_function("all players", |b| {
        b.iter(|| disqualify_many_people(black_box(500)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
