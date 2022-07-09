use clap::Parser;
use jwt::SignWithKey;
use totsugeki_api::{hmac, DiscordApiUser};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long, value_parser)]
    server_key: String,
}

fn main() {
    let args = Args::parse();

    let key = args.server_key.as_bytes();
    let key = hmac(key);
    let token = DiscordApiUser {}.sign_with_key(&key).unwrap();
    println!("{}", token)
}
