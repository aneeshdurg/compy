use std::env;
use std::fs::read_dir;
use std::path::Path;
use is_executable::IsExecutable;

// TODO write tests
pub fn path_prefix_completion(prefix: &str) {
    let path = env::var("PATH");
    if path.is_err() {
        return;
    }

    let path = path.unwrap();
    for p in path.split(":") {
        if !Path::new(p).exists() {
            continue;
        }

        if let Ok(dir) = read_dir(&p) {
            for entry in dir {
                if entry.is_err() {
                    continue;
                }

                let entry = entry.unwrap();
                if !entry.path().is_executable() {
                    continue
                }

                let name = entry.file_name().into_string().unwrap();
                if name.starts_with(prefix) {
                    println!("{}", name);
                }
            }
        }
    }
}

pub fn path_fuzzy_completion(_: &str) {
    panic!("Unimplemented!");
}
