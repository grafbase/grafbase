use super::bridge_api;
use super::types::{Operation, Record, Sql, SqlValue};
use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::runtime::Runtime;
use crate::LocalContext;
use dynomite::AttributeValue;
use maplit::hashmap;
use quick_error::quick_error;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

quick_error! {
    #[derive(Debug, Clone)]
    pub enum BatchGetItemLoaderError {
        UnknownError {
            display("An internal error happened")
        }
    }
}

pub struct BatchGetItemLoader {
    local_ctx: Arc<LocalContext>,
}

#[async_trait::async_trait]
impl Loader<(String, String)> for BatchGetItemLoader {
    type Value = HashMap<String, AttributeValue>;
    type Error = BatchGetItemLoaderError;

    async fn load(&self, keys: &[(String, String)]) -> Result<HashMap<(String, String), Self::Value>, Self::Error> {
        let (partition_keys, sorting_keys): (Vec<String>, Vec<String>) = keys.iter().cloned().unzip();

        let key_count = partition_keys.len();

        let value_map = hashmap! {
            "partition_keys" => SqlValue::VecDeque(partition_keys.into()),
            "sorting_keys"=> SqlValue::VecDeque(sorting_keys.into()),
        };

        let (query, values) = Sql::SelectIdPairs(key_count).compile(value_map);

        let results = bridge_api::query(
            Operation {
                sql: query,
                values,
                kind: None,
            },
            &self.local_ctx.bridge_port,
        )
        .await
        .map_err(|_| Self::Error::UnknownError)?;

        let response = results
            .iter()
            .map(
                |&Record {
                     ref pk,
                     ref sk,
                     ref document,
                     ..
                 }| ((pk.clone(), sk.clone()), document.clone()),
            )
            .collect();

        Ok(response)
    }
}

pub fn get_loader_batch_transaction(local_ctx: Arc<LocalContext>) -> DataLoader<BatchGetItemLoader, LruCache> {
    DataLoader::with_cache(
        BatchGetItemLoader { local_ctx },
        |f| Runtime::locate().spawn(f),
        LruCache::new(128),
    )
    .max_batch_size(100)
    .delay(Duration::from_millis(2))
}
