use anyhow::{anyhow, Result};
use chrono::{
    serde::{ts_milliseconds, ts_milliseconds_option},
    DateTime, Utc,
};
use hex::encode;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::{blocking::Response, StatusCode};
use ring::hmac;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::convert::From;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::deserializer::{bool_from_val, bool_from_val_option};
use super::Bitfinex;
use crate::strategy::lending;

static API_HOST: &str = "https://api.bitfinex.com/";
static NO_PARAMS: [(); 0] = [];

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct Trade {
    pub id: u32,
    #[serde(with = "ts_milliseconds")]
    pub mts: DateTime<Utc>,
    pub amount: f64,
    pub rate: f64,
    pub period: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct Book {
    pub rate: f64,
    pub period: u32,
    pub count: u32,
    pub amount: f64, // ask if amount > 0
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct FundingInfo {
    key: String,
    symbol: String,
    pub funding: InfoDetail,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct InfoDetail {
    pub yield_loan: f64,
    pub yield_lend: f64,
    pub duration_loan: f64,
    pub duration_lend: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct FundingOffer {
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
pub(super) struct FundingOfferResponse {
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
pub(super) struct FundingCredit {
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

impl Bitfinex {
    pub(crate) fn new(api_key: &str, api_secret: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            strategies: Arc::new(vec![]),
            client: reqwest::blocking::Client::new(),
        }
    }

    pub(super) fn trades(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Trade>> {
        self.get(
            format!("v2/trades/{}/hist", symbol).as_str(),
            &[
                ("start", format!("{}", start.timestamp() * 1000)),
                ("end", format!("{}", end.timestamp() * 1000)),
            ],
        )
    }

    pub(super) fn books(&self, symbol: &str) -> Result<Vec<Book>> {
        self.get(format!("v2/book/{}/P3", symbol).as_str(), &NO_PARAMS)
    }

    pub(super) fn funding_info(&self, symbol: &str) -> Result<FundingInfo> {
        self.post_signed(
            format!("v2/auth/r/info/funding/{}", symbol).as_str(),
            "{}",
            &NO_PARAMS,
        )
    }

    //
    // Funding
    //
    pub(super) fn funding_balance_available(&self, symbol: &str) -> Result<f64> {
        let payload = json!({
            "symbol": symbol,
            "type": "FUNDING",
        })
        .to_string();

        let balance: Vec<f64> =
            self.post_signed("v2/auth/calc/order/avail", payload.as_str(), &NO_PARAMS)?;

        Ok(-balance.first().unwrap())
    }

    pub(super) fn active_funding_offers(&self, symbol: &str) -> Result<Vec<FundingOffer>> {
        self.post_signed(
            format!("v2/auth/r/funding/offers/{}", symbol).as_str(),
            "{}",
            &NO_PARAMS,
        )
    }

    pub(super) fn submit_funding_offer(
        &self,
        symbol: &str,
        amount: f64,
        rate: f64,
        period: u32,
    ) -> Result<FundingOfferResponse> {
        let payload = json!({
            "type": "LIMIT",
            "symbol": symbol,
            "amount": amount.to_string(),
            "rate": rate.to_string(),
            "period": period,
        })
        .to_string();

        self.post_signed(
            "v2/auth/w/funding/offer/submit",
            payload.as_str(),
            &NO_PARAMS,
        )
    }

    pub(super) fn cancel_funding_offer(&self, id: u32) -> Result<FundingOfferResponse> {
        let payload = json!({ "id": id }).to_string();

        self.post_signed(
            "v2/auth/w/funding/offer/cancel",
            payload.as_str(),
            &NO_PARAMS,
        )
    }

    pub(super) fn funding_credits(&self, symbol: &str) -> Result<Vec<FundingCredit>> {
        self.post_signed(
            format!("v2/auth/r/funding/credits/{}", symbol).as_str(),
            "{}",
            &NO_PARAMS,
        )
    }

    pub(super) fn funding_credit_history(&self, symbol: &str) -> Result<Vec<FundingCredit>> {
        self.post_signed(
            format!("v2/auth/r/funding/credits/{}/hist", symbol).as_str(),
            "{}",
            &NO_PARAMS,
        )
    }

    fn get<P, D>(&self, api_path: &str, params: &P) -> Result<D>
    where
        P: Serialize,
        D: DeserializeOwned,
    {
        let url = format!("{}{}", API_HOST, api_path);
        let response = self.client.get(&url).query(params).send()?;

        self.response_body(response)
    }

    fn post_signed<P, D>(&self, api_path: &str, payload: &str, params: &P) -> Result<D>
    where
        P: Serialize,
        D: DeserializeOwned,
    {
        let url = format!("{}{}", API_HOST, api_path);
        let headers = self.build_headers(api_path, payload)?;
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .body(payload.to_string())
            .query(params)
            .send()?;

        self.response_body(response)
    }

    fn build_headers(&self, api_path: &str, payload: &str) -> Result<HeaderMap> {
        let nonce =
            (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() * 1000 + 524287).to_string();
        let signature: String = format!("/api/{}{}{}", api_path, nonce, payload);
        let signed_key = hmac::Key::new(hmac::HMAC_SHA384, self.api_secret.as_bytes());
        let sig = encode(hmac::sign(&signed_key, signature.as_bytes()).as_ref());

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            HeaderName::from_static("bfx-nonce"),
            HeaderValue::from_str(nonce.as_str())?,
        );
        headers.insert(
            HeaderName::from_static("bfx-apikey"),
            HeaderValue::from_str(self.api_key.as_str())?,
        );
        headers.insert(
            HeaderName::from_static("bfx-signature"),
            HeaderValue::from_str(sig.as_str())?,
        );

        Ok(headers)
    }

    fn response_body<D>(&self, response: Response) -> Result<D>
    where
        D: DeserializeOwned,
    {
        match response.status() {
            StatusCode::OK => {
                let body = response.text()?;

                match serde_json::from_str(body.as_str()) {
                    Ok(d) => Ok(d),
                    Err(e) => Err(anyhow!("reason => {:?} body => {}", e, body)),
                }
            }
            s => Err(anyhow!("{:?}", s)),
        }
    }
}

impl From<FundingOffer> for lending::Offer {
    fn from(item: FundingOffer) -> Self {
        Self {
            id: item.id,
            symbol: item.symbol,
            rate: item.rate,
            mts_created: item.mts_created,
        }
    }
}

impl From<Book> for lending::Book {
    fn from(item: Book) -> Self {
        Self {
            amount: item.amount,
            rate: item.rate,
            period: item.period,
        }
    }
}

impl From<Trade> for lending::Trade {
    fn from(item: Trade) -> Self {
        Self {
            mts: item.mts,
            amount: item.amount,
            rate: item.rate,
            period: item.period,
        }
    }
}

impl From<FundingCredit> for lending::Credit {
    fn from(item: FundingCredit) -> Self {
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

impl From<FundingInfo> for lending::Info {
    fn from(item: FundingInfo) -> Self {
        Self {
            yield_lend: item.funding.yield_lend,
            duration_lend: item.funding.duration_lend,
        }
    }
}
