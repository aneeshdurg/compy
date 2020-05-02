use std::env;
use std::fs::{read_dir, ReadDir};
use is_executable::IsExecutable;

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
    if path.is_err() {
        return None;
    }

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
