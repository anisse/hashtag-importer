mod types;

use std::collections::HashSet;
use std::env;
use std::io;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::types::*;

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
    let resp: ApplicationResponse = client()?
        .post(url.join("api/v1/apps")?)
        .json(&ApplicationRegistration {
            client_name: CLIENT_NAME,
            redirect_uris: OOB_URI,
            website: CLIENT_WEBSITE,
            scopes: Scope::Read,
        })
        .send()
        .with_context(|| "create app post failed")?
        .json()
        .with_context(|| "create app response body not valid json")?;
    dbg!(&resp);
    println!("Copy paste this into your config.toml:");
    println!("[auth]");
    println!("client_id = '{}'", resp.client_id.unwrap());
    println!("client_secret = '{}'", resp.client_secret.unwrap());
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
    let source_servers = [
        "mastodon.social",
        "fosstodon.org",
        "hachyderm.io",
        "chaos.social",
        "ioc.exchange",
    ];
    for server in source_servers.iter() {
        //TODO: dedup multi-source statuses
        let remote_statuses: HashSet<Status> =
            HashSet::from_iter(hashtags(server, "", 12)?.into_iter());
        let local_statuses: HashSet<Status> =
            HashSet::from_iter(hashtags(&config.server, &config.auth.token, 40)?.into_iter());
        for status in remote_statuses.difference(&local_statuses) {
            println!("Importing {} from {server}", status.url);
            //TODO: check for importing errors
            import(&config.server, &config.auth.token, &status.url)?;
            sleep(Duration::from_secs(5));
        }
    }
    Ok(())
}

fn token<S: AsRef<str>>(server: S, client_id: S, client_secret: S, code: S) -> Result<String> {
    let response = client()?
        .post(format!("https://{}/oauth/token", server.as_ref()))
        .json(&TokenQuery {
            redirect_uri: OOB_URI,
            grant_type: GrantType::AuthorizationCode,
            code: Some(code.as_ref()),
            client_id: client_id.as_ref(),
            client_secret: client_secret.as_ref(),
            scope: Some(Scope::Read),
        })
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

fn hashtags(server: &str, token: &str, limit: u8) -> Result<Vec<Status>> {
    let response: Vec<Status> = client()?
        .get(format!(
            //"https://{server}/api/v2/search?q=https%3A%2F%2Fmastodon.social%2F%40jfstudiospaleoart%40sauropods.win%2F111112209821147036&resolve=true&limit=11&type=statuses"
            "https://{server}/api/v1/timelines/tag/KernelRecipes?any[]=kr2023&any[]=KernelRecipes2023&limit={limit}"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .with_context(|| "hashtags get failed")?
        .json()
        .with_context(|| "hash tag staencodetuses body not valid json")?;
    Ok(response)
}

fn import(server: &str, token: &str, url: &str) -> Result<()> {
    let _response = client()?
        .get(
            reqwest::Url::parse_with_params(
                &format!("https://{server}/api/v2/search"),
                &[
                    ("q", url),
                    ("resolve", "true"),
                    ("limit", "25"),
                    ("type", "statuses"),
                ],
            )
            .with_context(|| format!("import search url for {url}"))?
            .as_str(),
        )
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .with_context(|| "import get failed")?;
    Ok(())
}
