fn main() {
    const ROWS: i32 = 16384;
    for i in 1..=ROWS {
        println!("        '{i}': '{i}',");
    }
}
