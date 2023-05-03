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
    //main function that might fail
    color_eyre::install().expect("unable to install color-eyre"); //install error-handling/backtraces framework

    let mut env_args = env::args().skip(1); //get the environment args, except for the first one (that is the program name)

    let target_remove_extensions = [vec![OsStr::new("aux")], vec![OsStr::new("log")], vec![OsStr::new("gz"), OsStr::new("synctex")]]; //list of extensions to delete - first el is extension, further are what the name must also contain
    let target_compile_extensions = [OsStr::new("tex")]; //list of extensions to compile

    let path = env_args
        .next()
        .map_or_else(current_dir, |p| Ok(PathBuf::from(p)))?; //get the path, and if we get it, turn it into a Path, if not use the current directory
    let update_before = env_args
        .next()
        .map_or(false, |s| s.trim().eq_ignore_ascii_case("u")); //get whether or not we need to update, if nothing assume we don't

    if update_before {
        for entry in WalkDir::new(path.clone())
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |e| target_compile_extensions.contains(&e))
            })
        {
            let path = entry.path(); //get path
                                     //if we update before and the extension is in the to-compile list
            println!("Compiling {path:?}");

            let parent = path.parent().ok_or(eyre!("unable to get path parent"))?; //get the parent directory
            let status = Command::new("pdflatex") //run pdflatex
                .current_dir(parent) //in the parent directory
                .arg("-interaction=nonstopmode") //no interactions
                .arg(path) //with the path we're in
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
        .filter(|path| {
            let stringed = path.to_str().unwrap_or_default();
            if let Some(ext) = path.extension() {
                let mut res = false;
                'extensions: for mut list in target_remove_extensions.clone().into_iter()  {
                    let req_ext: &OsStr = list.remove(0);
                    if req_ext == ext {
                        let mut works = true;

                        'contents: for content in list.into_iter().filter_map(|x| x.to_str()) {
                            if !stringed.contains(content) {
                                works = false;
                                break 'contents;
                            }
                        }

                        if works {
                            res = true;
                            break 'extensions;
                        }
                    }
                }
                res
            } else {
                false
            }
        })
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
