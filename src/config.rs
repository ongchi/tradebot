use anyhow::Result;
use serde::Deserialize;
use std::sync::Arc;

use crate::exchange::Exchange;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub webserver: Option<String>,
    pub database: Option<String>,
    pub exchanges: Vec<Exchange>,
}

impl Config {
    pub fn from_file(file_name: &str) -> Result<Arc<Self>> {
        let mut conf = config::Config::new();
        conf.merge(config::File::with_name(file_name))?;
        let conf: Config = conf.try_into()?;
        Ok(Arc::new(conf))
    }
}
