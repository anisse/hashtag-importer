use serde::Deserialize;
use serde::Serialize;

pub const OOB_URI: &str = "urn:ietf:wg:oauth:2.0:oob";

#[derive(Serialize)]
pub struct ApplicationRegistration<'a> {
    pub client_name: &'a str,
    pub redirect_uris: &'a str,
    pub website: &'a str,
    pub scopes: Scope, // TODO: more than one scope
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum Scope {
    Read,
    #[serde(rename = "read:statuses")]
    ReadStatuses,
    #[serde(rename = "read:search")]
    ReadSearch,
    Write,
}

#[derive(Debug, Deserialize)]
pub struct ApplicationResponse {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    //pub name: String,
}

#[derive(Serialize)]
pub struct TokenQuery<'a> {
    pub redirect_uri: &'a str,
    pub code: Option<&'a str>,
    pub grant_type: GrantType,
    pub client_id: &'a str,
    pub client_secret: &'a str,
    pub scope: Option<Scope>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum GrantType {
    AuthorizationCode,
    ClientCredentials,
}

#[derive(Deserialize)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub created_at: u64,
    pub scope: String,
}
