#[macro_use]
extern crate clap;
extern crate glob;

use std::process;

use compyrs;

/**
 * compgen [-u] [-o option] [-W wordlist] [-F function] [-C command]
 * The action may be one of the following to generate a list of possible completions:
      signal  Signal names.
      user    User names.  May also be specified as -u.

      TODO:
      + allow every completion to return two strings for optional data
      + implement -p for searching (pids, cmds)
      + add fuzzy completion mode
 */

struct FilterParams<'a> {
    filter: Option<glob::Pattern>,
    keep_filter: bool,
    prefix: &'a str,
    suffix: &'a str,
}

fn filter_and_display(
        completions: impl Iterator<Item=String>, params: &FilterParams) {
    for entry in completions {
        let mut keep_entry = true;
        if let Some(f) = params.filter.as_ref() {
            keep_entry = params.keep_filter == f.matches(&entry);
        }

        if keep_entry {
            println!("{}{}{}", params.prefix, entry, params.suffix);
        }
    }
}

// TODO use a real arg parsing crate to match compgen's features
pub fn main() {
    let matches = clap_app!(compy =>
        (version: "1.0")
        (author: "Aneesh Durg <aneeshdurg17@gmail.com>")
        (about: "Shell agnostic command completion")
        (@arg search_commands: -c --search_commands
            "Search $PATH for completions")
        (@arg search_dirs: -d --search_dirs
            "Search current working directory for directory completions")
        (@arg search_env: -e --search_env
            "Search ENVIRONMENT for completions")
        (@arg search_files: -f --search_files
            "Search current working directory for file completions")
        (@arg search_groups: -g --search_groups
            "Search groups for completions")
        (@arg search_hosts: -h --search_hosts
            "Search hostnames in /etc/hosts for completions")
        (@arg search_services: -s --search_services
            "Search services in /etc/services for completions")
        (@arg prefix: -P --prefix
            +takes_value "Add prefix to results")
        (@arg suffix: -S --suffix
            +takes_value "Add prefix to results")
        (@arg filter: -X --filter
            +takes_value "Exclude results matching the supplied filter")
        (@arg INPUT: "input to complete")).get_matches();

    let prefix = matches.value_of("prefix").unwrap_or("");
    let suffix = matches.value_of("suffix").unwrap_or("");

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

    let filter_params = FilterParams { filter, keep_filter, prefix, suffix };

    let input = matches.value_of("INPUT").unwrap_or("");
    if matches.is_present("search_commands") {
        filter_and_display(
            compyrs::PathPrefixCompletion::new(input.to_string()),
            &filter_params);
    }

    let search_files = matches.is_present("search_files");
    let search_dirs = matches.is_present("search_dirs");
    if search_files || search_dirs {
        filter_and_display(
            compyrs::DirPrefixCompletion::new(
                input.to_string(), search_files, search_dirs).unwrap(),
            &filter_params);
    }

    if matches.is_present("search_env") {
        filter_and_display(
            compyrs::EnvPrefixCompletion::new(input.to_string()),
            &filter_params);
    }

    if matches.is_present("search_groups") {
        filter_and_display(
            compyrs::GroupPrefixCompletion::new(input.to_string()),
            &filter_params);
    }

    if matches.is_present("search_hosts") {
        filter_and_display(
            compyrs::HostPrefixCompletion::new(input.to_string()),
            &filter_params);
    }

    if matches.is_present("search_services") {
        filter_and_display(
            compyrs::ServicePrefixCompletion::new(input.to_string()),
            &filter_params);
    }

}
