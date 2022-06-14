use super::bridge_api;
use super::types::{Record, Sql};
use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::LocalContext;
use dynomite::AttributeValue;
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
        NetworkError {
            display("An internal error happened while sending a request to the bridge server")
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
        let serial_keys: Vec<String> = keys.iter().flat_map(|(pk, sk)| [pk.clone(), sk.clone()]).collect();

        let query = Sql::SelectIdPairs(serial_keys.len() / 2);

        let results = bridge_api::query(&query.to_string(), &serial_keys, &self.local_ctx.bridge_port)
            .await
            .map_err(|_| Self::Error::NetworkError)?;

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
        wasm_bindgen_futures::spawn_local,
        LruCache::new(256),
    )
    .max_batch_size(100)
    .delay(Duration::from_millis(2))
}
