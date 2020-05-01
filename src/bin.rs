#[macro_use]
extern crate clap;

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

    let input = matches.value_of("INPUT").unwrap_or("");
    if matches.is_present("command") {
        // turn this into an iterator
        compyrs::path_prefix_completion(&input);
    }

    // use glob::Pattern for -X and glob::glob for -G

    // compyrs::path_fuzzy_completion(&input);
}
