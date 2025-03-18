use std::sync::Arc;

use engine::{ErrorCode, GraphqlError};
use engine_schema::Subgraph;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::extension::{ExtensionGuard, WasmExtensions, api::wit};

use super::SubscriptionStream;

/// A subscription that deduplicates multiple identical subscription requests.
///
/// This struct manages GraphQL subscription streams by creating
/// a single underlying subscription and broadcasting the results to multiple clients.
///
/// The extension determines if and how to deduplicate the subscription requests.
pub struct DeduplicatedSubscription<'ctx, 'wit> {
    pub extensions: WasmExtensions,
    pub instance: ExtensionGuard,
    pub headers: http::HeaderMap,
    pub key: Vec<u8>,
    pub subgraph: Subgraph<'ctx>,
    pub directive: wit::FieldDefinitionDirective<'wit>,
}

impl<'ctx> DeduplicatedSubscription<'ctx, '_> {
    pub async fn resolve<'f>(self) -> Result<SubscriptionStream<'f>, GraphqlError>
    where
        'ctx: 'f,
    {
        let DeduplicatedSubscription {
            extensions,
            mut instance,
            headers,
            key,
            subgraph,
            directive,
        } = self;

        let receiver = extensions
            .subscriptions()
            .get(&key)
            .as_ref()
            .map(|channel| channel.subscribe());

        let (sender, receiver) = match receiver {
            Some(receiver) => {
                tracing::debug!("reuse existing channel");

                let stream = BroadcastStream::new(receiver).map(|result| match result {
                    Ok(data) => data,
                    Err(_) => Err(stream_lag_error()),
                });

                return Ok(Box::pin(stream));
            }
            None => {
                tracing::debug!("create new channel");
                broadcast::channel(1000)
            }
        };

        let sender = extensions
            .subscriptions()
            .entry(key.clone())
            .or_insert(sender)
            .to_owned();

        instance
            .resolve_subscription(headers, subgraph.name(), directive)
            .await
            .map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError))?;

        tokio::spawn(async move {
            let mut registerations_closed = false;

            loop {
                let items = loop {
                    if registerations_closed && sender.receiver_count() == 0 {
                        return;
                    }

                    match instance.resolve_next_subscription_item().await {
                        Ok(Some(items)) if items.is_empty() => {
                            continue;
                        }
                        Ok(Some(items)) => {
                            tracing::debug!("subscription item resolved");

                            break items;
                        }
                        Ok(None) => {
                            tracing::debug!("subscription ended");
                            extensions.subscriptions().remove(&key);

                            return;
                        }
                        Err(err) => {
                            tracing::error!("subscription item error: {err}");
                            extensions.subscriptions().remove(&key);

                            return;
                        }
                    }
                };

                for item in items {
                    let data = match item {
                        Ok(data) => Ok(Arc::new(data)),
                        Err(err) => Err(err),
                    };

                    if sender.send(data).is_err() {
                        tracing::debug!("all subscribers are gone");
                        extensions.subscriptions().remove(&key);

                        if registerations_closed {
                            return;
                        }

                        registerations_closed = true;

                        break;
                    }
                }
            }
        });

        let stream = BroadcastStream::new(receiver).map(|result| match result {
            Ok(data) => data,
            Err(_) => Err(stream_lag_error()),
        });

        Ok(Box::pin(stream))
    }
}

fn stream_lag_error() -> GraphqlError {
    GraphqlError::new(
        "The stream is lagging behind due to not being able to keep up with the data. Events are being dropped.",
        ErrorCode::ExtensionError,
    )
}
