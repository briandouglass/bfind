#[macro_use]
extern crate clap; // Command Line Argument Parser

use clap::{Arg, App};
use std::env;
// use std::path::Path;
use std::path::PathBuf;
// use std::cmp::Ordering;

fn tgt_match(tgt: &str)

fn get_dir_contents<'a>(dir: &'a PathBuf, tgt: &str) {
    println!("fn: The directory contents are: ");
    for entry in dir.read_dir().expect("read_dir call failed!") {
        if let Ok(entry) = entry {
            // Need to add 'entry' to the queue here
            // Check files against the target

            // Check for directories
            if entry.path().is_dir() {
                println!("DIR: {:?}", entry.path().file_name().unwrap() );
            }
            // Check if any files match the target
            else if entry.path().is_file() {
                if entry.path().file_name().unwrap() == tgt {
                    println!( "Found: {}", entry.path().display() );
                }
            }
            // Just to handle any errors
            else {
                println!("{:?} -> UNK", entry.path().file_name().unwrap());
            }
        }
    }
}



fn main() {

    // Get the current directory
    let p: PathBuf = env::current_dir().unwrap();
    // println!("Current directory: {}", p.display());

    // The Clap crate helps with input arguments and --help output
    let in_args = App::new("bfind")
                    .version(crate_version!())
                    .author("Brian Douglass <brian.douglass@colorado.edu>")
                    .about("A multithreaded implementation of the unix find command written in Rust.")
                    .args(&[
                        Arg::with_name("start_dir")
                            .help("The directory where the search will begin.")
                            .index(1)
                            .required(true),
                        Arg::with_name("target")
                            .help("Name of the target file.")
                            .index(2)
                            .required(true)
                        ])
                    .get_matches();

    // Pull the target name from the command line arguments
    let tgt = in_args.value_of("target").unwrap();
    println!("TARGET: {}", tgt);
    let st_dir = in_args.value_of("start_dir").unwrap();
    let mut srch_path: PathBuf = PathBuf::new().join(p);//.canonicalize();
    srch_path.push(st_dir);
    srch_path = srch_path.canonicalize().unwrap();
    println!("START DIRECTORY: {}", srch_path.display());

    get_dir_contents(&srch_path, &tgt);






}



// if entry.path().file_name() == tgt {
//     println!("Found: {}", entry.path().file_name().display());
// }
