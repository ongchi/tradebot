mod bitfinex;
mod cex;

use crate::db::DbPool;
use crate::strategy::{self, lending, Strategy};
use anyhow::{anyhow, Result};
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
pub enum Exchange {
    Cex(Params),
    Bitfinex(Params),
}

impl Exchange {
    fn get_strategies(self) -> Vec<strategy::Config> {
        match self {
            Self::Cex(params) => params.strategies,
            Self::Bitfinex(params) => params.strategies,
        }
    }
}

pub enum ExchangeApiClient {
    Cex(Arc<cex::Client>),
    Bitfinex(Arc<bitfinex::Client>),
}

impl From<Exchange> for ExchangeApiClient {
    fn from(config: Exchange) -> Self {
        match config {
            Exchange::Cex(params) => ExchangeApiClient::Cex(Arc::new(params.into())),
            Exchange::Bitfinex(params) => ExchangeApiClient::Bitfinex(Arc::new(params.into())),
        }
    }
}

impl Exchange {
    pub async fn exec(&self, db_pool: DbPool) -> Result<()> {
        let client: Arc<ExchangeApiClient> = Arc::new(self.clone().into());
        let strategy_configs = self.clone().get_strategies();

        for config in strategy_configs {
            let client = client.clone();
            let db_pool = db_pool.clone();
            match config {
                strategy::Config::Lending(config) => {
                    lending::Strategy::new(client, db_pool, config)
                        .exec()
                        .map_err(|e| anyhow!("[{:?}]: {:?}", self, e))?
                }
            };
        }

        Ok(())
    }
}
