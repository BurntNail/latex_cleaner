use std::{env, env::current_dir, ffi::OsStr, fs::remove_file, path::PathBuf};
use walkdir::WalkDir;

fn main() {
    let target_extensions = [OsStr::new("aux"), OsStr::new("log"), OsStr::new("gz")];

    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| current_dir().expect("unable to get current working directory."));
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path
            .extension()
            .map_or(false, |e| target_extensions.contains(&e))
        {
            match remove_file(path) {
                Ok(_) => println!("Successfully removed {path:?}"),
                Err(e) => eprintln!("Error removing {path:?}: {e:?}"),
            }
        }
    }
}
