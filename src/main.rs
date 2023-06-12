use std::sync::OnceLock;
use std::time::Duration;

use anyhow::anyhow;
use clap::Parser;
use tokio_cron_scheduler::{Job, JobScheduler};

use tradebot::config;
use tradebot::db;
use tradebot::db::DbPool;
use tradebot::exchange;

#[derive(Parser)]
#[clap(version = "0.1")]
struct Opts {
    #[clap(short, long, default_value = "config.toml")]
    config: String,
}

static EXCHANGE: OnceLock<Vec<exchange::Exchange>> = OnceLock::new();
static DB_POOL: OnceLock<DbPool> = OnceLock::new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli_opts: Opts = Opts::parse();

    let conf = config::Config::from_file(cli_opts.config.as_str())?;
    let exchange = conf.exchanges.clone();
    let db_pool = db::get_pool(conf.database.clone())?;

    EXCHANGE.set(exchange).map_err(|e| anyhow!("{:?}", e))?;
    DB_POOL.set(db_pool).map_err(|e| anyhow!("{:?}", e))?;

    let sched = JobScheduler::new().await?;

    if let Some(exchange_cfg) = EXCHANGE.get() {
        for exch in exchange_cfg {
            if let Some(db_pool) = DB_POOL.get() {
                let bot = Job::new_repeated_async(Duration::from_secs(60), |_, _| {
                    Box::pin(async {
                        if let Err(e) = exch.exec(db_pool.clone()).await {
                            log::error!("{:?}", e);
                        }
                    })
                })?;
                sched.add(bot).await?;
            }
        }
    }

    sched.start().await?;

    loop {
        tokio::time::sleep(core::time::Duration::from_secs(600)).await;
    }
}
