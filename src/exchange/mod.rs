use log::error;
use serde::Deserialize;
use std::sync::Arc;

mod bitfinex;
mod deserializer;
use crate::db::DbPool;
use crate::strategy;

trait ExchangeEntry {
    fn strategy_configs(&self) -> Arc<Vec<Arc<strategy::Config>>>;
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExchangeConfig {
    api_key: Option<String>,
    api_secret: Option<String>,
    strategies: Arc<Vec<Arc<strategy::Config>>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name")]
pub enum Exchange {
    Bitfinex(Arc<ExchangeConfig>),
}

impl Exchange {
    fn configs(&self) -> Arc<Vec<Arc<strategy::Config>>> {
        match self {
            Self::Bitfinex(x) => x.strategies.clone(),
        }
    }

    pub fn run(&self, db_pool: DbPool) {
        for config in self.configs().iter() {
            let strategy = config.get_strategy(self, db_pool.clone());

            match strategy.lock().unwrap().run() {
                Ok(_) => {}
                Err(e) => {
                    error!("{:?}", e)
                }
            };
        }
    }
}
