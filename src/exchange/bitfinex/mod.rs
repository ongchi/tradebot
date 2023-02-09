mod api;
mod deserializer;
mod lending;

use secrecy::Secret;

use api::*;

#[derive(Clone, Debug)]
pub struct Client {
    pub api_key: Secret<String>,
    pub api_secret: Secret<String>,
    pub client: reqwest::blocking::Client,
}

impl From<super::Params> for Client {
    fn from(item: super::Params) -> Self {
        Self {
            api_key: item.api_key,
            api_secret: item.api_secret,
            client: reqwest::blocking::Client::new(),
        }
    }
}
