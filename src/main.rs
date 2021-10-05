use anyhow::Result;
use clap::Clap;
// use env_logger;
use futures::join;
// use log;

mod config;
mod db;
mod exchange;
mod strategy;
mod web;

#[derive(Clap)]
#[clap(version = "0.1")]
struct Opts {
    #[clap(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let opts: Opts = Opts::parse();
    let conf = config::Config::from_file(opts.config.as_str())?;

    log::debug!("{:?}", conf);

    let db_pool = db::get_pool(conf.database.clone())?;

    join!(
        web::run_webserver(conf.webserver.clone()),
        strategy::run(conf.exchanges.clone(), db_pool)
    );

    Ok(())
}
