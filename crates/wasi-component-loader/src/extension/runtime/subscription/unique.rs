use std::{collections::VecDeque, sync::Arc};

use super::SubscriptionStream;
use crate::extension::{pool::ExtensionGuard, wit};
use engine_schema::Subgraph;
use futures::stream;
use runtime::{
    error::{PartialErrorCode, PartialGraphqlError},
    extension::Data,
};

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
    pub async fn resolve<'f>(self) -> Result<SubscriptionStream<'f>, PartialGraphqlError>
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
            .map_err(|err| err.into_graphql_error(PartialErrorCode::ExtensionError))?;

        let stream = stream::unfold((instance, VecDeque::new()), async move |(mut instance, mut tail)| {
            if let Some(data) = tail.pop_front() {
                return Some((data, (instance, tail)));
            }

            let item = loop {
                match instance.resolve_next_subscription_item().await {
                    Ok(Some(item)) if item.outputs.is_empty() => {
                        continue;
                    }
                    Ok(Some(item)) => {
                        tracing::debug!("subscription item resolved");
                        break item;
                    }
                    Ok(None) => {
                        tracing::debug!("subscription completed");
                        return None;
                    }
                    Err(e) => {
                        tracing::error!("Error resolving subscription item: {e}");
                        return Some((Err(PartialGraphqlError::internal_extension_error()), (instance, tail)));
                    }
                }
            };

            for item in item.outputs {
                match item {
                    Ok(item) => {
                        tail.push_back(Ok(Arc::new(Data::CborBytes(item))));
                    }
                    Err(error) => {
                        let error = error.into_graphql_error(PartialErrorCode::InternalServerError);
                        tail.push_back(Err(error));
                    }
                }
            }

            tail.pop_front().map(|item| (item, (instance, tail)))
        });

        Ok(Box::pin(stream))
    }
}
