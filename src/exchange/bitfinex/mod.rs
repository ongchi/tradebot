mod api;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;

use super::deserializer;
use crate::strategy::{self, lending};

#[derive(Clone, Debug, Deserialize)]
pub struct Bitfinex {
    api_key: String,
    api_secret: String,
    strategies: Arc<Vec<Arc<strategy::Config>>>,
    #[serde(skip_deserializing)]
    client: reqwest::blocking::Client,
}

impl super::ExchangeEntry for Bitfinex {
    fn strategy_configs(&self) -> Arc<Vec<Arc<strategy::Config>>> {
        self.strategies.clone()
    }
}

impl lending::Api for Bitfinex {
    fn info(&self, symbol: &str) -> Result<lending::Info> {
        Ok(self.funding_info(symbol).unwrap().into())
    }
    fn history(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<lending::Trade>> {
        let trades = self.trades(symbol, start, end)?;
        Ok(trades.into_iter().map(|t| t.into()).collect())
    }
    fn credits(&self, symbol: &str) -> Result<Vec<lending::Credit>> {
        let credits = self.funding_credits(symbol)?;
        Ok(credits.into_iter().map(|c| c.into()).collect())
    }
    fn credit_history(&self, symbol: &str) -> Result<Vec<lending::Credit>> {
        let credits = self.funding_credit_history(symbol)?;
        Ok(credits.into_iter().map(|c| c.into()).collect())
    }
    fn balance(&self, symbol: &str) -> Result<f64> {
        self.funding_balance_available(symbol)
    }
    fn active_offers(&self, symbol: &str) -> Result<Vec<lending::Offer>> {
        let offers = self.active_funding_offers(symbol)?;
        Ok(offers.into_iter().map(|o| o.into()).collect())
    }
    fn submit_offer(&self, symbol: &str, amount: f64, rate: f64, period: u32) -> Result<()> {
        self.submit_funding_offer(symbol, amount, rate, period)?;
        Ok(())
    }
    fn cancel_offer(&self, id: u32) -> Result<()> {
        self.cancel_funding_offer(id)?;
        Ok(())
    }
    fn books(&self, symbol: &str) -> Result<Vec<lending::Book>> {
        let books = self.books(symbol)?;
        Ok(books.into_iter().map(|b| b.into()).collect())
    }
}
