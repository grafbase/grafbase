use std::collections::VecDeque;

use super::SubscriptionStream;
use crate::{
    Error, SharedContext,
    extension::{ExtensionGuard, api::wit::SubscriptionItem},
};
use engine_error::{ErrorCode, GraphqlError};
use futures::{StreamExt as _, stream};
use runtime::extension::Response;

/// A subscription without deduplication, reserving one extension instance for each subscription.
///
/// The system uses this when the extension does not define a deduplication key.
pub struct UniqueSubscription {
    pub instance: ExtensionGuard,
}

impl UniqueSubscription {
    pub async fn resolve(self, context: SharedContext) -> SubscriptionStream<'static> {
        let UniqueSubscription { instance } = self;

        stream::unfold(
            (Some(instance), VecDeque::<Response>::new(), context),
            async move |(instance, mut queue, context)| {
                let mut instance = instance?;

                if let Some(response) = queue.pop_front() {
                    return Some((response, (Some(instance), queue, context)));
                }

                loop {
                    let next =
                        instance
                            .resolve_next_subscription_item(context.clone())
                            .await
                            .map_err(|err| match err {
                                Error::Internal(err) => {
                                    tracing::error!("Wasm error: {err}");
                                    GraphqlError::new("Internal error", ErrorCode::ExtensionError)
                                }
                                Error::Guest(err) => err.into_graphql_error(ErrorCode::ExtensionError),
                            });

                    match next {
                        Ok(Ok(Some(item))) => match item {
                            SubscriptionItem::Single(resp) => {
                                return Some((resp.into(), (Some(instance), queue, context)));
                            }
                            SubscriptionItem::Multiple(items) if items.is_empty() => continue,
                            SubscriptionItem::Multiple(items) => {
                                queue.extend(items.into_iter().map(Into::into));
                                return queue.pop_front().map(|resp| (resp, (Some(instance), queue, context)));
                            }
                        },
                        Ok(Ok(None)) => {
                            if let Err(err) = instance.drop_subscription(context.clone()).await {
                                tracing::error!("Error dropping subscription: {err}");
                            }
                            return None;
                        }
                        Ok(Err(err)) => {
                            if let Err(err) = instance.drop_subscription(context.clone()).await {
                                tracing::error!("Error dropping subscription: {err}");
                            }
                            return Some((Response::error(err), (None, queue, context)));
                        }
                        Err(err) => {
                            tracing::error!("Error resolving subscription item: {err}");
                            if let Err(err) = instance.drop_subscription(context.clone()).await {
                                tracing::error!("Error dropping subscription: {err}");
                            }
                            return Some((
                                Response::error(GraphqlError::internal_extension_error()),
                                (None, queue, context),
                            ));
                        }
                    }
                }
            },
        )
        .boxed()
    }
}
