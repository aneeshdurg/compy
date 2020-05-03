extern crate pgs_files;
extern crate users;

use std::collections::HashSet;
use std::env;
use std::fs::{read_dir, ReadDir};
use hostfile::parse_hostfile;
use is_executable::IsExecutable;
use pgs_files::group;
use users::{all_users, User};
use servicefile::parse_servicefile;

pub struct FilterParams<'a> {
    pub filter: Option<glob::Pattern>,
    pub keep_filter: bool,
    pub input: &'a str,
    pub prepend: &'a str,
    pub append: &'a str,
}

pub fn filter_and_display(
        completions: impl Iterator<Item=String>, params: &FilterParams) {
    for entry in completions {
        if !entry.starts_with(params.input) {
            continue;
        }

        let mut keep_entry = true;
        if let Some(f) = params.filter.as_ref() {
            keep_entry = params.keep_filter == f.matches(&entry);
        }

        if keep_entry {
            println!("{}{}{}", params.prepend, entry, params.append);
        }
    }
}

pub struct PathDirIterator {
    paths: String,
    idx: usize,
}

impl Iterator for PathDirIterator {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let startidx = self.idx;
        match &self.paths.as_str()[startidx..].find(':') {
            None => None,
            Some(idx) => {
                let endidx = idx + startidx;
                self.idx = endidx + 1;
                Some(String::from(&self.paths[startidx..endidx]))
            }
        }
    }
}

fn path_dir_iterator() -> Option<PathDirIterator>{
    let path = env::var("PATH");
    if let Ok(path) = path {
        return Some(PathDirIterator { paths: path, idx: 0 });
    }

    None
}

pub struct PathCompletion {
    paths: PathDirIterator,
    dir: Option<ReadDir>,
}

impl PathCompletion {
    pub fn new() -> Option<PathCompletion> {
        match path_dir_iterator() {
            Some(paths) => Some(PathCompletion { paths, dir: None }),
            None => None,
        }
    }

    fn advance_path(&mut self) {
        loop {
            let path = self.paths.next();
            match path {
                None => {
                    self.dir = None;
                    break;
                }
                Some(p) => {
                    if let Ok(dir) = read_dir(&p) {
                        self.dir = Some(dir);
                        break;
                    }

                    // couldn't open dir, try the next path
                }
            }
        }
    }
}

impl Iterator for PathCompletion {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if let None = self.dir {
            self.advance_path();
        }

        loop {
            let entry = self.dir.as_mut().unwrap().next();
            if let None = entry {
                self.advance_path();
                if let None = self.dir {
                    return None;
                }
                continue;
            }

            let entry = match entry {
                Some(Ok(entry)) => entry,
                _ => continue,
            };

            if !entry.path().is_executable() {
                continue;
            }

            return Some(entry.file_name().into_string().unwrap());
        }
    }
}

pub struct DirCompletion {
    dir: ReadDir,
    search_files: bool,
    search_dirs: bool,
}

impl Iterator for DirCompletion {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            if let Some(entry) = self.dir.next()  {
                match entry {
                    Err(_) => {},
                    Ok(entry) => {
                        let is_dir = match entry.metadata() {
                            Ok(metadata) =>  metadata.is_dir(),
                            _ => false,
                        };

                        if (self.search_dirs && is_dir) ||
                            (self.search_files && !is_dir) {
                            let name =
                                entry.file_name().into_string().unwrap();
                            return Some(name);
                        }
                    }
                }

                continue;
            }

            return None;
        }
    }
}

impl DirCompletion {
    pub fn new(search_files: bool, search_dirs: bool) -> Option<DirCompletion> {
        if let Ok(dir) = read_dir(".") {
            Some(DirCompletion {
                dir,
                search_files,
                search_dirs,
            })
        } else {
            None
        }
    }
}

pub struct EnvCompletion {
    vars: env::Vars,
}

impl Iterator for EnvCompletion {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.vars.next().map(|v| v.0)
    }
}

impl EnvCompletion {
    pub fn new() -> EnvCompletion {
        EnvCompletion {
            vars: env::vars(),
        }
    }
}

pub trait Stringify {
    fn get_string(&self) -> String;
}

pub struct VecCompletion<T: Stringify> {
    elements: Vec<T>,
    idx: usize,
}

impl<T: Stringify> Iterator for VecCompletion<T> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            if self.idx >= self.elements.len() {
                return None;
            }

            let element = &self.elements[self.idx].get_string();
            self.idx += 1;

            return Some(element.to_string());
        }
    }
}

impl Stringify for group::GroupEntry {
    fn get_string(&self) -> String {
        self.name.to_string()
    }
}

pub type GroupCompletion = VecCompletion<group::GroupEntry>;

impl GroupCompletion {
    pub fn new() -> GroupCompletion {
        GroupCompletion {
            elements: group::get_all_entries(),
            idx: 0,
        }
    }
}

impl Stringify for String {
    fn get_string(&self) -> String {
        return self.to_string();
    }
}

pub type HostCompletion = VecCompletion<String>;

impl HostCompletion {
    pub fn new() -> HostCompletion {
        let mut hosts = HashSet::new();

        let host_entries = parse_hostfile().unwrap();
        for host_entry in host_entries {
            for name in host_entry.names {
                hosts.insert(name);
            }
        }

        let mut elements = Vec::new();
        for host in hosts {
            elements.push(host.to_string());
        }

        HostCompletion { elements, idx: 0 }
    }
}

pub struct ServiceCompletion{
    _inner: VecCompletion<String>,
}

impl ServiceCompletion {
    pub fn new() -> ServiceCompletion {
        let mut services = HashSet::new();

        let service_entries = parse_servicefile(true).unwrap();
        for service_entry in service_entries {
            services.insert(service_entry.name);
            for alias in service_entry.aliases {
                services.insert(alias);
            }
        }

        let mut elements = Vec::new();
        for service in services {
            elements.push(service.to_string());
        }

        ServiceCompletion {
            _inner: VecCompletion::<String> { elements, idx: 0 }
        }
    }
}

impl Iterator for ServiceCompletion {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self._inner.next()
    }
}

impl Stringify for User {
    fn get_string(&self) -> String {
        self.name().to_str().unwrap().to_string()
    }
}

pub type UserCompletion = VecCompletion<User>;

impl UserCompletion {
    pub fn new() -> UserCompletion {
        let elements = unsafe { all_users() }.collect();
        UserCompletion { elements, idx: 0 }
    }
}

pub struct WordListCompletion {
    _inner: VecCompletion<String>,
}

impl WordListCompletion {
    pub fn new(wordlist: &str) -> WordListCompletion {
        let elements =
            wordlist.split_whitespace().map(|s| s.to_string()).collect();
        WordListCompletion {
            _inner: VecCompletion::<String> { elements, idx: 0 }
        }
    }
}

impl Iterator for WordListCompletion {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self._inner.next()
    }
}
