use dynomite::AttributeValue;
use quick_error::quick_error;
use rusoto_dynamodb::{BatchGetItemInput, DynamoDb, KeysAndAttributes};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::DynamoDBContext;

// TODO: Should ensure Rosoto Errors impl clone
quick_error! {
    #[derive(Debug, Clone)]
    pub enum BatchGetItemLoaderError {
        UnknowError {
            display("An internal error happened")
        }
        DynamoError {
            display("An internal error happened while fetching entities")
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

        let mut request_items = HashMap::new();
        let mut keys_to_send = vec![];
        for (pk, sk) in keys {
            let mut h = HashMap::new();
            h.insert(
                "__pk".to_string(),
                AttributeValue {
                    s: Some(pk.to_string()),
                    ..Default::default()
                },
            );
            h.insert(
                "__sk".to_string(),
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

        request_items.insert((&self.ctx.dynamodb_table_name).clone(), keys_and_attributes);

        let get_items = self
            .ctx
            .dynamodb_client
            .batch_get_item(BatchGetItemInput {
                request_items,
                return_consumed_capacity: None,
            })
            .inspect_err(|err| log::error!(self.ctx.trace_id, "Error while getting items: {:?}", err))
            .await
            .map_err(|_| BatchGetItemLoaderError::DynamoError)?
            .responses
            .ok_or(BatchGetItemLoaderError::UnknowError)?
            .remove(&self.ctx.dynamodb_table_name)
            .ok_or(BatchGetItemLoaderError::UnknowError)?
            .into_iter()
            .fold(HashMap::new(), |mut acc, cur| {
                let pk = cur.get("__pk").and_then(|x| x.s.clone()).unwrap();
                let sk = cur.get("__sk").and_then(|x| x.s.clone()).unwrap();
                acc.insert((pk, sk), cur);
                acc
            });

        Ok(get_items)
    }
}

pub fn get_loader_batch_transaction(ctx: Arc<DynamoDBContext>) -> DataLoader<BatchGetItemLoader, LruCache> {
    DataLoader::with_cache(
        BatchGetItemLoader { ctx },
        wasm_bindgen_futures::spawn_local,
        LruCache::new(256),
    )
    .max_batch_size(100)
    .delay(Duration::from_millis(2))
}
