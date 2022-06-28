use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::DynamoDBContext;
use dynomite::AttributeValue;
use futures_util::TryFutureExt;
use log::info;
use quick_error::quick_error;
use rusoto_dynamodb::{DynamoDb, TransactWriteItem, TransactWriteItemsInput};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct TxItem {
    pub pk: String,
    pub sk: String,
    pub relation_name: Option<String>,
    pub transaction: TransactWriteItem,
}

impl Hash for TxItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pk.hash(state);
        self.sk.hash(state);
        self.relation_name.hash(state);
    }
}

impl PartialEq for TxItem {
    fn eq(&self, other: &Self) -> bool {
        self.pk.eq(&other.pk) && self.sk.eq(&other.sk) && self.relation_name.eq(&other.relation_name)
    }
}

impl Eq for TxItem {}

quick_error! {
    #[derive(Debug, Clone)]
    pub enum TransactionError {
        UnknownError {
            display("An issue happened while applying the transaction.")
        }
    }

}

/// The result is not accessible, the Hashmap will be empty
async fn transaction_by_pk(
    ctx: &DynamoDBContext,
    tx: Vec<TxItem>,
) -> Result<HashMap<TxItem, AttributeValue>, TransactionError> {
    let mut result_hashmap = HashMap::with_capacity(tx.len());
    let input = TransactWriteItemsInput {
        client_request_token: None, // TODO: Should add one
        return_consumed_capacity: None,
        return_item_collection_metrics: None,
        transact_items: tx
            .into_iter()
            .map(|x| {
                result_hashmap.insert(x.clone(), AttributeValue::default());
                x.transaction
            })
            .collect(),
    };
    info!(ctx.trace_id, "TransactionWrite {:?}", input);

    let again = again::RetryPolicy::default()
        .with_max_delay(Duration::from_millis(50))
        .with_max_retries(3)
        .with_jitter(true);

    let item_collections = again
        .retry(|| async {
            ctx.dynamodb_client
                .transact_write_items(input.clone())
                .inspect_err(|err| log::error!(ctx.trace_id, "Error while writing the transaction: {:?}", err))
                .await
                .map_err(|_| TransactionError::UnknownError)
        })
        .await?;

    info!(ctx.trace_id, "TransactionWriteOuput {:?}", item_collections);
    Ok(result_hashmap)
}

pub struct TransactionLoader {
    ctx: Arc<DynamoDBContext>,
}

#[async_trait::async_trait]
impl Loader<TxItem> for TransactionLoader {
    type Value = AttributeValue;
    type Error = TransactionError;

    async fn load(&self, keys: &[TxItem]) -> Result<HashMap<TxItem, Self::Value>, Self::Error> {
        transaction_by_pk(&self.ctx, keys.to_vec()).await
    }
}

pub fn get_loader_transaction(ctx: Arc<DynamoDBContext>) -> DataLoader<TransactionLoader, LruCache> {
    DataLoader::with_cache(
        TransactionLoader { ctx },
        wasm_bindgen_futures::spawn_local,
        LruCache::new(256),
    )
    .max_batch_size(25)
    .delay(Duration::from_millis(2))
}
