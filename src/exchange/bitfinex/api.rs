use anyhow::{anyhow, Result};
use chrono::{
    serde::{ts_milliseconds, ts_milliseconds_option},
    DateTime, Utc,
};
use hex::encode;
use hmac::{Hmac, Mac};
use reqwest::{blocking::Response, StatusCode};
use secrecy::ExposeSecret;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha384;
use std::time::{SystemTime, UNIX_EPOCH};

use super::deserializer::{bool_from_val, bool_from_val_option};
use super::Client;

static API_HOST: &str = "https://api.bitfinex.com/";

#[derive(Serialize, Deserialize, Debug)]
pub struct Trade {
    pub id: u32,
    #[serde(with = "ts_milliseconds")]
    pub mts: DateTime<Utc>,
    pub amount: f64,
    pub rate: f64,
    pub period: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Book {
    pub rate: f64,
    pub period: u32,
    pub count: u32,
    pub amount: f64, // ask if amount > 0
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FundingInfo {
    key: String,
    symbol: String,
    pub funding: InfoDetail,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoDetail {
    pub yield_loan: f64,
    pub yield_lend: f64,
    pub duration_loan: f64,
    pub duration_lend: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FundingOffer {
    pub id: u32,
    pub symbol: String,
    #[serde(with = "ts_milliseconds")]
    pub mts_created: DateTime<Utc>,
    #[serde(with = "ts_milliseconds")]
    pub mts_updated: DateTime<Utc>,
    pub amount: f64,
    pub amount_orig: f64,
    pub funding_type: String,
    #[serde(skip_serializing)]
    _placeholder_1: Option<String>,
    #[serde(skip_serializing)]
    _placeholder_2: Option<String>,
    pub flags: Option<Value>,
    pub status: String,
    #[serde(skip_serializing)]
    _placeholder_3: Option<String>,
    #[serde(skip_serializing)]
    _placeholder_4: Option<String>,
    #[serde(skip_serializing)]
    _placeholder_5: Option<String>,
    pub rate: f64,
    pub period: u32,
    #[serde(deserialize_with = "bool_from_val")]
    pub notify: bool,
    #[serde(deserialize_with = "bool_from_val")]
    pub hidden: bool,
    #[serde(skip_serializing)]
    _placeholder_6: Option<String>,
    #[serde(deserialize_with = "bool_from_val")]
    pub renew: bool,
    #[serde(skip_serializing)]
    _placeholder_7: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FundingOfferResponse {
    #[serde(with = "ts_milliseconds")]
    pub mts: DateTime<Utc>,
    pub funding_type: String,
    pub message_id: Option<u64>,
    #[serde(skip_serializing)]
    _placeholder_1: Option<String>,
    pub offer: FundingOffer,
    pub code: Option<u64>,
    pub status: String,
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FundingCredit {
    pub id: u32,
    pub symbol: String,
    pub side: i8,
    #[serde(with = "ts_milliseconds")]
    pub mts_create: DateTime<Utc>,
    #[serde(with = "ts_milliseconds")]
    pub mts_update: DateTime<Utc>,
    pub amount: f64,
    pub flags: Option<Value>,
    pub status: String,
    #[serde(skip_serializing)]
    _placeholder_1: Option<String>,
    #[serde(skip_serializing)]
    _placeholder_2: Option<String>,
    #[serde(skip_serializing)]
    _placeholder_3: Option<String>,
    pub rate: f64,
    pub period: u32,
    #[serde(with = "ts_milliseconds")]
    pub mts_opening: DateTime<Utc>,
    #[serde(default)]
    #[serde(with = "ts_milliseconds_option")]
    pub mts_last_payout: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "bool_from_val_option")]
    pub notify: Option<bool>,
    #[serde(deserialize_with = "bool_from_val")]
    pub hidden: bool,
    #[serde(skip_serializing)]
    _placeholder_4: Option<String>,
    #[serde(deserialize_with = "bool_from_val")]
    pub renew: bool,
    #[serde(skip_serializing)]
    _placeholder_5: Option<String>,
    #[serde(deserialize_with = "bool_from_val")]
    pub no_close: bool,
    pub position_pair: String,
}

impl Client {
    pub fn trades(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Trade>> {
        self.get(
            &format!("v2/trades/{symbol}/hist"),
            &[
                ("start", format!("{}", start.timestamp() * 1000)),
                ("end", format!("{}", end.timestamp() * 1000)),
            ],
        )
    }

    pub fn books(&self, symbol: &str) -> Result<Vec<Book>> {
        self.get(&format!("v2/book/{symbol}/P3"), &[("", "")])
    }

    pub fn funding_info(&self, symbol: &str) -> Result<FundingInfo> {
        self.post(&format!("v2/auth/r/info/funding/{symbol}"), json!({}))
    }

    //
    // Funding
    //
    pub fn funding_balance_available(&self, symbol: &str) -> Result<f64> {
        let balance: Vec<f64> = self.post(
            "v2/auth/calc/order/avail",
            json!({
                "symbol": symbol,
                "type": "FUNDING",
            }),
        )?;

        Ok(-balance.first().unwrap())
    }

    pub fn active_funding_offers(&self, symbol: &str) -> Result<Vec<FundingOffer>> {
        self.post(&format!("v2/auth/r/funding/offers/{symbol}"), json!({}))
    }

    pub fn submit_funding_offer(
        &self,
        symbol: &str,
        amount: f64,
        rate: f64,
        period: u32,
    ) -> Result<FundingOfferResponse> {
        self.post(
            "v2/auth/w/funding/offer/submit",
            json!({
                "type": "LIMIT",
                "symbol": symbol,
                "amount": amount.to_string(),
                "rate": rate.to_string(),
                "period": period,
            }),
        )
    }

    pub fn cancel_funding_offer(&self, id: u32) -> Result<FundingOfferResponse> {
        self.post("v2/auth/w/funding/offer/cancel", json!({ "id": id }))
    }

    pub fn funding_credits(&self, symbol: &str) -> Result<Vec<FundingCredit>> {
        self.post(&format!("v2/auth/r/funding/credits/{symbol}"), json!({}))
    }

    pub fn funding_credit_history(&self, symbol: &str) -> Result<Vec<FundingCredit>> {
        self.post(
            &format!("v2/auth/r/funding/credits/{symbol}/hist"),
            json!({}),
        )
    }

    fn get<P, R>(&self, path: &str, params: &P) -> Result<R>
    where
        P: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let url = format!("{API_HOST}{path}");
        let response = self.client.get(url).query(params).send()?;

        self.response_body(response)
    }

    fn post<R>(&self, path: &str, payload: Value) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let url = format!("{API_HOST}{path}");
        let nonce = (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() * 1000000 + 524287)
            .to_string();
        let sig = {
            let mut mac =
                Hmac::<Sha384>::new_from_slice(self.api_secret.expose_secret().as_bytes())?;
            mac.update(format!("/api/{path}{nonce}{payload}").as_bytes());
            encode(mac.finalize().into_bytes())
        };

        let response = self
            .client
            .post(url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("bfx-nonce", nonce)
            .header("bfx-apikey", self.api_key.expose_secret())
            .header("bfx-signature", sig)
            .json(&payload)
            .send()?;

        self.response_body(response)
    }

    fn response_body<R>(&self, response: Response) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let url = response.url().clone();
        match response.status() {
            StatusCode::OK => match response.json::<R>() {
                Ok(d) => Ok(d),
                Err(e) => Err(anyhow!("[{}]: {}", url, e)),
            },
            s => {
                if let Ok(message) = response.text() {
                    Err(anyhow!("[{}] {}: {:?}", url, s, message))
                } else {
                    Err(anyhow!("[{}]: {:?}", url, s))
                }
            }
        }
    }
}
