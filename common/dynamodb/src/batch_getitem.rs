use dataloader::{DataLoader, Loader, LruCache};
use dynomite::AttributeValue;
use quick_error::quick_error;
use rusoto_dynamodb::{BatchGetItemInput, DynamoDb, KeysAndAttributes};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "tracing")]
use tracing::{info_span, Instrument};

use crate::constant::{OWNED_BY, PK, SK};
use crate::{DynamoDBContext, OperationAuthorization, OperationAuthorizationError, RequestedOperation};

// TODO: Should ensure Rosoto Errors impl clone
quick_error! {
    #[derive(Debug, Clone)]
    pub enum BatchGetItemLoaderError {
        UnknownError {
            display("An internal error happened")
        }
        DynamoError {
            display("An internal error happened while fetching entities")
        }
        MissingUniqueFields {
            display("Couldn't find values for required unique fields")
        }
        AuthorizationError(err: OperationAuthorizationError) {
            from()
            source(err)
            display("Unauthorized")
        }
    }
}

pub struct BatchGetItemLoader {
    ctx: Arc<DynamoDBContext>,
}

#[async_trait::async_trait]
impl Loader<(String, String)> for BatchGetItemLoader {
    type Value = HashMap<String, AttributeValue>;
    type Error = BatchGetItemLoaderError;

    async fn load(&self, keys: &[(String, String)]) -> Result<HashMap<(String, String), Self::Value>, Self::Error> {
        use futures_util::TryFutureExt;
        log::debug!(self.ctx.trace_id, "Loader Dataloader invoked {:?}", keys);

        let mut request_items = HashMap::new();
        let mut keys_to_send = vec![];
        for (pk, sk) in keys {
            let mut h = HashMap::new();
            h.insert(
                PK.to_string(),
                AttributeValue {
                    s: Some(pk.to_string()),
                    ..Default::default()
                },
            );
            h.insert(
                SK.to_string(),
                AttributeValue {
                    s: Some(sk.to_string()),
                    ..Default::default()
                },
            );

            keys_to_send.push(h);
        }

        let keys_and_attributes: KeysAndAttributes = KeysAndAttributes {
            attributes_to_get: None,
            keys: keys_to_send,
            consistent_read: None,
            projection_expression: None,
            expression_attribute_names: None,
        };

        request_items.insert(self.ctx.dynamodb_table_name.clone(), keys_and_attributes);
        let owned_by = match self.ctx.authorize_operation(RequestedOperation::Get)? {
            OperationAuthorization::OwnerBased(owned_by) => Some(owned_by),
            _ => None,
        };

        let request_fut = crate::retry::rusoto_retry(|| {
            self.ctx
                .dynamodb_client
                .batch_get_item(BatchGetItemInput {
                    request_items: request_items.clone(),
                    return_consumed_capacity: None,
                })
                .inspect_err(|err| log::error!(self.ctx.trace_id, "Error while getting items: {:?}", err))
        });
        #[cfg(feature = "tracing")]
        let request_fut = request_fut.instrument(info_span!("fetch batch_get_item"));
        let get_items = request_fut
            .await
            .map_err(|_| BatchGetItemLoaderError::DynamoError)?
            .responses
            .ok_or(BatchGetItemLoaderError::UnknownError)?
            .remove(&self.ctx.dynamodb_table_name)
            .ok_or(BatchGetItemLoaderError::UnknownError)?
            .into_iter()
            .filter(|item| {
                // BatchGetItem doesn't support filtering, so do it manually
                if let Some(user_id) = owned_by {
                    item.get(OWNED_BY)
                        .and_then(|av| av.ss.as_ref())
                        .map(|owners| owners.iter().any(|it| it == user_id))
                        .unwrap_or(false)
                } else {
                    true
                }
            })
            .fold(HashMap::new(), |mut acc, cur| {
                let pk = cur.get(PK).and_then(|x| x.s.clone()).unwrap();
                let sk = cur.get(SK).and_then(|x| x.s.clone()).unwrap();
                acc.insert((pk, sk), cur);
                acc
            });

        log::debug!(self.ctx.trace_id, "Loader Dataloader finished {:?}", keys);
        Ok(get_items)
    }
}

pub fn get_loader_batch_transaction(ctx: Arc<DynamoDBContext>) -> DataLoader<BatchGetItemLoader, LruCache> {
    DataLoader::with_cache(BatchGetItemLoader { ctx }, async_runtime::spawn, LruCache::new(128))
        .max_batch_size(100)
        .delay(Duration::from_millis(2))
}
