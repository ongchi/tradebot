use crate::db::{DbConn, DbPool};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info};
use rusqlite::params;
use serde::Deserialize;
use std::sync::Arc;

pub struct Info {
    pub yield_lend: f64,
    pub duration_lend: f64,
}

pub struct Offer {
    pub id: u32,
    pub symbol: String,
    pub rate: f64,
    pub mts_created: DateTime<Utc>,
}

pub struct Book {
    pub amount: f64,
    pub rate: f64,
    pub period: u32,
}

pub struct Trade {
    pub mts: DateTime<Utc>,
    pub amount: f64,
    pub rate: f64,
    pub period: u32,
}

pub struct Credit {
    pub id: u32,
    pub symbol: String,
    pub mts_create: DateTime<Utc>,
    pub mts_update: DateTime<Utc>,
    pub amount: f64,
    pub rate: f64,
    pub period: u32,
    pub mts_opening: DateTime<Utc>,
    pub mts_last_payout: Option<DateTime<Utc>>,
    pub position_pair: String,
}

pub trait Api: std::fmt::Debug {
    fn info(&self, symbol: &str) -> Result<Info>;
    fn history(&self, symbol: &str, start: DateTime<Utc>, end: DateTime<Utc>)
        -> Result<Vec<Trade>>;
    fn credit_history(&self, symbol: &str) -> Result<Vec<Credit>>;
    fn credits(&self, symbol: &str) -> Result<Vec<Credit>>;
    fn balance(&self, symbol: &str) -> Result<f64>;
    fn active_offers(&self, symbol: &str) -> Result<Vec<Offer>>;
    fn submit_offer(&self, symbol: &str, amount: f64, rate: f64, period: u32) -> Result<()>;
    fn cancel_offer(&self, id: u32) -> Result<()>;
    fn books(&self, symbol: &str) -> Result<Vec<Book>>;
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub symbol: String,
    pub lending_size: Option<f64>,
    pub min_apy: Option<f64>,
    pub max_apy: Option<f64>,
    pub reserved_amount_1: Option<f64>,
    pub reserved_amount_2: Option<f64>,
}

#[derive(Debug)]
pub struct Strategy {
    client: Arc<dyn Api>,
    db_connection: DbConn,
    config: Config,
    now: DateTime<Utc>,
    last_tick: DateTime<Utc>,
}

impl Strategy {
    pub fn new(client: Arc<crate::exchange::ApiClient>, db_pool: DbPool, config: Config) -> Self {
        let now = Utc::now();
        let last_tick = now - Duration::minutes(1);
        let client = match client.as_ref() {
            crate::exchange::ApiClient::Cex(_) => unimplemented!(),
            crate::exchange::ApiClient::Bitfinex(client) => client.clone(),
        };

        Self {
            client,
            db_connection: db_pool.get().unwrap(),
            config,
            now,
            last_tick,
        }
    }

    pub fn log_history(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<()> {
        let symbol = self.config.symbol.as_str();
        let history = self.client.history(symbol, start, end)?;
        for h in &history {
            self.db_connection
                .execute(
                    "INSERT INTO trades (symbol, mts, amount, rate, period)
    VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![format!("f{symbol}"), &h.mts, &h.amount, &h.rate, &h.period],
                )
                .map_err(|err| anyhow!("failed to log history: {:?}", err))?;
        }

        Ok(())
    }

    pub fn log_credits(&self) -> Result<()> {
        let symbol = self.config.symbol.as_str();
        let credits = self.client.credit_history(symbol)?;
        for c in &credits {
            self.db_connection
                .execute(
                    "INSERT OR REPLACE INTO credits VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        &c.id,
                        &c.symbol,
                        &c.amount,
                        &c.rate,
                        &c.period,
                        &c.mts_opening,
                        &c.mts_last_payout,
                        &c.position_pair
                    ],
                )
                .map_err(|err| anyhow!("failed to log credits: {:?}", err))?;
        }

        Ok(())
    }

    pub fn log_provided(&self) -> Result<()> {
        let symbol = self.config.symbol.as_str();
        let credits = self.client.credits(symbol)?;
        for c in &credits {
            self.db_connection
                .execute(
                    "INSERT OR REPLACE INTO provided VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        &c.id,
                        &c.symbol,
                        &c.mts_create,
                        &c.mts_update,
                        &c.amount,
                        &c.rate,
                        &c.period,
                        &c.position_pair
                    ],
                )
                .map_err(|err| anyhow!("failed to log provided: {:?}", err))?;
        }

        Ok(())
    }

    fn get_rate(&self) -> Result<f64> {
        let symbol = self.config.symbol.as_str();
        match self.db_connection.query_row(
            "SELECT MAX(rate) * 0.8 + AVG(rate) * 0.2
            FROM trades
            WHERE symbol = ?1 AND DATETIME(mts) > DATETIME('now', '-3 hours')",
            params![format!("f{symbol}")],
            |row| row.get(0),
        ) {
            Ok(rate) => Ok(rate),
            Err(_) => {
                match self.db_connection.query_row(
                    "SELECT MAX(rate) * 0.8 + AVG(rate) * 0.2
                    FROM trades
                    WHERE symbol = ?1 ORDER BY DATETIME(mts) DESC LIMIT 100",
                    params![format!("f{symbol}")],
                    |row| row.get(0),
                ) {
                    Ok(rate) => Ok(rate),
                    Err(e) => Err(anyhow!("failed to get rate: {:?}", e)),
                }
            }
        }
    }

    fn update_offer(&self) -> Result<()> {
        let symbol = self.config.symbol.as_str();

        // cancel offer if rate difference > 5% or creation time > 1 hours
        let rate = self.get_rate()?;

        for offer in self.client.active_offers(symbol)? {
            if (rate - offer.rate).abs() / rate > 0.05
                && (self.last_tick - offer.mts_created) > Duration::hours(1)
            {
                self.client.cancel_offer(offer.id)?;
            }
        }

        Ok(())
    }

    fn get_fair_offer_pair(&self) -> Result<Vec<(f64, u32)>> {
        let symbol = self.config.symbol.as_str();

        let rate = self.get_rate()?;

        let mut offer_pair: Vec<(f64, u32)> = self
            .client
            .books(symbol)?
            .iter()
            .filter_map(|b| {
                if b.amount < 0.
                    && ((b.rate - rate).abs() / rate <= 0.05 || period_by_rate(b.rate) >= b.period)
                {
                    Some((b.rate, b.period))
                } else {
                    None
                }
            })
            .collect();
        offer_pair.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        Ok(offer_pair)
    }

    fn submit_offer(&self) -> Result<()> {
        let symbol = self.config.symbol.as_str();

        self.update_offer()?;

        let lend_unit_amount = self.config.lending_size.unwrap_or(200.0);
        let min_lend_rate = self.config.min_apy.unwrap_or(0.0003);
        let max_lend_rate = self.config.max_apy.unwrap_or(0.00082);
        let preserved_amoount = self.config.reserved_amount_1.unwrap_or(1000.0);
        let preserved_amoount_l2 = self.config.reserved_amount_1.unwrap_or(1500.0);

        if let Ok(ba) = self.client.balance(symbol) {
            log::debug!("balance available: {}", ba);
            if ba >= lend_unit_amount {
                let mut amount = (ba * 100.0).floor() / 100.0;

                let rate = self.get_rate()?;
                let period = period_by_rate(rate);

                // sumit offer by calculated rate
                if rate >= min_lend_rate || amount > preserved_amoount {
                    self.client
                        .submit_offer(symbol, lend_unit_amount, rate, period)?;
                    amount -= lend_unit_amount;
                }

                // submit if fair offer found
                for (b_rate, b_period) in self.get_fair_offer_pair()? {
                    let period_lim = period_by_rate(b_rate);

                    if amount > lend_unit_amount
                        && (b_rate > max_lend_rate
                            || (amount > preserved_amoount
                                && period_lim >= b_period
                                && b_rate >= min_lend_rate))
                    {
                        self.client
                            .submit_offer(symbol, lend_unit_amount, b_rate, period_lim)?;
                        amount -= lend_unit_amount;
                    } else if amount > preserved_amoount_l2
                        && b_period <= 5
                        && b_rate >= min_lend_rate
                    {
                        self.client
                            .submit_offer(symbol, lend_unit_amount, b_rate, b_period)?;
                        amount -= lend_unit_amount;
                    } else {
                        debug!(
                        "condition not met for (avail, rate, period, period_lim) = ({:.2}, {:.4}, {}, {})",
                        amount, b_rate * 100.0, b_period, period_lim
                    );
                    }
                }
            }
        }

        Ok(())
    }
}

fn period_by_rate(rate: f64) -> u32 {
    match rate {
        x if (0.00035..0.0004).contains(&x) => (4. * x * 10000. - 9.) as u32,
        x if (0.0004..0.0005).contains(&x) => (8. * x * 10000. - 25.) as u32,
        x if (0.0005..0.0006).contains(&x) => (15. * x * 10000. - 60.) as u32,
        x if (0.0006..0.0007).contains(&x) => (40. * x * 10000. - 210.) as u32,
        x if (0.0007..0.0008).contains(&x) => (25. * x * 10000. - 105.) as u32,
        x if (0.0008..0.0009).contains(&x) => (20. * x * 10000. - 65.) as u32,
        x if (0.0009..0.001).contains(&x) => (5. * x * 10000. - 70.) as u32,
        x if x > 0.001 => 120,
        _ => 2,
    }
}

impl super::Strategy for Strategy {
    fn exec(&mut self) -> Result<()> {
        if self.log_history(self.last_tick, self.now).is_ok() {
            self.last_tick = self.now;
            self.now += Duration::minutes(1);
        } else {
            error!("History fetch error");
        };

        self.submit_offer()?;
        self.log_credits()?;
        self.log_provided()?;

        let info = self.client.info(self.config.symbol.clone().as_str())?;
        let rate = self.get_rate()?;
        info!(
            "{} => (rate, r_3h, dur) = ({:.4}, {:.4}, {:.0})",
            self.config.symbol,
            info.yield_lend * 100.,
            rate * 100.,
            info.duration_lend
        );

        Ok(())
    }
}
