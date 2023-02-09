use anyhow::Result;
use serde::Deserialize;
use std::sync::Arc;

use crate::exchange;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub webserver: Option<String>,
    pub database: Option<String>,
    pub exchanges: Vec<exchange::Config>,
}

impl Config {
    pub fn from_file(file_name: &str) -> Result<Arc<Self>> {
        let mut conf = config::Config::new();
        conf.merge(config::File::with_name(file_name))?;
        Ok(Arc::new(conf.try_into()?))
    }
}
