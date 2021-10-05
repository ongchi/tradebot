use anyhow::Result;
use chrono::Duration;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
// use tokio;

pub(super) mod lending;
use crate::db::DbConn;
use crate::db::DbPool;
use crate::exchange;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name")]
pub enum Config {
    Lending(lending::Config),
}

impl Config {
    pub fn get_strategy(&self, cex: &exchange::Exchange, db_pool: DbPool ) -> Arc<Mutex<dyn Strategy>> {
        let db_conn = db_pool.get().unwrap();
        let strategy = match &self {
            Config::Lending(c) => {
                let cex = match cex {
                    exchange::Exchange::Bitfinex(x) => x.clone(),
                };
                lending::Strategy::new(cex, db_conn, c.clone())
            }
        };
        Arc::new(Mutex::new(strategy))
    }
}

pub trait Strategy {
    fn run(&mut self) -> Result<()>;
}

pub async fn run(exchanges: Vec<exchange::Exchange>, db_pool: DbPool) {
    let mut interval = tokio::time::interval(Duration::minutes(1).to_std().unwrap());
    let exchanges: Vec<Arc<exchange::Exchange>> =
        exchanges.into_iter().map(|x| Arc::new(x)).collect();
    loop {
        interval.tick().await;
        for x in exchanges.iter() {
            let cex = x.clone();
            let pool = db_pool.clone();
            tokio::spawn(async move {
                cex.run(pool);
            });
        }
    }
}
