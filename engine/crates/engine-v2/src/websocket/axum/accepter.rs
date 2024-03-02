use std::{collections::HashMap, sync::Arc};

use ::axum::extract::ws::{self, WebSocket};
use engine_v2_common::BatchGraphqlRequest;
use futures::Stream;
use futures_util::{pin_mut, stream::SplitStream, SinkExt, StreamExt};
use runtime::auth::AccessToken;
use tokio::sync::{mpsc, watch};

use super::service::MessageConvert;
use crate::{
    response::Response,
    websocket::messages::{Event, Message},
    Engine,
};

pub type EngineWatcher = watch::Receiver<Option<Arc<Engine>>>;
pub type WebsocketSender = tokio::sync::mpsc::Sender<WebSocket>;
pub type WebsocketReceiver = tokio::sync::mpsc::Receiver<WebSocket>;

const CONNECTION_INIT_WAIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

/// An actor that manages websocket connections for federated dev
pub struct WebsocketAccepter {
    sockets: WebsocketReceiver,
    engine: EngineWatcher,
}

impl WebsocketAccepter {
    pub fn new(sockets: WebsocketReceiver, engine: EngineWatcher) -> Self {
        Self { sockets, engine }
    }

    pub async fn handler(mut self) {
        while let Some(mut connection) = self.sockets.recv().await {
            let engine = self.engine.clone();

            tokio::spawn(async move {
                let accept_future =
                    tokio::time::timeout(CONNECTION_INIT_WAIT_TIMEOUT, accept_websocket(&mut connection, &engine));

                match accept_future.await {
                    Ok(Some(session)) => websocket_loop(connection, session).await,
                    Ok(None) => {
                        tracing::warn!("Failed to accept websocket connection");
                    }
                    Err(_) => {
                        tracing::info!("Connection wasn't initialised on time, dropping");
                        connection
                            .send(
                                Message::close(4408, "Connection initialisation timeout")
                                    .to_axum_message()
                                    .unwrap(),
                            )
                            .await
                            .ok();
                    }
                }
            });
        }
    }
}

/// Message handling loop for a single websocket connection
async fn websocket_loop(socket: WebSocket, session: WebsocketSession) {
    let (sender, mut receiver) = {
        let (mut socket_sender, socket_receiver) = socket.split();

        // The WebSocket sender isn't clone, so we switch it for an mpsc and
        // spawn a message pumping task to hook up the mpsc & the sender.
        let (message_sender, mut message_receiver) = mpsc::channel::<Message>(16);
        tokio::spawn(async move {
            while let Some(message) = message_receiver.recv().await {
                let message = match message.to_axum_message() {
                    Ok(message) => message,
                    Err(error) => {
                        tracing::warn!("Couldn't encode websocket message: {error:?}");
                        return;
                    }
                };

                if socket_sender.send(message).await.is_err() {
                    break;
                }
            }
        });

        (message_sender, socket_receiver)
    };

    let mut tasks = tokio::task::JoinSet::new();
    let mut subscriptions = HashMap::new();

    while let Some(bytes) = receiver.recv_message().await {
        let response = handle_incoming_event(bytes, &session, &sender, &mut tasks, &mut subscriptions).await;
        match response {
            None => {}
            Some(message @ Message::Close { .. }) => {
                sender.send(message).await.ok();
                return;
            }
            Some(message) => {
                if sender.send(message).await.is_err() {
                    return;
                }
            }
        }
    }
}

async fn handle_incoming_event(
    bytes: Vec<u8>,
    session: &WebsocketSession,
    sender: &tokio::sync::mpsc::Sender<Message>,
    tasks: &mut tokio::task::JoinSet<()>,
    subscriptions: &mut HashMap<String, tokio::task::AbortHandle>,
) -> Option<Message> {
    let event: Event<'_> = serde_json::from_slice(&bytes).ok()?;
    match event {
        Event::Subscribe { id, payload } => {
            if subscriptions.contains_key(&id) {
                return Some(Message::close(4409, format!("Subscriber for {id} already exists")));
            }

            let BatchGraphqlRequest::Single(request) = *payload else {
                return Some(Message::close(4409, "Batch requests not supported"));
            };

            let handle = tasks.spawn({
                let session = session.clone();
                let sender = sender.clone();
                let id = id.clone();
                let stream = session
                    .engine
                    .execute_stream(session.headers, session.access_token, "", request)
                    .await;
                async move { subscription_loop(stream, id, sender).await }
            });
            subscriptions.insert(id, handle);

            None
        }
        Event::Complete { id } => {
            if let Some(handle) = subscriptions.remove(&id) {
                handle.abort();
            }
            None
        }
        Event::Pong { .. } => None,
        Event::Ping { .. } => Some(Message::Pong { payload: None }),
        Event::ConnectionInit { .. } => Some(Message::Close {
            code: 4429,
            reason: "Too many initialisation requests".into(),
        }),
    }
}

async fn subscription_loop(stream: impl Stream<Item = Response>, id: String, sender: mpsc::Sender<Message>) {
    pin_mut!(stream);
    while let Some(response) = stream.next().await {
        if matches!(response, Response::BadRequest(_)) {
            sender
                .send(Message::Error {
                    id: id.clone(),
                    payload: response,
                })
                .await
                .ok();

            return;
        }

        let result = sender
            .send(Message::Next {
                id: id.clone(),
                payload: response,
            })
            .await;

        if result.is_err() {
            // No point continuing if the sender is dead
            return;
        }
    }
    sender.send(Message::Complete { id }).await.ok();
}

async fn accept_websocket(websocket: &mut WebSocket, engine: &EngineWatcher) -> Option<WebsocketSession> {
    while let Some(bytes) = websocket.recv_message().await {
        let event: Event<'_> = serde_json::from_slice(&bytes).ok()?;
        match event {
            Event::ConnectionInit { payload } => {
                let Some(engine) = engine.borrow().clone() else {
                    websocket
                        .send(
                            Message::close(4995, "register a subgraph before connecting")
                                .to_axum_message()
                                .unwrap(),
                        )
                        .await
                        .ok();
                    return None;
                };

                let Some(access_token) = engine.auth.authorize(&payload.headers).await else {
                    websocket
                        .send(Message::close(4403, "Forbidden").to_axum_message().unwrap())
                        .await
                        .ok();
                    return None;
                };

                websocket
                    .send(Message::ConnectionAck { payload: None }.to_axum_message().unwrap())
                    .await
                    .ok()?;

                return Some(WebsocketSession {
                    engine,
                    access_token: Arc::new(access_token),
                    headers: Arc::new(payload.headers),
                });
            }
            Event::Ping { .. } => {
                websocket
                    .send(
                        Message::Ping { payload: None }
                            .to_axum_message()
                            .expect("ping should always be serializable"),
                    )
                    .await
                    .ok()?;
            }
            Event::Subscribe { .. } => {
                websocket
                    .send(Message::close(4401, "Unauthorized").to_axum_message().unwrap())
                    .await
                    .ok();
                return None;
            }
            _ => {}
        }
    }

    None
}

trait WebsocketExt {
    async fn recv(&mut self) -> Option<Result<ws::Message, ::axum::Error>>;

    async fn recv_message(&mut self) -> Option<Vec<u8>> {
        while let Some(message) = self.recv().await {
            return match message {
                Ok(ws::Message::Ping(_) | ws::Message::Pong(_)) => continue,
                Ok(ws::Message::Close(_)) => None,
                Ok(ws::Message::Text(contents)) => Some(contents.into_bytes()),
                Ok(ws::Message::Binary(contents)) => Some(contents),
                Err(error) => {
                    tracing::warn!("Error receiving websocket message: {error:?}");
                    None
                }
            };
        }
        None
    }
}

impl WebsocketExt for WebSocket {
    async fn recv(&mut self) -> Option<Result<ws::Message, ::axum::Error>> {
        WebSocket::recv(self).await
    }
}

impl WebsocketExt for SplitStream<WebSocket> {
    async fn recv(&mut self) -> Option<Result<ws::Message, ::axum::Error>> {
        self.next().await
    }
}

#[derive(Clone)]
pub struct WebsocketSession {
    engine: Arc<Engine>,
    access_token: Arc<AccessToken>,
    headers: Arc<http::HeaderMap>,
}
