use std::collections::HashMap;

use ::axum::extract::ws::{self, WebSocket};
use futures_util::{pin_mut, stream::SplitStream, SinkExt, StreamExt};
use gateway_v2::Session;
use tokio::sync::mpsc;

use self::messages::{Event, Message};

use super::bus::GatewayWatcher;

mod axum;
mod messages;

pub use axum::WebsocketService;

pub type WebsocketSender = tokio::sync::mpsc::Sender<WebSocket>;
pub type WebsocketReceiver = tokio::sync::mpsc::Receiver<WebSocket>;

/// An actor that manages websocket connections for federated dev
pub(crate) struct WebsocketAccepter {
    sockets: WebsocketReceiver,
    gateway: GatewayWatcher,
}

impl WebsocketAccepter {
    pub fn new(sockets: WebsocketReceiver, gateway: GatewayWatcher) -> Self {
        Self { sockets, gateway }
    }

    pub async fn handler(mut self) {
        while let Some(connection) = self.sockets.recv().await {
            let gateway = self.gateway.clone();

            tokio::spawn(async move {
                let Some((websocket, session)) = accept_websocket(connection, &gateway).await else {
                    log::warn!("Failed to accept websocket connection");
                    return;
                };

                websocket_loop(websocket, session).await;
            });
        }
    }
}

/// Message handling loop for a single websocket connection
async fn websocket_loop(socket: WebSocket, session: Session) {
    let (sender, mut receiver) = {
        let (mut socket_sender, socket_receiver) = socket.split();

        // The WebSocket sender isn't clone, so we switch it for an mpsc and
        // spawn a message pumping task to hook up the mpsc & the sender.
        let (message_sender, mut message_receiver) = mpsc::channel(16);
        tokio::spawn(async move {
            while let Some(message) = message_receiver.recv().await {
                let message = match ws::Message::try_from(message) {
                    Ok(message) => message,
                    Err(error) => {
                        log::warn!("Couldn't encode websocket message: {error:?}");
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

    while let Some(event) = receiver.recv_graphql().await {
        let response = handle_incoming_event(event, &session, &sender, &mut tasks, &mut subscriptions).await;
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
    event: Event,
    session: &Session,
    sender: &tokio::sync::mpsc::Sender<Message>,
    tasks: &mut tokio::task::JoinSet<()>,
    subscriptions: &mut HashMap<String, tokio::task::AbortHandle>,
) -> Option<Message> {
    match event {
        Event::Subscribe { id, payload } => {
            if subscriptions.contains_key(&id) {
                return Some(Message::close(4409, format!("Subscriber for {id} already exists")));
            }

            let handle = tasks.spawn(subscription_loop(session.clone(), payload, id.clone(), sender.clone()));
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

async fn subscription_loop(
    session: gateway_v2::Session,
    request: engine::Request,
    id: String,
    sender: mpsc::Sender<Message>,
) {
    let stream = session.execute_stream(request);

    pin_mut!(stream);
    while let Some(response) = stream.next().await {
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

async fn accept_websocket(mut websocket: WebSocket, gateway: &GatewayWatcher) -> Option<(WebSocket, Session)> {
    while let Some(event) = websocket.recv_graphql().await {
        match event {
            Event::ConnectionInit { payload } => {
                let Some(gateway) = gateway.borrow().clone() else {
                    websocket
                        .send(Message::close(4403, "Forbidden").try_into().unwrap())
                        .await
                        .ok();
                    return None;
                };

                let Ok(session) = gateway.authorize(payload.headers.into()).await else {
                    websocket
                        .send(Message::close(4403, "Forbidden").try_into().unwrap())
                        .await
                        .ok();
                    return None;
                };

                websocket
                    .send(Message::ConnectionAck { payload: None }.try_into().unwrap())
                    .await
                    .ok()?;

                return Some((websocket, session));
            }
            Event::Ping { .. } => {
                websocket
                    .send(
                        Message::Ping { payload: None }
                            .try_into()
                            .expect("ping should always be serializable"),
                    )
                    .await
                    .ok()?;
            }
            _ => {}
        }
    }

    None
}

trait WebsocketExt {
    async fn recv(&mut self) -> Option<Result<ws::Message, ::axum::Error>>;

    async fn recv_graphql(&mut self) -> Option<Event> {
        while let Some(message) = self.recv().await {
            let event = match message {
                Ok(ws::Message::Ping(_) | ws::Message::Pong(_)) => continue,
                Ok(ws::Message::Close(_)) => return None,
                Ok(ws::Message::Text(contents)) => serde_json::from_str::<Event>(&contents),
                Ok(ws::Message::Binary(contents)) => serde_json::from_slice::<Event>(&contents),
                Err(error) => {
                    log::warn!("Error receiving websocket message: {error:?}");
                    return None;
                }
            };

            return event
                .map_err(|error| {
                    log::warn!("error decoding websocket message: {error:?}");
                    error
                })
                .ok();
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
