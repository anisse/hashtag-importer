use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub auth: Auth,
    pub server: String,
    pub hashtag: Vec<Hashtag>,
}
#[derive(Deserialize)]
pub(crate) struct Auth {
    pub client_id: String,
    pub client_secret: String,
    pub token: String,
}
#[derive(Deserialize)]
pub(crate) struct Hashtag {
    //Be very careful: base tag needs to exist in source servers otherwiise additionnal tags will be useless
    pub name: String,
    pub any: Option<Vec<String>>,
    pub sources: Vec<String>,
}

pub(crate) fn load_config(filename: &str) -> Result<Config> {
    let config: Config = toml::from_str(
        &std::fs::read_to_string(filename).with_context(|| format!("cannot read {filename}"))?,
    )
    .with_context(|| format!("invalid config file {filename}"))?;
    Ok(config)
}
