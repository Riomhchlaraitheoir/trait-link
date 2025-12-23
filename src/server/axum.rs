use crate::{Handler, Rpc};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;
use futures::FutureExt;

pub fn serve<S: Handler + Copy>(State(server): State<Arc<S>>, Json(request): Json<<S::Service as Rpc>::Request>) -> impl Future<Output = impl IntoResponse> + Send {
    server.handle(request).map(|response| Json(response).into_response())
}
