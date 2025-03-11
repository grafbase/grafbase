use std::sync::Arc;

use engine_schema::Subgraph;
use runtime::{
    error::{PartialErrorCode, PartialGraphqlError},
    extension::Data,
};
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::extension::{api::wit, pool::ExtensionGuard, runtime::Subscriptions};

use super::SubscriptionStream;

/// A subscription that deduplicates multiple identical subscription requests.
///
/// This struct manages GraphQL subscription streams by creating
/// a single underlying subscription and broadcasting the results to multiple clients.
///
/// The extension determines if and how to deduplicate the subscription requests.
pub struct DeduplicatedSubscription<'ctx, 'wit> {
    pub subscriptions: Subscriptions,
    pub instance: ExtensionGuard,
    pub headers: http::HeaderMap,
    pub key: Vec<u8>,
    pub subgraph: Subgraph<'ctx>,
    pub directive: wit::directive::FieldDefinitionDirective<'wit>,
}

impl<'ctx> DeduplicatedSubscription<'ctx, '_> {
    pub async fn resolve<'f>(self) -> Result<SubscriptionStream<'f>, PartialGraphqlError>
    where
        'ctx: 'f,
    {
        let DeduplicatedSubscription {
            subscriptions,
            mut instance,
            headers,
            key,
            subgraph,
            directive,
        } = self;

        let receiver = subscriptions.get(&key).as_ref().map(|channel| channel.subscribe());

        let (sender, receiver) = match receiver {
            Some(receiver) => {
                tracing::debug!("reuse existing channel");

                let stream = BroadcastStream::new(receiver).map(|result| match result {
                    Ok(data) => data,
                    Err(_) => Err(PartialGraphqlError::stream_lag()),
                });

                return Ok(Box::pin(stream));
            }
            None => {
                tracing::debug!("create new channel");
                broadcast::channel(1000)
            }
        };

        let sender = subscriptions.entry(key.clone()).or_insert(sender).to_owned();

        instance
            .resolve_subscription(headers, subgraph.name(), directive)
            .await
            .map_err(|err| err.into_graphql_error(PartialErrorCode::ExtensionError))?;

        tokio::spawn(async move {
            let mut registerations_closed = false;

            loop {
                let item = loop {
                    if registerations_closed && sender.receiver_count() == 0 {
                        return;
                    }

                    match instance.resolve_next_subscription_item().await {
                        Ok(Some(item)) if item.outputs.is_empty() => {
                            continue;
                        }
                        Ok(Some(item)) => {
                            tracing::debug!("subscription item resolved");

                            break item;
                        }
                        Ok(None) => {
                            tracing::debug!("subscription ended");
                            subscriptions.remove(&key);

                            return;
                        }
                        Err(err) => {
                            tracing::error!("subscription item error: {err}");
                            subscriptions.remove(&key);

                            return;
                        }
                    }
                };

                for item in item.outputs {
                    let data = match item {
                        Ok(item) => Ok(Arc::new(Data::CborBytes(item))),
                        Err(err) => Err(err.into_graphql_error(PartialErrorCode::InternalServerError)),
                    };

                    if sender.send(data).is_err() {
                        tracing::debug!("all subscribers are gone");
                        subscriptions.remove(&key);

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
            Err(_) => Err(PartialGraphqlError::stream_lag()),
        });

        Ok(Box::pin(stream))
    }
}
