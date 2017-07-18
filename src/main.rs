/*
 * bfind
 * Written by Brian Douglass
 * for ECEN 4313 - Concurrent Programming
 * CU Boulder, Spring 2017
 * Prof. Cerny
 */

extern crate crossbeam;
extern crate clap;


use std::env;
use clap::{App, Arg};
use std::thread;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use crossbeam::sync::MsQueue;

// These enums are for message passing
enum PathOrWorkState {
    // check_dir to main
    Path(PathBuf), // directory to enqueue
    StartWork,
    DoneWork,
}

enum PathOrExit {
    // main to check_dir
    NmlPath(PathBuf), // directory to search
    RecPath(PathBuf),
    Exit, // Exit the thread
}

// check_dir takes a PathBuf to search, a target to compare against, and a Sender channel to
// send sub-directories back to main with
fn q_check<'a>(directory: PathBuf, target: &'a str, tx: &'a Sender<PathOrWorkState>) {

    let tgt = OsStr::new(target);
    let mut dir = directory;

    loop {
        let mut dircount = 0;
        let mut nextdirchosen = false;

        // DEBUGGING: print the current directory that is being searched
        // println!("{}", dir.display());

        // Check each item in the directory
        for item in dir.read_dir().expect("read_dir failed") {
            if let Ok(item) = item {

                // For each sub-directory encountered,
                if item.path().is_dir() {
                    dircount += 1;

                    // Check to see if the directory matches the target
                    if item.path().file_stem().unwrap() == tgt {
                        println!("FOUND: {}", item.path().display());
                    }

                    // If it is the first one, set it as the next one
                    if nextdirchosen == false {
                        dir = item.path();
                        nextdirchosen = true;
                    } else if tx.send(PathOrWorkState::Path(item.path())).is_err() {
                        panic!("nml send error");
                        // Otherwise send it back to the main thread
                        //tx.send(PathOrWorkState::Path(item.path())).unwrap();
                    }
                }
                // Check each file encountered against the tgt
                else if item.path().is_file() {
                    if item.path().file_name().unwrap() == tgt {
                        println!("FOUND: {}", item.path().display());
                    }
                }
            }
        }
        // If a 'leaf' directory is found break out of the loop
        if dircount == 0 {
            break;
        }
    }
}

fn rec_check(directory: PathBuf, target: &str) {

    let tgt = OsStr::new(target);

    for item in directory.read_dir().expect("read_dir failed") {
        if let Ok(item) = item {
            if item.path().is_dir() {
                rec_check(item.path(), &target);
            } else if item.path().is_file() {
                if item.path().file_name().unwrap() == tgt {
                    println!("FOUND: {}", item.path().display());
                }
            }
        }
    }
}

// fn get_input() {
//     //The Clap crate helps with input arguments and --help output
//     let in_args = App::new("bfind")
//         .version(crate_version!())
//         .author("Brian Douglass <brian.douglass@mac.com>")
//         .about("A multithreaded implementation of the unix find command written in Rust.")
//         .args(&[Arg::with_name("start_dir")
//                     .help("The directory where the search will begin.")
//                     .index(1)
//                     .required(true),
//                 Arg::with_name("target")
//                     .help("Name of the target file.")
//                     .index(2)
//                     .required(true)])
//         .get_matches();
//
//     // Pull the target name from the command line arguments
//     //let target = in_args.value_of("target").unwrap(); // println!("TARGET: {}", tgt);
//     let target;
//     let args = in_args.clone();
//
//     if let Some(x) = args.value_of("target") {
//         x.clone();
//         target = x.as_ref();
//         //target.trim();
//     }
//
//
//     // Set up the starting directory
//     st_dir = args.value_of("start_dir").unwrap();
//     // if let Some(x) = args.value_of("start_dir") {
//     //     //x.clone();
//     //     st_dir = x;
//     //}
//
//
//     if !st_dir.is_empty() {
//         println!("No starting directory was given. Using current directory.");
//         //start_dir = st_dir.as_path_buf();
//     } else {
//         start_dir.push(st_dir);
//         start_dir = start_dir.canonicalize().unwrap();
//     }
// }


fn main() {

    // Immutable bindings
    let (tx, rx) = channel();
    let target; // = env::args().nth(2).unwrap(); // These should be handled more robustly, i.e. without the .unwrap()
    let num_threads = 7; // = env::args().nth(3).unwrap().parse().unwrap();
    let cmd_vec: Arc<MsQueue<PathOrExit>> = Arc::new(MsQueue::new());
    let q_size = Arc::new(Mutex::new(0));

    // Mutable bindings
    let mut threadsrunning = 0;
    let mut start_dir = PathBuf::new().join(env::current_dir().unwrap());
    let mut handles = vec![];

    // Command line arguments
    let in_args = App::new("bfind")
        .author("Brian Douglass <brian.douglass@mac.com>")
        .about("A multithreaded implementation of the unix find command written in Rust.")
        .args(&[Arg::with_name("start_dir")
                    .help("The directory where the search will begin.")
                    .index(1)
                    .required(true),
                Arg::with_name("target")
                    .help("Name of the target file.")
                    .index(2)
                    .required(true)])
        .get_matches();

    target = in_args.value_of("target");
    let stdir = in_args.value_of("start_dir");



    start_dir.push(Path::new(stdir.unwrap()).to_path_buf());
    start_dir = start_dir.canonicalize().unwrap();

    // DEBUGGING: print the information passed in
    // println!("TARGET: {}", target);
    // println!("STARTDIR: {}", start_dir.display());

    // Push starting directory to the queue
    cmd_vec.push(PathOrExit::NmlPath(start_dir));
    {
        let mut counter = q_size.lock().unwrap();
        *counter += 1;
    }
    // Create the thread_pool here
    for _ in 0..num_threads {

        // These bindings must be cloned so ownership can be transferred to the thread
        let tx = tx.clone();
        let cmd_vec = cmd_vec.clone();
        let target = target.clone();
        let q_size = q_size.clone();

        // Spawn the threads
        let handle = thread::spawn(move || 'thd: loop {
            match cmd_vec.pop() {
                PathOrExit::NmlPath(dir) => {
                    // println!("thread {} nml: {}", t, dir.display());
                    // Let the main thread know this function started
                    if tx.send(PathOrWorkState::StartWork).is_err() {
                        panic!("Startwork error");
                    }

                    // Check this directory
                    q_check(dir, &target.unwrap().clone(), &tx);
                    let mut counter = q_size.lock().unwrap();
                    *counter -= 1;

                    // Let the main thread know this function is completed
                    if tx.send(PathOrWorkState::DoneWork).is_err() {
                        panic!("Donework error");
                    }
                }
                PathOrExit::RecPath(dir) => {
                    // Let the main thread know this function started
                    if tx.send(PathOrWorkState::StartWork).is_err() {
                        panic!("Startwork error");
                    }

                    // Check this directory
                    rec_check(dir, &target.unwrap().clone());
                    let mut counter = q_size.lock().unwrap();
                    *counter -= 1;

                    // Let the main thread know this function is completed
                    if tx.send(PathOrWorkState::DoneWork).is_err() {
                        panic!("Donework error");
                    }
                }
                PathOrExit::Exit => break 'thd,
            }
        });
        handles.push(handle);
    }


    // Handle the sub-directories and overall program control here
    // This is effectivly the 'main' thread
    for rxer in rx {

        {
            let mut counter = q_size.lock().unwrap();

            // Handle items returned from the threads
            match rxer {
                PathOrWorkState::Path(path) => {
                    if *counter < 100 {
                        cmd_vec.push(PathOrExit::NmlPath(path));
                        *counter += 1;
                        // println!("normal {}", *counter);
                    } else {
                        cmd_vec.push(PathOrExit::RecPath(path));
                        // println!("recursive {}", *counter);
                    }
                }
                PathOrWorkState::StartWork => threadsrunning += 1,
                PathOrWorkState::DoneWork => threadsrunning -= 1,
            }
        }
        // When all of the threads have completed and the command vector is empty,
        {
            if threadsrunning == 0 && cmd_vec.is_empty() {

                // Push num_threads 'Exit' commands
                for _ in 0..num_threads {
                    cmd_vec.push(PathOrExit::Exit);
                }
                break;
            }
        }
    }

    // Join all threads before exiting
    for hand in handles {
        if hand.join().is_err() {
            panic!("Join error");
        }

    }
}
