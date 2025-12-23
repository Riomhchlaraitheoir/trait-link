pub use serde;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use thiserror::Error;

pub mod server;
#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(all(feature = "browser", target_arch = "wasm32"))]
pub mod browser;
#[cfg(all(feature = "browser", not(target_arch = "wasm32")))]
compile_error!("browser feature is only available for wasm32 target arch");

pub use macros::rpc;

pub trait Transport<Req, Resp> {
    type Error: std::error::Error;
    fn send(self, request: Req) -> impl Future<Output = Result<Resp, LinkError<Self::Error>>>;
}

pub trait Rpc: Sync {
    type Client<T: Transport<Self::Request, Self::Response>>;
    type Request: Serialize + DeserializeOwned + Send;
    type Response: Serialize + DeserializeOwned + Send;
}

pub trait Handler {
    type Service: Rpc;
    fn handle(self, request: <Self::Service as Rpc>::Request) -> impl Future<Output = <Self::Service as Rpc>::Response> + Send;
}

#[derive(Debug, Error, Clone)]
pub enum LinkError<T: Error> {
    #[error("Failed to send request: {0}")]
    Transport(#[from] T),
    /// Response was the wrong type, sent a request for one function, but received the response of a different one
    ///
    /// This is not an expected case and is simply included as an alternative to panicking in this case
    /// This error either means the server side is misbehaving quite badly, or the transport is not configured to the correct endpoint
    #[error("Response was the wrong type, sent a request for one function, but received the response of a different one")]
    WrongResponseType,
}

#[derive(Debug)]
pub struct MappedTransport<T, InnerReq, OuterReq, InnerResp, OuterResp, Args> {
    outer: T,
    args: Args,
    to_inner: fn(OuterResp) -> Option<InnerResp>,
    to_outer: fn(Args, InnerReq) -> OuterReq
}

impl<T: Copy, InnerReq, OuterReq, InnerResp, OuterResp, Args: Copy> Copy for MappedTransport<T, InnerReq, OuterReq, InnerResp, OuterResp, Args> {

}

impl<T: Clone, InnerReq, OuterReq, InnerResp, OuterResp, Args: Clone> Clone for MappedTransport<T, InnerReq, OuterReq, InnerResp, OuterResp, Args> {
    fn clone(&self) -> Self {
        Self {
            outer: self.outer.clone(),
            args: self.args.clone(),
            to_inner: self.to_inner,
            to_outer: self.to_outer,
        }
    }
}

impl<T, InnerReq, OuterReq, InnerResp, OuterResp, Args> MappedTransport<T, InnerReq, OuterReq, InnerResp, OuterResp, Args> {
    pub fn new(inner: T, args: Args, to_inner: fn(OuterResp) -> Option<InnerResp>, to_outer: fn(Args, InnerReq) -> OuterReq) -> Self {
        Self {
            outer: inner,
            args,
            to_inner,
            to_outer,
        }
    }
}
impl<T, InnerReq, OuterReq, InnerResp, OuterResp, Args> Transport<InnerReq, InnerResp> for MappedTransport<T, InnerReq, OuterReq, InnerResp, OuterResp, Args>
where
    Args: Clone,
    T: Transport<OuterReq, OuterResp> {
    type Error = T::Error;
    async fn send(self, request: InnerReq) -> Result<InnerResp, LinkError<T::Error>> {
        let request = (self.to_outer)(self.args.clone(), request);
        let response = self.outer.send(request).await?;
        let response = (self.to_inner)(response).ok_or(LinkError::WrongResponseType)?;
        Ok(response)
    }
}
/*
pub struct FallibleMappedTransport<'a, T, InnerReq, OuterReq, InnerResp, OuterResp, Args, Error> {
    outer: &'a T,
    args: Args,
    to_inner: fn(OuterResp) -> Option<Result<InnerResp, Error>>,
    to_outer: fn(Args, InnerReq) -> OuterReq
}

impl<'a, T, InnerReq, OuterReq, InnerResp, OuterResp, Args, Error> FallibleMappedTransport<'a, T, InnerReq, OuterReq, InnerResp, OuterResp, Args, Error> {
    pub fn new(inner: &'a T, args: Args, to_inner: fn(OuterResp) -> Option<Result<InnerResp, Error>>, to_outer: fn(Args, InnerReq) -> OuterReq) -> Self {
        Self {
            outer: inner,
            args,
            to_inner,
            to_outer,
        }
    }
}
impl<T, InnerReq, OuterReq, InnerResp, OuterResp, Args, Error> Transport<InnerReq, Result<InnerResp, Error>> for FallibleMappedTransport<'_, T, InnerReq, OuterReq, InnerResp, OuterResp, Args, Error>
where
    Args: Clone,
    T: Transport<OuterReq, OuterResp> {
    type Error = T::Error;
    async fn send(&self, request: InnerReq) -> Result<Result<InnerResp, Error>, LinkError<T::Error>> {
        let request = (self.to_outer)(self.args.clone(), request);
        let response = self.outer.send(request).await?;
        let response = (self.to_inner)(response).ok_or(LinkError::WrongResponseType)?;
        Ok(response)
    }
}
*/