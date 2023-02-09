pub mod lending;

use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name")]
pub enum Config {
    Lending(lending::Config),
}

pub trait Strategy {
    fn exec(&mut self) -> Result<()>;
}
