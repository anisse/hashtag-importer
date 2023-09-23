use std::collections::HashMap;
use std::env;
use std::io;
use std::io::Write;

use serde::Deserialize;

const USER_AGENT: &str = concat!("hashtag-importer v", env!("CARGO_PKG_VERSION"));
fn main() {
    // We'll add an argument parser later
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} run|associate", args[0])
    }
    match args[1].as_str() {
        "create-app" => create_app().unwrap(),
        "user-auth" => user_auth().unwrap(),
        "run" => run().unwrap(),
        _ => panic!("Unsupported action"),
    }
}
fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent(USER_AGENT)
        .cookie_store(true)
        .build()
        .map_err(|e| format!("Cannot build custom client: {e}"))
}


fn create_app() -> Result<(), String> {
    print!("Enter your mastodon server api domain name: ");
    io::stdout().flush().map_err(|e| format!("flush: {e}"))?;
    let mut server_domain = String::new();
    io::stdin()
        .read_line(&mut server_domain)
        .map_err(|e| format!("unable to read stdin: {e}"))?;
    let url = reqwest::Url::parse(format!("https://{server_domain}/").as_str())
        .map_err(|e| format!("{server_domain} is not a domain: {e}"))?;
    // Register the app
    // TODO: struct + deserialize instead
    let resp: HashMap<String, String> = client()?
        .post(url.join("api/v1/apps").map_err(|e| format!("{e}"))?)
        .json(&HashMap::from([
            // TODO: struct + serialize instead
            ("client_name", "hashtag-importer test version"),
            ("redirect_uris", "urn:ietf:wg:oauth:2.0:oob"),
            ("website", "https://github.com/anisse/hashtag-importer?soon"),
            ("scopes", "read"),
        ]))
        .send()
        .map_err(|e| format!("post failed: {e}"))?
        .json()
        .map_err(|e| format!("body not valid json: {e}"))?;
    println!("Copy paste this into your config.toml:");
    println!("[auth]");
    println!("client_id = '{}'", resp["client_id"]);
    println!("client_secret = '{}'", resp["client_secret"]);
    Ok(())
}

#[derive(Deserialize)]
struct Config {
    auth: Auth,
    server: String,
}
#[derive(Deserialize)]
struct Auth {
    client_id: String,
    client_secret: String,
    token: String,
}

fn user_auth() -> Result<(), String> {
    let config: Config = toml::from_str(
        &std::fs::read_to_string("config.toml")
            .map_err(|e| format!("cannot read config.toml: {e}"))?,
    )
    .map_err(|e| format!("invalid config: {e}"))?;
    webbrowser::open(&format!(
        "https://{}/oauth/authorize?response_type=code&client_id={}&redirect_uri=urn:ietf:wg:oauth:2.0:oob&scope=read",
        config.server, config.auth.client_id,
    ))
    .map_err(|e| format!("cannot open browser: {e}"))?;
    println!("Paste the code your server gave you:");
    let mut code = String::new();
    io::stdin()
        .read_line(&mut code)
        .map_err(|e| format!("unable to read stdin: {e}"))?;
    let token = token(
        &config.server,
        &config.auth.client_id,
        &config.auth.client_secret,
        &code.trim().to_string(),
    )?;
    println!("Updated your config.toml auth section:");
    println!("[auth]");
    println!("token = '{token}'");
    Ok(())
}

fn run() -> Result<(), String> {
    /*
    let body = reqwest::blocking::get("https://mastodon.social/api/v1/timelines/tag/fakeshakespearefacts?max_id=111111639034423756")
        .map_err(|e| format!("get failed: {e}"))?
        .text()
        .map_err(|e| format!("body not valid text: {e}"))?;
    println!("{body}");
    */
    let config: Config = toml::from_str(
        &std::fs::read_to_string("config.toml")
            .map_err(|e| format!("cannot read config.toml: {e}"))?,
    )
    .map_err(|e| format!("invalid config: {e}"))?;
    hashtags(&config.server, &config.auth.token)
}

#[derive(Deserialize)]
struct Token {
    access_token: String,
    // unused fields
    /*
    token_type: String,
    created_at: u64,
    scope: String,
    */
}
fn token<S: AsRef<str>>(
    server: S,
    client_id: S,
    client_secret: S,
    code: S,
) -> Result<String, String> {
    let response = client()?
        .post(format!("https://{}/oauth/token", server.as_ref()))
        .json(&HashMap::from([
            // TODO: struct + serialize instead
            ("redirect_uri", "urn:ietf:wg:oauth:2.0:oob"),
            ("grant_type", "authorization_code"),
            ("code", code.as_ref()),
            ("client_id", client_id.as_ref()),
            ("client_secret", client_secret.as_ref()),
            ("scope", "read"),
        ]))
        .send()
        .map_err(|e| format!("post failed: {e}"))?;
    let status_err = response.error_for_status_ref();
    if let Err(e) = status_err {
        return Err(format!(
            "Got response {}: {e}",
            response
                .text()
                .map_err(|e| format!("body not valid text: {e}"))?
        ));
    }
    let token: Token = response
        .json()
        .map_err(|e| format!("token body not valid json: {e}"))?;
    Ok(token.access_token)
}

fn hashtags(server: &str, token: &str) -> Result<(), String> {
    let response = client()?
        .get(format!(
            //"https://{server}/api/v2/search?q=https%3A%2F%2Fmastodon.social%2F%40jfstudiospaleoart%40sauropods.win%2F111112209821147036&resolve=true&limit=11&type=statuses"
            "https://{server}/api/v1/timelines/tag/TearsOfTheKingdom"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .map_err(|e| format!("get failed: {e}"))?
        .text();
    dbg!(response.unwrap());
    Ok(())
}
