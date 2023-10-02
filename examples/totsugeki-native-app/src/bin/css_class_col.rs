fn main() {
    // 16'384=2^14 rows
    // 14 rounds to get a winner
    // lets take 60 rounds and be done with it
    const COLS: i32 = 60 * 3;
    for i in 13..=COLS {
        println!("        '{i}': 'repeat({i}, minmax(0, 1fr))',");
    }
}
