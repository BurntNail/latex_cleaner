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

fn main() -> color_eyre::Result<()> { //main function that might fail
    color_eyre::install().expect("unable to install color-eyre"); //install error-handling/backtraces framework

    let mut env_args = env::args().skip(1); //get the environment args, except for the first one (that is the program name)

    let target_remove_extensions = [OsStr::new("aux"), OsStr::new("log"), OsStr::new("gz")]; //list of extensions to delete
    let target_compile_extensions = [OsStr::new("tex")]; //list of extensions to compile

    let path = env_args
        .next()
        .map_or_else(current_dir, |p| Ok(PathBuf::from(p)))?; //get the path, and if we get it, turn it into a Path, if not use the current directory
    let mut update_before = env_args
        .next()
        .map_or(false, |s| s.trim().eq_ignore_ascii_case("u")); //get whether or not we need to update, if nothing assume we don't

    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) { //for every directory we have access to
        let path = entry.path(); //get the path
        if let Some(ext) = path.extension() { //if we have an extension - can be none if no file name, no ., begins with . and has no other .s
            if update_before && target_compile_extensions.contains(&ext) { //if we update before and the extension is in the to-compile list
                println!("Compiling {path:?}");

                let parent = path.parent().ok_or(eyre!("unable to get path parent"))?; //get the parent directory
                let status = Command::new("pdflatex") //run pdflatex
                    .current_dir(parent) //in the parent directory
                    .arg("-interaction=nonstopmode") //no interactions
                    .arg(path) //with the path we're in
                    .stdout(Stdio::null()) //without stdout, but with stderr
                    .status()?; //get the status

                if status.success() { //if we did it right
                    println!("Successfully compiled: {path:?}"); //celebrate
                } else {
                    eprintln!( //else, fail
                        "Failed to compile {path:?} - stopping future compilations: {status:?}."
                    );
                }
            } else if target_remove_extensions.contains(&ext) { //else if we need to remove it
                match remove_file(path) { //try to remove if
                    Ok(_) => println!("Successfully removed {path:?}"), //if it worked, celebrate
                    Err(e) => eprintln!("Error removing {path:?}: {e:?}"), //if not, then print error msg
                }
            }
        }
    }

    Ok(())
}
