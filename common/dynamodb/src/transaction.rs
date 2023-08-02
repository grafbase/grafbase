use std::{collections::HashMap, hash::Hash, sync::Arc, time::Duration};

use dataloader::{DataLoader, Loader, LruCache};
use dynomite::AttributeValue;
use futures_util::TryFutureExt;
use log::debug;
use rusoto_dynamodb::{DynamoDb, TransactWriteItem, TransactWriteItemsInput};
#[cfg(feature = "tracing")]
use tracing::{info_span, Instrument};

use crate::DynamoDBContext;

#[derive(Clone, Debug)]
pub enum TxItemMetadata {
    Unique { values: Vec<String>, fields: Vec<String> },
    None,
}

#[derive(Clone, Debug)]
pub struct TxItem {
    pub pk: String,
    pub sk: String,
    pub relation_name: Option<String>,
    pub metadata: TxItemMetadata,
    pub transaction: TransactWriteItem,
}

impl Hash for TxItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pk.hash(state);
        self.sk.hash(state);
        self.relation_name.hash(state);
        self.transaction
            .update
            .as_ref()
            .map(|x| &x.update_expression)
            .hash(state);
    }
}

impl PartialEq for TxItem {
    fn eq(&self, other: &Self) -> bool {
        self.pk.eq(&other.pk) && self.sk.eq(&other.sk) && self.relation_name.eq(&other.relation_name)
    }
}

impl Eq for TxItem {}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TransactionError {
    #[error("An issue happened while applying the transaction.")]
    UnknownError,
    #[error("Numeric values in a unique constraint cannot be incremented or decremented")]
    UniqueNumericAtomic,

    #[error(r#"The {} {} {} already taken on {} "{}""#,
        if .values.len() == 1 { "value" } else { "values"},
        .values.join(", "),
        if .values.len() == 1 { "is" } else { "are"},
        if .fields.len() == 1 { "field" } else { "fields" },
        .fields.join(", ")
    )]
    UniqueCondition { values: Vec<String>, fields: Vec<String> },
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
            .clone()
            .into_iter()
            .map(|x| {
                result_hashmap.insert(x.clone(), AttributeValue::default());
                x.transaction
            })
            .collect(),
    };
    debug!(ctx.trace_id, "TransactionWrite {:?}", input);

    let again = again::RetryPolicy::fixed(Duration::from_millis(10))
        .with_max_delay(Duration::from_millis(100))
        .with_max_retries(5);

    let item_collections = again.retry(|| async { ctx.dynamodb_client.transact_write_items(input.clone()).await });
    #[cfg(feature = "tracing")]
    let item_collections = item_collections.instrument(info_span!("fetch transaction"));
    let item_collections = item_collections
        .inspect_err(|err| match err {
            rusoto_core::RusotoError::Service(rusoto_dynamodb::TransactWriteItemsError::TransactionCanceled(msg))
                if msg.contains("ConditionalCheckFailed") =>
            {
                log::warn!(
                    ctx.trace_id,
                    "Error writing items in transaction due to ConditionalCheckFailed: {err:?}"
                );
            }
            _ => {
                log::error!(ctx.trace_id, "Error writing items in transaction: {err:?}");
            }
        })
        .map_err(|err| {
            if let rusoto_core::RusotoError::Service(rusoto_dynamodb::TransactWriteItemsError::TransactionCanceled(
                msg,
            )) = err
            {
                if let Some(reasons) = dynamodb_utils::transaction_cancelled_reasons(&msg) {
                    for (index, reason) in reasons.iter().enumerate() {
                        if let dynamodb_utils::TransactionCanceledReason::ConditionalCheckFailed = reason {
                            if let TxItemMetadata::Unique { ref values, ref fields } = tx[index].metadata {
                                return TransactionError::UniqueCondition {
                                    values: values.clone(),
                                    fields: fields.clone(),
                                };
                            }
                        }
                    }
                }
            }
            TransactionError::UnknownError
        })
        .await?;

    debug!(ctx.trace_id, "TransactionWriteOuput {:?}", item_collections);
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
    DataLoader::with_cache(TransactionLoader { ctx }, async_runtime::spawn, LruCache::new(256))
        .max_batch_size(25)
        .delay(Duration::from_millis(1))
}
