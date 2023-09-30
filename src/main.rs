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
        .context("cannot build custom client")
}

fn create_app() -> Result<()> {
    print!("Enter your mastodon server api domain name: ");
    io::stdout().flush().context("flush")?;
    let mut server_domain = String::new();
    io::stdin()
        .read_line(&mut server_domain)
        .context("unable to read stdin")?;
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
        .context("create app post failed")?
        .json()
        .context("create app response body not valid json")?;
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
    hashtag: Vec<Hashtag>,
}
#[derive(Deserialize)]
struct Auth {
    client_id: String,
    client_secret: String,
    token: String,
}
#[derive(Deserialize)]
struct Hashtag {
    //Be very careful: base tag needs to exist in source servers otherwiise additionnal tags will be useless
    name: String,
    any: Option<Vec<String>>,
    sources: Vec<String>,
}

fn user_auth() -> Result<()> {
    let config: Config =
        toml::from_str(&std::fs::read_to_string("config.toml").context("cannot read config.toml")?)
            .context("invalid config")?;
    webbrowser::open(&format!(
        "https://{}/oauth/authorize?response_type=code&client_id={}&redirect_uri=urn:ietf:wg:oauth:2.0:oob&scope=read",
        config.server, config.auth.client_id,
    ))
    .context("cannot show auth in browser")?;
    println!("Paste the code your server gave you:");
    let mut code = String::new();
    io::stdin()
        .read_line(&mut code)
        .context("unable to read stdin")?;
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
    let config: Config =
        toml::from_str(&std::fs::read_to_string("config.toml").context("cannot read config.toml")?)
            .context("invalid config")?;
    println!("{} hashtags in config", config.hashtag.len());
    for hashtag in config.hashtag.iter() {
        let mut remote_statuses: HashSet<String> = HashSet::new();
        for server in hashtag.sources.iter() {
            remote_statuses.extend(
                hashtags(server, "", &hashtag.name, &hashtag.any, 25)?
                    .into_iter()
                    .map(|s| s.url),
            );
        }
        let local_statuses: HashSet<String> = HashSet::from_iter(
            hashtags(
                &config.server,
                &config.auth.token,
                &hashtag.name,
                &hashtag.any,
                40,
            )?
            .into_iter()
            .map(|s| s.url),
        );
        for status in remote_statuses.difference(&local_statuses) {
            println!("Importing {}", status);
            let res = import(&config.server, &config.auth.token, status);
            if let Err(e) = res {
                println!("Error: {e}");
            }
            sleep(Duration::from_secs(60));
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
        .context("token post failed")?
        .with_error_text()?;
    let token: Token = response.json().context("token body not valid json")?;
    Ok(token.access_token)
}

fn hashtags(
    server: &str,
    token: &str,
    name: &str,
    any: &Option<Vec<String>>,
    limit: u8,
) -> Result<Vec<Status>> {
    let response: Vec<Status> = client()?
        .get(
            reqwest::Url::parse_with_params(
                &format!("https://{server}/api/v1/timelines/tag/{name}?limit={limit}"),
                //"any[]=kr2023&any[]=KernelRecipes2023",
                any.iter()
                    .flat_map(|l| l.iter().map(|h| ("any[]", h)))
                    .collect::<Vec<_>>(),
            )
            .with_context(|| format!("hashtags url for {server}"))?
            .as_str(),
        )
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .context("hashtags get failed")?
        .with_error_text()?
        .json()
        .context("hash tag statuses body not valid json")?;
    Ok(response)
}

fn import(server: &str, token: &str, url: &str) -> Result<()> {
    client()?
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
        .context("import get failed")?
        .with_error_text()?;
    Ok(())
}

trait WithErrorText {
    fn with_error_text(self) -> Result<Self>
    where
        Self: Sized;
}
impl WithErrorText for reqwest::blocking::Response {
    fn with_error_text(self) -> Result<Self> {
        let status_err = self.error_for_status_ref();
        if let Err(e) = status_err {
            bail!(
                "Got response {}: {e}",
                self.text().context("body not valid text")?
            );
        }
        Ok(self)
    }
}
