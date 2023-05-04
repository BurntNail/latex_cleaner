#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use crate::file_matcher::FileMatcher;
use clap::Parser;
use owo_colors::OwoColorize;
use std::{
    fs::remove_file,
    path::PathBuf,
    process::{Command, Output}, sync::mpsc::channel,
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

fn main() {
    let target_remove_extensions = match FileMatcher::try_from(
        [
            ["aux"].as_slice(),
            ["log"].as_slice(),
            ["gz", "synctex"].as_slice(),
        ]
        .as_slice(),
    ) {
        Ok(tre) => tre,
        Err(e) => {
            eprintln!("{}: {e:?}", "Error setting up FileMatcher to correctly target files".red());
            std::process::exit(1);
        }
    };
    let target_compile_extensions = match FileMatcher::try_from([["tex"].as_slice()].as_slice()) {
        Ok(tre) => tre,
        Err(e) => {
            eprintln!("{}: {e:?}", "Error setting up FileMatcher to correctly target files".red());
            std::process::exit(1);
        }
    };

    let Args { path, update } = Args::parse();

    // we need to check the update files first as they may add new files that we need to delete.
    if update {
        let (fail_tx, fail_rx) = channel();
        let mut failed = vec![];

        println!("{}", "Beginning Compilation.".bold().bright_white());

        ctrlc::set_handler(move || {
            let mut failed = fail_rx.try_iter().collect::<Vec<_>>();
            match failed.len() {
                0 => println!("{}", "No files failed!".green()),
                1 => println!("{}: {:?}.", "This file failed".red(), failed.remove(0)),
                n => {
                    print!("{}: ", "These files failed".red());
                    for _ in 0..(n - 1) {
                        print!("{:?}, ", failed.remove(0));
                    }
                    println!("{:?}.", failed.remove(0));
                }
            }
            std::process::exit(1);
        }).unwrap_or_else(|err| eprintln!("{}: {err:?}", "Error setting CTRLC handler".red()));

        for path in WalkDir::new(path.clone())
            .into_iter()
            .filter_map(Result::ok)
            .map(|e| e.path().to_path_buf())
            .filter(|path| target_compile_extensions.matches(path).unwrap_or(false))
        {
            println!("{} {path:?}", "Compiling".white());

            let Some(parent) = path.parent() else {
                eprintln!("{}", "Error getting path parent".red());
                continue;
            };

            match Command::new("pdflatex") //run pdflatex
                .current_dir(parent) //in the parent directory
                .arg("-interaction=nonstopmode") //no interactions
                .arg(path.clone()) //with the path we're in
                // .stdout(Stdio::null())
                // .stderr(Stdio::null())
                .output()
            {
                Err(e) => eprintln!("{}: {e:?}", "Error with running pdflatex".red()),
                Ok(Output {stdout, stderr, status}) => {
                    if status.success() {
                        //if we did it right
                        println!("{}", "Successfully compiled!".green()); //celebrate
                    } else {
                        //else, fail
                        eprintln!("{}", String::from_utf8_lossy(&stdout));
                        eprintln!("{}", String::from_utf8_lossy(&stderr));

                        eprintln!("{}", "Failed to compile.".red(),);

                        failed.push(path.clone());
                        fail_tx.send(path).unwrap_or_else(|err| eprintln!("{}: {err:?}", "Error sending file to CTRLC handler".red()));
                    }
                }
            }
        }

        match failed.len() {
            0 => println!("{}", "No files failed!".green()),
            1 => println!("{}: {:?}.", "This file failed".red(), failed.remove(0)),
            n => {
                print!("{}: ", "These files failed".red());
                for _ in 0..(n - 1) {
                    print!("{:?}, ", failed.remove(0));
                }
                println!("{:?}.", failed.remove(0));
            }
        }

        ctrlc::set_handler(|| std::process::exit(1)).unwrap_or_else(|err| eprintln!("{}: {err:?}", "Error setting CTRLC handler".red()));

        println!();
    }


    println!("{}", "Beginning Deletion.".bold().bright_white());

    for path in WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .filter(|path| target_remove_extensions.matches(path).unwrap_or(false))
    {
        //for every path we need to delete
        match remove_file(path.clone()) {
            //try to remove if
            Ok(_) => println!("{} {path:?}.", "Successfully removed".green()), //if it worked, celebrate
            Err(e) => eprintln!("{} {path:?}: {e:?}", "Error removing".red()), //if not, then print error msg
        }
    }
}
