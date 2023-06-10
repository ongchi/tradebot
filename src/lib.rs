pub mod config;
pub mod db;
pub mod exchange;
pub mod strategy;

use std::sync::Arc;

use tokio::time::Interval;

pub struct TradeBot {
    interval: Interval,
    exchange_cfg: Vec<exchange::Config>,
    pool: db::DbPool,
}

impl TradeBot {
    pub fn new(config: Arc<config::Config>) -> anyhow::Result<Self> {
        let interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        let exchange_cfg = config.exchanges.clone();
        let pool = db::get_pool(config.database.clone())?;

        Ok(Self {
            interval,
            exchange_cfg,
            pool,
        })
    }

    pub async fn run(&mut self) {
        loop {
            self.interval.tick().await;
            for cfg in &self.exchange_cfg {
                let db_pool = self.pool.clone();
                let config = cfg.clone();
                tokio::spawn(async move {
                    if let Err(error) = config.exec(db_pool).await {
                        log::error!("execution error: {:?}", error);
                    }
                })
                .await
                .expect("task panic");
            }
        }
    }
}
