use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{LinkError, Transport};
pub use reqwest::Error;

/// A Client which uses the [reqwest] crate
pub struct Client {
    client: reqwest::Client,
    url: String,
    method: reqwest::Method,
}

impl Client {
    /// Create a new client using the given URL and method
    pub fn new(url: &str, method: reqwest::Method) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
            method,
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
            .request(self.method.clone(), &self.url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?)
    }
}