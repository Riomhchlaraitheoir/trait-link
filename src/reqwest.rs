use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{LinkError, Transport};
pub use reqwest::Error;

pub struct Client {
    client: reqwest::Client,
    url: String,
}

impl Client {
    pub fn new(url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
        }
    }
}

impl<Req, Resp> Transport<Req, Resp> for Client
where
    Req: Serialize,
    Resp: DeserializeOwned,
{
    type Error = Error;

    async fn send(self, request: Req) -> Result<Resp, LinkError<Self::Error>> {
        (&self).send(request).await
    }
}

impl<Req, Resp> Transport<Req, Resp> for &Client
where
    Req: Serialize,
    Resp: DeserializeOwned,
{
    type Error = Error;

    async fn send(self, request: Req) -> Result<Resp, LinkError<Self::Error>> {
        Ok(self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?)
    }
}