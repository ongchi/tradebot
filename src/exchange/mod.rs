mod bitfinex;
mod cex;

use crate::db::DbPool;
use crate::strategy::{self, lending, Strategy};
use anyhow::Result;
use secrecy::Secret;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize)]
pub struct Params {
    pub api_key: Secret<String>,
    pub api_secret: Secret<String>,
    pub strategies: Vec<strategy::Config>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name")]
pub enum Config {
    Cex(Params),
    Bitfinex(Params),
}

impl Config {
    fn get_strategies(self) -> Vec<strategy::Config> {
        match self {
            Self::Cex(params) => params.strategies,
            Self::Bitfinex(params) => params.strategies,
        }
    }
}

pub enum ApiClient {
    Cex(Arc<cex::Client>),
    Bitfinex(Arc<bitfinex::Client>),
}

impl From<Config> for ApiClient {
    fn from(config: Config) -> Self {
        match config {
            Config::Cex(params) => ApiClient::Cex(Arc::new(params.into())),
            Config::Bitfinex(params) => ApiClient::Bitfinex(Arc::new(params.into())),
        }
    }
}

impl Config {
    pub async fn exec(&self, db_pool: DbPool) -> Result<()> {
        let client: Arc<ApiClient> = Arc::new(self.clone().into());
        let strategy_configs = self.clone().get_strategies();

        for config in strategy_configs {
            let client = client.clone();
            let db_pool = db_pool.clone();
            match config {
                strategy::Config::Lending(config) => {
                    lending::Strategy::new(client, db_pool, config).exec()?
                }
            };
        }

        Ok(())
    }
}
