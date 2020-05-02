#[macro_use]
extern crate clap;
extern crate glob;

use std::process;

use compyrs;

/**
 * compgen [-abcdefgjksuv] [-o option] [-A action] [-G globpat] [-W wordlist]
 * [-P prefix] [-S suffix] [-X filterpat] [-F function] [-C command] [word]
 */

// TODO use a real arg parsing crate to match compgen's features
pub fn main() {
    let matches = clap_app!(compy =>
        (version: "1.0")
        (author: "Aneesh Durg <aneeshdurg17@gmail.com>")
        (about: "Shell agnostic command completion")
        (@arg command: -c --command "Search $PATH for completions")
        (@arg prefix: -P --prefix
            +takes_value "Add prefix to results")
        (@arg suffix: -S --suffix
            +takes_value "Add prefix to results")
        (@arg filter: -X --filter
            +takes_value "Exclude results matching the supplied filter")
        (@arg INPUT: "input to complete")
    ).get_matches();

    let mut filter: Option<glob::Pattern> = None;
    let mut keep_filter = false;
    if matches.is_present("filter") {
        let filter_pattern = matches.value_of("filter").unwrap();
        let mut filter_str = filter_pattern;
        if filter_pattern.chars().next() == Some('!') {
            keep_filter = true;
            filter_str = &filter_pattern[1..];
        }

        if let Ok(pattern) = glob::Pattern::new(filter_str) {
            filter = Some(pattern);
        } else {
            eprintln!("Invalid glob: '{}'", filter_str);
            process::exit(1);
        }
    }

    let input = matches.value_of("INPUT").unwrap_or("");
    if matches.is_present("command") {
        for p in compyrs::PathPrefixCompletion::new(input.to_string()) {
            if let Some(f) = filter.as_ref() {
                if keep_filter == f.matches(&p) {
                    println!("{}", p);
                }
            } else {
                println!("{}", p);
            }
        }
    }

    // TODO:
    // tests
    // use glob::glob for -G
    // add fuzzy completion mode
    // rest of compgen flags
}
