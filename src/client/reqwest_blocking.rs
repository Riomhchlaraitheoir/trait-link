use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{LinkError, BlockingTransport};
pub use reqwest::Error;

/// A AsyncClient which uses the [reqwest] crate
pub struct AsyncClient {
    client: reqwest::blocking::Client,
    url: String,
    method: reqwest::Method,
}

impl AsyncClient {
    /// Create a new client using the given URL and method
    pub fn new(url: &str, method: reqwest::Method) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            url: url.to_string(),
            method,
        }
    }
}

impl<Req, Resp> BlockingTransport<Req, Resp> for AsyncClient
where
    Req: Serialize,
    Resp: DeserializeOwned,
{
    type Error = Error;

    fn send(self, request: Req) -> Result<Resp, LinkError<Self::Error>> {
        (&self).send(request)
    }
}

impl<Req, Resp> BlockingTransport<Req, Resp> for &AsyncClient
where
    Req: Serialize,
    Resp: DeserializeOwned,
{
    type Error = Error;

    fn send(self, request: Req) -> Result<Resp, LinkError<Self::Error>> {
        Ok(self
            .client
            .request(self.method.clone(), &self.url)
            .json(&request)
            .send()?
            .json()?)
    }
}