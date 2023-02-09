#[derive(Clone, Debug)]
pub struct Client {
    pub client: reqwest::blocking::Client,
}

impl From<super::Params> for Client {
    fn from(_item: super::Params) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}
