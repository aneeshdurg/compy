use compyrs;
use std::env;

// TODO use a real arg parsing crate to match compgen's features
pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Error!");
        return;
    }

    compyrs::path_prefix_completion(&args[1]);
    compyrs::path_fuzzy_completion(&args[1]);
}
