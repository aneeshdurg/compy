use compyrs;
use std::env;
use std::process;

// TODO use a real arg parsing crate to match compgen's features
pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Error!");
        process::exit(1);
    }

    compyrs::path_prefix_completion(&args[1]);
    compyrs::path_fuzzy_completion(&args[1]);
}
