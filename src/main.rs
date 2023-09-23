use std::env;

fn main() {
    // We'll add an argument parser later
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} run|associate", args[0])
    }
    match args[1].as_str() {
        "associate" => associate().unwrap(),
        "run" => run().unwrap(),
        _ => panic!("Unsupported action"),
    }
}
fn associate() -> Result<(), String> {
    Ok(())
}
fn run() -> Result<(), String> {
    let body = reqwest::blocking::get("https://mastodon.social/api/v1/timelines/tag/fakeshakespearefacts?max_id=111111639034423756")
        .map_err(|e| format!("get failed: {e}"))?
        .text()
        .map_err(|e| format!("body not valid text: {e}"))?;
    println!("{body}");
    Ok(())
}
