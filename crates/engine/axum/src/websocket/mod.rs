mod accepter;
mod service;

pub use accepter::*;
use axum::extract::ws::WebSocket;
pub use service::*;

pub type WebsocketSender = tokio::sync::mpsc::Sender<WebsocketRequest>;
pub type WebsocketReceiver = tokio::sync::mpsc::Receiver<WebsocketRequest>;

pub struct WebsocketRequest {
    websocket: WebSocket,
    parts: http::request::Parts,
}
