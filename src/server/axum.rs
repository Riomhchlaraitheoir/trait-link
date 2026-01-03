use crate::{Handler, Rpc};
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use futures::FutureExt;
use std::sync::Arc;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Serves the service, accepting JSON requests and responding with JSON
pub fn json<S: Handler + Copy>(
    State(server): State<Arc<S>>,
    Json(request): Json<<S::Service as Rpc>::Request>,
) -> impl Future<Output = impl IntoResponse> + Send
where <S::Service as Rpc>::Request: DeserializeOwned, 
      <S::Service as Rpc>::Response: Serialize
{
    server
        .handle(request)
        .map(|response| Json(response).into_response())
}
