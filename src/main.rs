#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use crate::file_matcher::FileMatcher;
use clap::Parser;
use color_eyre::eyre::eyre;
use std::{
    fs::remove_file,
    path::PathBuf,
    process::{Command, Stdio},
};
use walkdir::WalkDir;

mod file_matcher;

#[derive(Debug, Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub path: PathBuf,
    #[arg(short, long)]
    pub update: bool,
}

fn main() -> color_eyre::Result<()> {
    //main function that might fail
    color_eyre::install().expect("unable to install color-eyre"); //install error-handling/backtraces framework

    let target_remove_extensions = FileMatcher::try_from(
        [
            ["aux"].as_slice(),
            ["log"].as_slice(),
            ["gz", "synctex"].as_slice(),
        ]
        .as_slice(),
    )?;
    let target_compile_extensions = FileMatcher::try_from([["tex"].as_slice()].as_slice())?;

    let Args { path, update } = Args::parse();

    // we need to check the update files first as they may add new files that we need to delete.
    if update {
        for path in WalkDir::new(path.clone())
            .into_iter()
            .filter_map(Result::ok)
            .map(|e| e.path().to_path_buf())
            .filter(|x| target_compile_extensions.matches(&path).unwrap_or(false))
        {
            println!("Compiling {path:?}");

            let parent = path.parent().ok_or(eyre!("unable to get path parent"))?; //get the parent directory
            let status = Command::new("pdflatex") //run pdflatex
                .current_dir(parent) //in the parent directory
                .arg("-interaction=nonstopmode") //no interactions
                .arg(path.clone()) //with the path we're in
                .stdout(Stdio::null()) //without stdout, but with stderr
                .status()?; //get the status

            if status.success() {
                //if we did it right
                println!("Successfully compiled: {path:?}"); //celebrate
            } else {
                eprintln!(
                    //else, fail
                    "Failed to compile {path:?} - stopping future compilations: {status:?}."
                );
            }
        }
    }

    for path in WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .filter(|path| target_remove_extensions.matches(path).unwrap_or(false))
    {
        //for every path we need to delete
        match remove_file(path.clone()) {
            //try to remove if
            Ok(_) => println!("Successfully removed {path:?}"), //if it worked, celebrate
            Err(e) => eprintln!("Error removing {path:?}: {e:?}"), //if not, then print error msg
        }
    }

    Ok(())
}
