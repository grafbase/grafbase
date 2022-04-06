use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::DynamoDBContext;
use dynomite::AttributeValue;
use futures_util::TryFutureExt;
use log::debug;
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
    pub transaction: TransactWriteItem,
}

impl Hash for TxItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pk.hash(state);
        self.sk.hash(state);
    }
}

impl PartialEq for TxItem {
    fn eq(&self, other: &Self) -> bool {
        self.pk.eq(&other.pk) && self.sk.eq(&other.sk)
    }
}

impl Eq for TxItem {}

quick_error! {
    #[derive(Debug, Clone)]
    pub enum TransactionError {
        UnknowError {
            display("An issue happened while applying the transaction.")
        }
    }

}

/// The result is not accessible, the Hashmap will be empty
async fn transaction_by_pk(
    ctx: &DynamoDBContext,
    tx: Vec<TxItem>,
) -> Result<HashMap<TxItem, AttributeValue>, TransactionError> {
    let input = TransactWriteItemsInput {
        client_request_token: None, // TODO: Should add one
        return_consumed_capacity: None,
        return_item_collection_metrics: None,
        transact_items: tx.iter().map(|x| x.transaction.clone()).collect(),
    };
    debug!(ctx.trace_id, "TransactionWrite {:?}", input);

    let item_collections = ctx
        .dynamodb_client
        .transact_write_items(input)
        .inspect_err(|err| log::error!(ctx.trace_id, "Error while writing the transaction: {:?}", err))
        .await
        .map_err(|_| TransactionError::UnknowError)?;

    debug!(ctx.trace_id, "TransactionWriteOuput {:?}", item_collections);

    let result_hashmap = HashMap::new();
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

pub(crate) fn get_loader_transaction(ctx: Arc<DynamoDBContext>) -> DataLoader<TransactionLoader, LruCache> {
    let loader = DataLoader::with_cache(
        TransactionLoader { ctx },
        wasm_bindgen_futures::spawn_local,
        LruCache::new(128),
    )
    .max_batch_size(25)
    .delay(Duration::from_millis(2));
    loader
}
