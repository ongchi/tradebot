use anyhow::Result;
use serde::Deserialize;
use std::sync::Arc;

use crate::exchange::Exchange;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: Option<String>,
    pub exchanges: Vec<Exchange>,
}

impl Config {
    pub fn from_file(file_name: &str) -> Result<Arc<Self>> {
        let conf = config::Config::builder()
            .add_source(config::File::with_name(file_name))
            .build()?;
        Ok(Arc::new(conf.try_deserialize()?))
    }
}
