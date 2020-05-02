extern crate pgs_files;

use std::env;
use std::fs::{read_dir, ReadDir};
use is_executable::IsExecutable;
use pgs_files::group;

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
    if path.is_err() { return None; } 
    let path = path.unwrap();
    Some(PathDirIterator { paths: path, idx: 0 })
}

pub struct PathIterator {
    paths: PathDirIterator,
    dir: Option<ReadDir>,
}

impl PathIterator {
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

pub fn path_iterator() -> Option<PathIterator>{
    let paths = path_dir_iterator();
    match paths {
        None => None,
        Some(p) => Some(PathIterator {
            paths: p,
            dir: None,
        })
    }
}

impl Iterator for PathIterator {
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

            let entry = entry.unwrap();
            if entry.is_err() {
                continue;
            }

            let entry = entry.unwrap();
            if !entry.path().is_executable() {
                continue;
            }

            return Some(entry.file_name().into_string().unwrap());
        }
    }
}

pub struct PathPrefixCompletion {
    exes: PathIterator,
    prefix: String,
}

impl Iterator for PathPrefixCompletion {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            if let Some(name) = self.exes.next()  {
                if name.starts_with(&self.prefix) {
                    return Some(name);
                }
                continue;
            }

            return None;
        }
    }
}

impl PathPrefixCompletion {
    pub fn new(prefix: String) -> PathPrefixCompletion {
        PathPrefixCompletion {
            exes: path_iterator().unwrap(),
            prefix: prefix,
        }
    }
}

pub struct DirPrefixCompletion {
    dir: ReadDir,
    prefix: String,
    search_files: bool,
    search_dirs: bool,
}

impl Iterator for DirPrefixCompletion {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            if let Some(entry) = self.dir.next()  {
                if !entry.is_err() {
                    let entry = entry.unwrap();
                    let is_dir = {
                        let mut is_dir = false;
                        if let Ok(metadata) = entry.metadata() {
                            is_dir = metadata.is_dir()
                        }

                        is_dir
                    };
                    if (self.search_dirs && is_dir) ||
                            (self.search_files && !is_dir) {
                        let name =
                            entry.file_name().into_string().unwrap();
                        if name.starts_with(&self.prefix) {
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

impl DirPrefixCompletion {
    pub fn new(
        prefix: String, search_files: bool, search_dirs: bool
    ) -> Option<DirPrefixCompletion> {
        if let Ok(dir) = read_dir(".") {
            Some(DirPrefixCompletion {
                dir,
                prefix,
                search_files,
                search_dirs,
            })
        } else {
            None
        }
    }
}

pub struct EnvPrefixCompletion {
    vars: env::Vars,
    prefix: String,
}

impl Iterator for EnvPrefixCompletion {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            if let Some(v) = self.vars.next()  {
                if v.0.starts_with(&self.prefix) {
                    return Some(v.0);
                }

                continue;
            }

            return None;
        }
    }
}

impl EnvPrefixCompletion {
    pub fn new(prefix: String) -> EnvPrefixCompletion {
        EnvPrefixCompletion {
            vars: env::vars(),
            prefix,
        }
    }
}

pub struct GroupPrefixCompletion {
    groups: Vec<group::GroupEntry>,
    idx: usize,
    prefix: String,
}

impl GroupPrefixCompletion {
    pub fn new(prefix: String) -> GroupPrefixCompletion {
        GroupPrefixCompletion {
            groups: group::get_all_entries(),
            idx: 0,
            prefix,
        }
    }
}

impl Iterator for GroupPrefixCompletion {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            if self.idx >= self.groups.len() {
                return None;
            }

            let group = &self.groups[self.idx].name;
            self.idx += 1;

            if group.starts_with(&self.prefix) {
                return Some(group.to_string());
            }
        }
    }
}
