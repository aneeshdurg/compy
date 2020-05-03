#[macro_use]
extern crate clap;
extern crate glob;

use std::process;

use compyrs::*;

/**
* compgen
* The action may be one of the following to generate a list of possible completions:
*    TODO:
*    + allow every completion to return two strings for optional data
*    + implement -p for searching (pids, cmds)
*    + add fuzzy completion mode
*/

pub fn main() {
    let matches = clap_app!(compy =>
        (version: "1.0")
        (author: "Aneesh Durg <aneeshdurg17@gmail.com>")
        (about: "Shell agnostic command completion")
        (@arg search_commands: -c --search_commands "Search $PATH for completions")
        (@arg search_dirs: -d --search_dirs "Search current working directory for directory completions")
        (@arg search_env: -e --search_env "Search ENVIRONMENT for completions")
        (@arg search_files: -f --search_files "Search current working directory for file completions")
        (@arg search_groups: -g --search_groups "Search groups for completions")
        (@arg search_hosts: -h --search_hosts "Search hostnames in /etc/hosts for completions")
        (@arg search_services: -s --search_services "Search services in /etc/services for completions")
        (@arg search_users: -u --search_users "Search usernames for completions")
        (@arg prefix: -P --prefix +takes_value "Add prefix to results")
        (@arg suffix: -S --suffix +takes_value "Add prefix to results")
        (@arg wordlist: -W --wordlist +takes_value "A space separated list of words to use as possible completions")
        (@arg filter: -X --filter +takes_value "Exclude results matching the supplied filter")
        (@arg input: "input to complete")).get_matches();

    let prepend = matches.value_of("prefix").unwrap_or("");
    let append = matches.value_of("suffix").unwrap_or("");

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

    let input = matches.value_of("input").unwrap_or("");

    let filter_params = FilterParams {
        filter,
        keep_filter,
        input,
        prepend,
        append,
    };

    if matches.is_present("search_commands") {
        filter_and_display(PathCompletion::new().unwrap(), &filter_params);
    }

    let search_files = matches.is_present("search_files");
    let search_dirs = matches.is_present("search_dirs");
    if search_files || search_dirs {
        if let Some(completion) = DirCompletion::new(input, search_files, search_dirs) {
            filter_and_display(completion, &filter_params);
        }
    }

    if matches.is_present("search_env") {
        filter_and_display(EnvCompletion::new(), &filter_params);
    }

    if matches.is_present("search_groups") {
        filter_and_display(GroupCompletion::new(), &filter_params);
    }

    if matches.is_present("search_hosts") {
        filter_and_display(HostCompletion::new(), &filter_params);
    }

    if matches.is_present("search_users") {
        filter_and_display(UserCompletion::new(), &filter_params);
    }

    if matches.is_present("search_services") {
        filter_and_display(ServiceCompletion::new(), &filter_params);
    }

    if matches.is_present("wordlist") {
        filter_and_display(
            WordListCompletion::new(matches.value_of("wordlist").unwrap()),
            &filter_params,
        );
    }
}
