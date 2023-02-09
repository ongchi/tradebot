use anyhow::Result;
use chrono::{DateTime, Utc};
use std::convert::From;

use crate::strategy::lending::{Api, Book, Credit, Info, Offer, Trade};

impl From<super::FundingOffer> for Offer {
    fn from(item: super::FundingOffer) -> Self {
        Self {
            id: item.id,
            symbol: item.symbol,
            rate: item.rate,
            mts_created: item.mts_created,
        }
    }
}

impl From<super::Book> for Book {
    fn from(item: super::Book) -> Self {
        Self {
            amount: item.amount,
            rate: item.rate,
            period: item.period,
        }
    }
}

impl From<super::Trade> for Trade {
    fn from(item: super::Trade) -> Self {
        Self {
            mts: item.mts,
            amount: item.amount,
            rate: item.rate,
            period: item.period,
        }
    }
}

impl From<super::FundingCredit> for Credit {
    fn from(item: super::FundingCredit) -> Self {
        Self {
            id: item.id,
            symbol: item.symbol,
            mts_create: item.mts_create,
            mts_update: item.mts_update,
            amount: item.amount,
            rate: item.rate,
            period: item.period,
            mts_opening: item.mts_opening,
            mts_last_payout: item.mts_last_payout,
            position_pair: item.position_pair,
        }
    }
}

impl From<super::FundingInfo> for Info {
    fn from(item: super::FundingInfo) -> Self {
        Self {
            yield_lend: item.funding.yield_lend,
            duration_lend: item.funding.duration_lend,
        }
    }
}

impl Api for super::Client {
    fn info(&self, symbol: &str) -> Result<Info> {
        Ok(self.funding_info(symbol).unwrap().into())
    }
    fn history(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Trade>> {
        let trades = self.trades(symbol, start, end)?;
        Ok(trades.into_iter().map(|t| t.into()).collect())
    }
    fn credits(&self, symbol: &str) -> Result<Vec<Credit>> {
        let credits = self.funding_credits(symbol)?;
        Ok(credits.into_iter().map(|c| c.into()).collect())
    }
    fn credit_history(&self, symbol: &str) -> Result<Vec<Credit>> {
        let credits = self.funding_credit_history(symbol)?;
        Ok(credits.into_iter().map(|c| c.into()).collect())
    }
    fn balance(&self, symbol: &str) -> Result<f64> {
        self.funding_balance_available(symbol)
    }
    fn active_offers(&self, symbol: &str) -> Result<Vec<Offer>> {
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
    fn books(&self, symbol: &str) -> Result<Vec<Book>> {
        let books = self.books(symbol)?;
        Ok(books.into_iter().map(|b| b.into()).collect())
    }
}
