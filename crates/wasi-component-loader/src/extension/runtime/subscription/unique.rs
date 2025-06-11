use std::collections::VecDeque;

use super::SubscriptionStream;
use crate::{
    Error,
    extension::{ExtensionGuard, api::wit},
};
use engine_error::{ErrorCode, GraphqlError};
use engine_schema::Subgraph;
use futures::stream;

/// A subscription without deduplication, reserving one extension instance for each subscription.
///
/// The system uses this when the extension does not define a deduplication key.
pub struct UniqueSubscription<'ctx, 'wit> {
    pub instance: ExtensionGuard,
    pub headers: http::HeaderMap,
    pub subgraph: Subgraph<'ctx>,
    pub directive: wit::FieldDefinitionDirective<'wit>,
}

impl<'ctx> UniqueSubscription<'ctx, '_> {
    pub async fn resolve<'f>(self) -> Result<SubscriptionStream<'f>, GraphqlError>
    where
        'ctx: 'f,
    {
        let UniqueSubscription {
            mut instance,
            headers,
            subgraph,
            directive,
        } = self;

        instance
            .resolve_subscription(headers, subgraph.name(), directive)
            .await
            .map_err(|err| match err {
                Error::Internal(err) => {
                    tracing::error!("Wasm error: {err}");
                    GraphqlError::new("Internal error", ErrorCode::ExtensionError)
                }
                Error::Guest(err) => err.into_graphql_error(ErrorCode::ExtensionError),
            })?;

        let stream = stream::unfold((instance, VecDeque::new()), async move |(mut instance, mut tail)| {
            if let Some(data) = tail.pop_front() {
                return Some((data, (instance, tail)));
            }

            let items = loop {
                match instance.field_resolver_resolve_next_subscription_item().await {
                    Ok(Some(items)) if items.is_empty() => {
                        continue;
                    }
                    Ok(Some(items)) => {
                        tracing::debug!("subscription item resolved");
                        break items;
                    }
                    Ok(None) => {
                        tracing::debug!("subscription completed");
                        return None;
                    }
                    Err(e) => {
                        tracing::error!("Error resolving subscription item: {e}");
                        return Some((Err(GraphqlError::internal_extension_error()), (instance, tail)));
                    }
                }
            };

            for item in items {
                match item {
                    Ok(data) => {
                        tail.push_back(Ok(data));
                    }
                    Err(error) => {
                        tail.push_back(Err(error));
                    }
                }
            }

            tail.pop_front().map(|item| (item, (instance, tail)))
        });

        Ok(Box::pin(stream))
    }
}
