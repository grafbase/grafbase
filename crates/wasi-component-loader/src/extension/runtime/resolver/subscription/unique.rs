use std::collections::VecDeque;

use super::SubscriptionStream;
use crate::{
    WasmContext,
    extension::{ExtensionGuard, api::wit::SubscriptionItem},
    wasmsafe,
};
use futures::{StreamExt as _, stream};
use runtime::extension::Response;

/// A subscription without deduplication, reserving one extension instance for each subscription.
///
/// The system uses this when the extension does not define a deduplication key.
pub struct UniqueSubscription {
    pub instance: ExtensionGuard,
}

impl UniqueSubscription {
    pub async fn resolve(self, context: WasmContext) -> SubscriptionStream<'static> {
        let UniqueSubscription { instance } = self;

        stream::unfold(
            (Some(instance), VecDeque::<Response>::new(), context),
            async move |(instance, mut queue, context)| {
                let mut instance = instance?;

                if let Some(response) = queue.pop_front() {
                    return Some((response, (Some(instance), queue, context)));
                }

                loop {
                    let next = wasmsafe!(instance.resolve_next_subscription_item(context.clone()).await);

                    match next {
                        Ok(Some(item)) => match item {
                            SubscriptionItem::Single(resp) => {
                                return Some((resp.into(), (Some(instance), queue, context)));
                            }
                            SubscriptionItem::Multiple(items) if items.is_empty() => continue,
                            SubscriptionItem::Multiple(items) => {
                                queue.extend(items.into_iter().map(Into::into));
                                return queue.pop_front().map(|resp| (resp, (Some(instance), queue, context)));
                            }
                        },
                        Ok(None) => {
                            if let Err(err) = wasmsafe!(instance.drop_subscription(context.clone()).await) {
                                tracing::error!("Error dropping subscription: {err}");
                            }
                            instance.recyclable = true;
                            return None;
                        }
                        Err(err) => {
                            if let Err(err) = wasmsafe!(instance.drop_subscription(context.clone()).await) {
                                tracing::error!("Error dropping subscription: {err}");
                            }
                            instance.recyclable = true;
                            return Some((Response::error(err), (None, queue, context)));
                        }
                    }
                }
            },
        )
        .boxed()
    }
}
