use clap::Parser;

use tradebot::config;
use tradebot::db::{get_pool, DbPool};
use tradebot::exchange;

#[derive(Parser)]
#[clap(version = "0.1")]
struct Opts {
    #[clap(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let opts: Opts = Opts::parse();
    let conf = config::Config::from_file(opts.config.as_str())?;

    log::debug!("{:?}", conf);

    let db_pool = get_pool(conf.database.clone())?;

    run_tradebot(conf.exchanges.clone(), db_pool).await;

    Ok(())
}

async fn run_tradebot(exchange_configs: Vec<exchange::Config>, db_pool: DbPool) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        for config in exchange_configs.iter() {
            let db_pool = db_pool.clone();
            let config = config.clone();
            tokio::spawn(async move {
                if let Err(error) = config.exec(db_pool).await {
                    log::error!("{:?}", error);
                };
            });
        }
    }
}
