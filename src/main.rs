use std::collections::HashMap;
use std::env;
use std::io;
use std::io::Write;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

const USER_AGENT: &str = concat!("hashtag-importer v", env!("CARGO_PKG_VERSION"));
const CLIENT_NAME: &str = "hashtag-importer test version";
const CLIENT_WEBSITE: &str = "https://github.com/anisse/hashtag-importer?soon";

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
fn client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent(USER_AGENT)
        .cookie_store(true)
        .build()
        .with_context(|| "cannot build custom client")
}

fn create_app() -> Result<()> {
    print!("Enter your mastodon server api domain name: ");
    io::stdout().flush().with_context(|| "flush")?;
    let mut server_domain = String::new();
    io::stdin()
        .read_line(&mut server_domain)
        .with_context(|| "unable to read stdin")?;
    let url = reqwest::Url::parse(format!("https://{server_domain}/").as_str())
        .with_context(|| format!("{server_domain} is not a domain"))?;
    // Register the app
    // TODO: struct + deserialize instead
    let resp: HashMap<String, String> = client()?
        .post(url.join("api/v1/apps")?)
        .json(&HashMap::from([
            // TODO: struct + serialize instead
            ("client_name", CLIENT_NAME),
            ("redirect_uris", "urn:ietf:wg:oauth:2.0:oob"),
            ("website", CLIENT_WEBSITE),
            ("scopes", "read"),
        ]))
        .send()
        .with_context(|| "create app post failed")?
        .json()
        .with_context(|| "create app body not valid json")?;
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

fn user_auth() -> Result<()> {
    let config: Config = toml::from_str(
        &std::fs::read_to_string("config.toml").with_context(|| "cannot read config.toml")?,
    )
    .with_context(|| "invalid config")?;
    webbrowser::open(&format!(
        "https://{}/oauth/authorize?response_type=code&client_id={}&redirect_uri=urn:ietf:wg:oauth:2.0:oob&scope=read",
        config.server, config.auth.client_id,
    ))
    .with_context(|| "cannot show auth in browser")?;
    println!("Paste the code your server gave you:");
    let mut code = String::new();
    io::stdin()
        .read_line(&mut code)
        .with_context(|| "unable to read stdin")?;
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

fn run() -> Result<()> {
    /*
    let body = reqwest::blocking::get("https://mastodon.social/api/v1/timelines/tag/fakeshakespearefacts?max_id=111111639034423756")
        .map_err(|e| format!("get failed: {e}"))?
        .text()
        .map_err(|e| format!("body not valid text: {e}"))?;
    println!("{body}");
    */
    let config: Config = toml::from_str(
        &std::fs::read_to_string("config.toml").with_context(|| "cannot read config.toml")?,
    )
    .with_context(|| "invalid config")?;
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
fn token<S: AsRef<str>>(server: S, client_id: S, client_secret: S, code: S) -> Result<String> {
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
        .with_context(|| "token post failed")?;
    let status_err = response.error_for_status_ref();
    if let Err(e) = status_err {
        bail!(
            "Got response {}: {e}",
            response.text().with_context(|| "body not valid text")?
        );
    }
    let token: Token = response
        .json()
        .with_context(|| "token body not valid json")?;
    Ok(token.access_token)
}

fn hashtags(server: &str, token: &str) -> Result<()> {
    let response = client()?
        .get(format!(
            //"https://{server}/api/v2/search?q=https%3A%2F%2Fmastodon.social%2F%40jfstudiospaleoart%40sauropods.win%2F111112209821147036&resolve=true&limit=11&type=statuses"
            "https://{server}/api/v1/timelines/tag/TearsOfTheKingdom"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .with_context(|| "hashtags get failed")?
        .text();
    dbg!(response.unwrap());
    Ok(())
}
