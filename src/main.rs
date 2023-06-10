use clap::Parser;

use tradebot::{config, TradeBot};

#[derive(Parser)]
#[clap(version = "0.1")]
struct Opts {
    #[clap(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli_opts: Opts = Opts::parse();
    let conf = config::Config::from_file(cli_opts.config.as_str())?;

    log::debug!("{:?}", conf);

    let mut bot = TradeBot::new(conf)?;
    bot.run().await;

    Ok(())
}
