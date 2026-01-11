//! Defines a websocket client

use crate::client::AsyncClient;
use crate::format::Format;
use crate::{RpcError, get_request_id, prepend_id};
use futures::channel::{mpsc, oneshot};
use futures::lock::Mutex;
use futures::{FutureExt, SinkExt, StreamExt, select};
use std::collections::HashMap;
use std::error::Error;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use thiserror::Error;
use tracing::{error, warn};
use wasm_bindgen_futures::spawn_local;
use ws_stream_wasm::{CloseEvent, WsErr, WsMessage, WsMeta};

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

/// A client which communicates using a websocket connection
pub struct WebsocketClient<Req, Resp> {
    sender: Arc<Mutex<mpsc::Sender<(u32, Req)>>>,
    senders: SenderMap<Resp>,
}

type SenderMap<Resp> = Arc<Mutex<HashMap<u32, oneshot::Sender<Result<Resp, WebsocketError>>>>>;

impl<Req, Resp> Clone for WebsocketClient<Req, Resp> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            senders: self.senders.clone(),
        }
    }
}

impl<Req: Send + 'static, Resp: Send + 'static> WebsocketClient<Req, Resp> {
    /// Create a new websocket client
    ///
    /// # Errors
    /// Returns an error if the websocket connection could not be opened
    ///
    /// # Panics
    /// Certain unexpected edge cases that cannot be proven safe with the type system may cause a panic
    pub async fn new(
        url: impl AsRef<str>,
        format: impl Format<Resp, Req> + 'static,
    ) -> Result<Self, WsErr> {
        let (meta, mut stream) = WsMeta::connect(url, Some(vec![format.content_type()])).await?;
        let (sender, mut request_receiver) = mpsc::channel(100);
        let sender = Arc::new(Mutex::new(sender));
        let senders: SenderMap<Resp> = Arc::default();
        spawn_local({
            let response_senders = senders.clone();
            async move {
                let closed: bool = 'worker: loop {
                    select! {
                                        req = request_receiver.next() => {
                                            let Some((request_id, request)) = req else {
                                                continue 'worker;
                                            };
                                            let request = match format.write(request) {
                                                Ok(request) => request,
                                                Err(error) => {
                                                    let Some(response) = response_senders.lock().await.remove(&request_id) else {
                                                        error!("Response sender was not loaded prior to request");
                                                        continue 'worker;
                                                    };
                                                    // will only fail if the future is dropped, this case is not considered an error and can safely be ignored
                                                    let _ = response.send(Err(WebsocketError::SerialiseRequest(error)));
                                                    continue 'worker;
                                                }
                                            };
                                            let request = prepend_id(request_id, request);
                                            if let Err(error) = stream.send(WsMessage::Binary(request)).await {
                                                warn!("Error sending message: {}", error);
                                                break 'worker false;
                                            }
                                        },
                                        response = stream.next().fuse() => {
                                                let response = if let Some(message) = response {
                                                        match message {
                                                            WsMessage::Text(error) => {
                                                                warn!("Error from server: {}", error);
                                                                continue 'worker;
                                                            }
                                                            WsMessage::Binary(response) => response,
                                                        }
                                                } else {
                                                    warn!("websocket closed");
                                                    break 'worker false;
                                                };
                                                let (request_id, response) = get_request_id(&response);
                                                let sender = response_senders.lock().await.remove(&request_id).expect("sender not found");
                                                let response = match format.read(response) {
                                                    Ok(response) => response,
                                                    Err(error) => {
                                                        let _: Result<(), _> = sender.send(Err(WebsocketError::DeserialiseResponse(error)));
                                                        continue 'worker;
                                                    }
                                                };
                                                let _: Result<(), _> = sender.send(Ok(response));
                                        }
                                        }
                };
                if !closed {
                    let _: Result<CloseEvent, _> = meta.close().await;
                }
                let senders = mem::take(&mut *response_senders.lock().await);
                for (_, sender) in senders {
                    let _ = sender.send(Err(WebsocketError::ConnectionClosed));
                }
            }
        });
        Ok(Self { sender, senders })
    }
}

impl<Req, Resp> AsyncClient<Req, Resp> for WebsocketClient<Req, Resp> {
    type Error = RpcError<WebsocketError>;

    async fn send(&self, request: Req) -> Result<Resp, Self::Error> {
        let (sender, receiver) = oneshot::channel();
        let request_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        self.senders.lock().await.insert(request_id, sender);
        self.sender
            .lock()
            .await
            .send((request_id, request))
            .await
            .map_err(|_| RpcError::Transport(WebsocketError::RequestChannelClosed))?;
        receiver
            .await
            .map_err(|_| RpcError::Transport(WebsocketError::ResponseChannelClosed))?
            .map_err(RpcError::Transport)
    }
}

/// An error from the websocket client
#[derive(Debug, Error)]
pub enum WebsocketError {
    /// The websocket worker has closed the request channel, this is not expected
    #[error("Failed to send request to worker: channel closed")]
    RequestChannelClosed,
    /// The websocket worker has closed the response channel, this is not expected
    #[error("Failed to read response from worker: channel closed")]
    ResponseChannelClosed,
    /// The request could not be serialised
    #[error("Failed to write request: {0}")]
    SerialiseRequest(Box<dyn Error + Send>),
    /// The response could not be deserialised
    #[error("Failed to write request: {0}")]
    DeserialiseResponse(Box<dyn Error + Send>),
    /// The websocket connection has closed
    #[error("Websocket connection closed")]
    ConnectionClosed,
}
