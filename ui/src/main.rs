fn main() {
    /// For reference, evo 2023 SF6 tournament had 7k participants. However, it
    /// is divided in pools. Then online tournaments are the only tournaments
    /// where bracket are not divided in pools.
    /// Then last bracket I remember being big is a youtube thumbnail for 500
    /// man bracket.
    /// Css should not be too big, ideally, less than 5Mb. Generated js file
    /// has size:
    /// * 200: 0.2Mb
    /// * 500: 0.4Mb
    /// * 10k: 14Mb
    ///
    /// 10k is too much expected css. Then let's not accomodate it.
    const EXPECTED_MAX_PLAYERS: usize = 512;
    const DOUBLE_EXPECTATIONS: usize = 2 * EXPECTED_MAX_PLAYERS;
    const DOUBLE_EXPECTATIONS_AGAIN: usize = 2 * DOUBLE_EXPECTATIONS;

    /// All players are matched against each other, then divide player number
    /// by 2
    const MAX_MATCHES_FIRST_ROUND: usize = DOUBLE_EXPECTATIONS_AGAIN / 2;
    println!("export const rowSetup = {{");
    for i in 1..=MAX_MATCHES_FIRST_ROUND {
        println!("    '{i}': '{i}',");
    }
    println!("}};");

    println!("export const gridSetup = {{");
    // As the number of participants grows, the lower bracket gets longer than
    // the winner bracket. Getting sent to the loser bracket in the first round
    // means you have double the amount of matches to play.
    // 2^16: ~65k > 40k expected players
    // 2 * 3: 2 is for grand finals and winners finals
    // I will not introduce a new type/library to accodomate very big numbers.
    // standard library should suffice and we expect to remain in that range.
    let mut rounds: usize = 0;
    while rounds
        .checked_next_power_of_two()
        .expect("number within usize range")
        + 2 * 3
        < DOUBLE_EXPECTATIONS_AGAIN
    {
        rounds += 1;
    }
    // eprintln!("{rounds}");
    // Doing * 2 is probably not correct but correct enough for drawing
    // brackets. Then let's triple it:
    let max_expected_rounds_in_losers: usize = rounds * 3;
    // 3: round column + lines flowing out of + lines flowing into next round
    let max_expected_cols: usize = max_expected_rounds_in_losers * 3;
    // 12 default in tailwindcs
    for i in 13..=max_expected_cols {
        println!("    '{i}': 'repeat({i}, minmax(1.fr, 1.fr))',")
    }
    println!("}};");

    // While the theme can be extended, that does not mean those additionnal
    // style won't get purged. Then you need to add a safelist.

    println!("export const safelist = [");
    for i in 1..=max_expected_rounds_in_losers {
        println!("'row-start-{i}',");
    }
    for i in 1..=max_expected_cols {
        println!("'grid-cols-{i}',");
    }
    println!("];");
}
