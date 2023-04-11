#![warn(clippy::all, clippy::nursery, clippy::pedantic)]

use color_eyre::eyre::eyre;
use std::{
    env,
    env::current_dir,
    ffi::OsStr,
    fs::remove_file,
    path::PathBuf,
    process::{Command, Stdio},
};
use walkdir::WalkDir;

fn main() -> color_eyre::Result<()> {
    color_eyre::install().expect("unable to install color-eyre");

    let mut env_args = env::args().skip(1);

    let target_remove_extensions = [OsStr::new("aux"), OsStr::new("log"), OsStr::new("gz")];
    let target_compile_extensions = [OsStr::new("tex")];

    let path = env_args
        .next()
        .map_or_else(current_dir, |p| Ok(PathBuf::from(p)))?;
    let mut update_before = env_args
        .next()
        .map_or(false, |s| s.trim().eq_ignore_ascii_case("u"));

    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if update_before && target_compile_extensions.contains(&ext) {
                println!("Compiling {path:?}");

                let parent = path.parent().ok_or(eyre!("unable to get path parent"))?;
                let status = Command::new("pdflatex")
                    .current_dir(parent)
                    .arg("-interaction=nonstopmode")
                    .arg(path)
                    .stdout(Stdio::null())
                    .status()?;

                if status.success() {
                    println!("Successfully compiled: {path:?}");
                } else {
                    eprintln!(
                        "Failed to compile {path:?} - stopping future compilations: {status:?}."
                    );
                    update_before = false;
                }
            } else if target_remove_extensions.contains(&ext) {
                match remove_file(path) {
                    Ok(_) => println!("Successfully removed {path:?}"),
                    Err(e) => eprintln!("Error removing {path:?}: {e:?}"),
                }
            }
        }
    }

    Ok(())
}
