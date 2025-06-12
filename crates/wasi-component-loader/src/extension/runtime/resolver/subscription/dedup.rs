use dashmap::Entry;
use engine_error::{ErrorCode, GraphqlError};
use runtime::extension::Response;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::{
    Error, SharedContext,
    extension::{ExtensionGuard, WasmExtensions, api::wit::SubscriptionItem},
};

use super::SubscriptionStream;

/// A subscription that deduplicates multiple identical subscription requests.
///
/// This struct manages GraphQL subscription streams by creating
/// a single underlying subscription and broadcasting the results to multiple clients.
///
/// The extension determines if and how to deduplicate the subscription requests.
pub struct DeduplicatedSubscription {
    pub extensions: WasmExtensions,
    pub key: Vec<u8>,
    pub instance: ExtensionGuard,
    pub context: SharedContext,
}

impl DeduplicatedSubscription {
    pub async fn resolve(self) -> SubscriptionStream<'static> {
        let DeduplicatedSubscription {
            extensions,
            key,
            mut instance,
            context,
        } = self;

        let (sender, receiver) = match extensions.subscriptions().entry(key.clone()) {
            Entry::Occupied(occupied_entry) => {
                tracing::debug!("reuse existing channel");

                let receiver = occupied_entry.get().subscribe();

                return Box::pin(BroadcastStream::new(receiver).map(|result| match result {
                    Ok(resp) => resp,
                    Err(_) => Response::error(stream_lag_error()),
                }));
            }
            Entry::Vacant(vacant_entry) => {
                tracing::debug!("create new channel");

                let (sender, receiver) = broadcast::channel(1000);
                let sender = vacant_entry.insert(sender).to_owned();

                (sender, receiver)
            }
        };

        tokio::spawn(async move {
            let mut client_registrations_closed = false;

            loop {
                let next = instance
                    .resolve_next_subscription_item(context.clone())
                    .await
                    .map_err(|err| match err {
                        Error::Internal(err) => {
                            tracing::error!("Wasm error: {err}");
                            GraphqlError::new("Internal error", ErrorCode::ExtensionError)
                        }
                        Error::Guest(err) => err.into_graphql_error(ErrorCode::ExtensionError),
                    });

                let items = match next {
                    Ok(Ok(Some(item))) => match item {
                        SubscriptionItem::Single(item) => {
                            vec![item]
                        }
                        SubscriptionItem::Multiple(items) if items.is_empty() => continue,
                        SubscriptionItem::Multiple(items) => items,
                    },
                    Ok(Ok(None)) => {
                        if let Err(err) = instance.drop_subscription(context).await {
                            tracing::error!("Error dropping subscription: {err}");
                        }
                        if !client_registrations_closed {
                            extensions.subscriptions().remove(&key);
                        }
                        return;
                    }
                    Ok(Err(err)) => {
                        if let Err(err) = instance.drop_subscription(context).await {
                            tracing::error!("Error dropping subscription: {err}");
                        }
                        let _ = sender.send(Response::error(err));
                        if !client_registrations_closed {
                            extensions.subscriptions().remove(&key);
                        }
                        return;
                    }
                    Err(err) => {
                        tracing::error!("Error resolving subscription item: {err}");
                        if let Err(err) = instance.drop_subscription(context).await {
                            tracing::error!("Error dropping subscription: {err}");
                        }
                        let _ = sender.send(Response::error(GraphqlError::internal_extension_error()));
                        if !client_registrations_closed {
                            extensions.subscriptions().remove(&key);
                        }
                        return;
                    }
                };

                for item in items {
                    if sender.send(item.into()).is_err() {
                        tracing::debug!("all subscribers are gone");
                        if client_registrations_closed {
                            if let Err(err) = instance.drop_subscription(context.clone()).await {
                                tracing::error!("Error dropping subscription: {err}");
                            }
                            return;
                        } else {
                            extensions.subscriptions().remove(&key);
                            client_registrations_closed = true;
                        }
                    }
                }
            }
        });

        Box::pin(BroadcastStream::new(receiver).map(|result| match result {
            Ok(resp) => resp,
            Err(_) => Response::error(stream_lag_error()),
        }))
    }
}

fn stream_lag_error() -> GraphqlError {
    GraphqlError::new(
        "The stream is lagging behind due to not being able to keep up with the data. Events are being dropped.",
        ErrorCode::ExtensionError,
    )
}
