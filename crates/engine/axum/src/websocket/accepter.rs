use std::{collections::HashMap, sync::Arc};

use ::axum::extract::ws::{self, WebSocket};
use engine::{Engine, Runtime, WebsocketSession};
use futures_util::{SinkExt, Stream, StreamExt, pin_mut, stream::SplitStream};
use tokio::sync::{mpsc, watch};

use super::{WebsocketReceiver, WebsocketRequest, service::MessageConvert};
use engine::websocket::{Event, Message};

pub type EngineWatcher<R> = watch::Receiver<Arc<Engine<R>>>;

const CONNECTION_INIT_WAIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

/// An actor that manages websocket connections for federated dev
pub struct WebsocketAccepter<R: Runtime> {
    sockets: WebsocketReceiver,
    engine: EngineWatcher<R>,
}

impl<R: Runtime> WebsocketAccepter<R> {
    pub fn new(sockets: WebsocketReceiver, engine: EngineWatcher<R>) -> Self {
        Self { sockets, engine }
    }

    pub async fn handler(mut self) {
        while let Some(WebsocketRequest { mut websocket, parts }) = self.sockets.recv().await {
            let engine = self.engine.clone();

            tokio::spawn(async move {
                let accept_future = tokio::time::timeout(
                    CONNECTION_INIT_WAIT_TIMEOUT,
                    accept_websocket(parts, &mut websocket, &engine),
                );

                match accept_future.await {
                    Ok(Some(session)) => websocket_loop(websocket, session).await,
                    Ok(None) => {
                        tracing::warn!("Failed to accept websocket connection");
                    }
                    Err(_) => {
                        tracing::info!("Connection wasn't initialised on time, dropping");
                        websocket
                            .send(
                                Message::<R>::close(4408, "Connection initialisation timeout")
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
async fn websocket_loop<R: Runtime>(websocket: WebSocket, session: WebsocketSession<R>) {
    let (sender, mut receiver) = {
        let (mut socket_sender, socket_receiver) = websocket.split();

        // The WebSocket sender isn't clone, so we switch it for an mpsc and
        // spawn a message pumping task to hook up the mpsc & the sender.
        let (message_sender, mut message_receiver) = mpsc::channel::<Message<R>>(16);
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

    while let Some(text) = receiver.recv_message().await {
        let response = handle_incoming_event(text, &session, &sender, &mut tasks, &mut subscriptions).await;
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

async fn handle_incoming_event<R: Runtime>(
    text: String,
    session: &WebsocketSession<R>,
    sender: &tokio::sync::mpsc::Sender<Message<R>>,
    tasks: &mut tokio::task::JoinSet<()>,
    subscriptions: &mut HashMap<String, tokio::task::AbortHandle>,
) -> Option<Message<R>> {
    let event: Event = sonic_rs::from_str(&text).ok()?;
    match event {
        Event::Subscribe(event) => {
            if subscriptions.contains_key(&event.id) {
                return Some(Message::close(
                    4409,
                    format!("Subscriber for {} already exists", event.id),
                ));
            }

            let id = event.id.clone();
            let stream = session.execute(event);
            let handle = tasks.spawn(subscription_loop(stream, id.clone(), sender.clone()));
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

async fn subscription_loop<R: engine::Runtime>(
    stream: impl Stream<Item = Message<R>>,
    id: String,
    sender: mpsc::Sender<Message<R>>,
) {
    pin_mut!(stream);
    while let Some(message) = stream.next().await {
        if sender.send(message).await.is_err() {
            // No point continuing if the sender is dead
            return;
        }
    }
    sender.send(Message::Complete { id }).await.ok();
}

async fn accept_websocket<R: Runtime>(
    parts: http::request::Parts,
    websocket: &mut WebSocket,
    engine: &EngineWatcher<R>,
) -> Option<WebsocketSession<R>> {
    while let Some(text) = websocket.recv_message().await {
        let event: Event = sonic_rs::from_str(&text).ok()?;
        match event {
            Event::ConnectionInit { payload } => {
                let engine = engine.borrow().clone();

                let Ok(session) = engine.create_websocket_session(parts, payload).await else {
                    websocket
                        .send(Message::<R>::close(4403, "Forbidden").to_axum_message().unwrap())
                        .await
                        .ok();
                    return None;
                };

                websocket
                    .send(Message::<R>::ConnectionAck { payload: None }.to_axum_message().unwrap())
                    .await
                    .ok()?;

                return Some(session);
            }
            Event::Ping { .. } => {
                websocket
                    .send(
                        Message::<R>::Ping { payload: None }
                            .to_axum_message()
                            .expect("ping should always be serializable"),
                    )
                    .await
                    .ok()?;
            }
            Event::Subscribe { .. } => {
                websocket
                    .send(Message::<R>::close(4401, "Unauthorized").to_axum_message().unwrap())
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

    async fn recv_message(&mut self) -> Option<String> {
        while let Some(message) = self.recv().await {
            return match message {
                Ok(ws::Message::Ping(_) | ws::Message::Pong(_)) => continue,
                Ok(ws::Message::Close(_)) => None,
                Ok(ws::Message::Text(contents)) => Some(contents.to_string()),
                Ok(ws::Message::Binary(contents)) => String::from_utf8(contents.into()).ok(),
                Err(error) => {
                    tracing::debug!("Error receiving websocket message: {error:?}");
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
