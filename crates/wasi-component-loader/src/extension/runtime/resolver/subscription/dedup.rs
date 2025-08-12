use dashmap::Entry;
use engine::EngineOperationContext;
use engine_error::{ErrorCode, GraphqlError};
use runtime::extension::Response;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::{
    extension::{EngineWasmExtensions, ExtensionGuard, api::wit::SubscriptionItem},
    wasmsafe,
};

use super::SubscriptionStream;

/// A subscription that deduplicates multiple identical subscription requests.
///
/// This struct manages GraphQL subscription streams by creating
/// a single underlying subscription and broadcasting the results to multiple clients.
///
/// The extension determines if and how to deduplicate the subscription requests.
pub struct DeduplicatedSubscription {
    pub extensions: EngineWasmExtensions,
    pub key: Vec<u8>,
    pub instance: ExtensionGuard,
    pub context: EngineOperationContext,
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
                let next = wasmsafe!(instance.resolve_next_subscription_item(&context).await);

                let items = match next {
                    Ok(Some(item)) => match item {
                        SubscriptionItem::Single(item) => {
                            vec![item]
                        }
                        SubscriptionItem::Multiple(items) if items.is_empty() => continue,
                        SubscriptionItem::Multiple(items) => items,
                    },
                    Ok(None) => {
                        if let Err(err) = wasmsafe!(instance.drop_subscription(&context).await) {
                            tracing::error!("Error dropping subscription: {err}");
                        }
                        instance.recyclable = true;
                        if !client_registrations_closed {
                            extensions.subscriptions().remove(&key);
                        }
                        return;
                    }
                    Err(err) => {
                        if let Err(err) = wasmsafe!(instance.drop_subscription(&context).await) {
                            tracing::error!("Error dropping subscription: {err}");
                        }
                        instance.recyclable = true;
                        let _ = sender.send(Response::error(err));
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
                            if let Err(err) = wasmsafe!(instance.drop_subscription(&context).await) {
                                tracing::error!("Error dropping subscription: {err}");
                            }
                            instance.recyclable = true;
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
