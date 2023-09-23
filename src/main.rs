fn main() {
    let body = reqwest::blocking::get("https://mastodon.social/api/v1/timelines/tag/fakeshakespearefacts?max_id=111111639034423756").unwrap().text().unwrap();
    println!("{body}");
    //Ok(())
}
