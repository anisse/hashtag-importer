mod config;
mod hashtag_importer;
mod types;

use std::env;

fn main() {
    // We'll add an argument parser later
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} run|associate", args[0])
    }
    match args[1].as_str() {
        "create-app" => hashtag_importer::create_app().unwrap(),
        "user-auth" => hashtag_importer::user_auth().unwrap(),
        "run" => hashtag_importer::run().unwrap(),
        _ => panic!("Unsupported action"),
    }
}
