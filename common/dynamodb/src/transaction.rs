use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::runtime::Runtime;
use crate::DynamoDBContext;
use dynomite::AttributeValue;
use futures_util::TryFutureExt;
use itertools::Itertools;
use log::debug;
use rusoto_dynamodb::{DynamoDb, TransactWriteItem, TransactWriteItemsInput};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info_span, Instrument};

#[derive(Clone, Debug)]
pub enum TxItemMetadata {
    Unique { value: String, field: String },
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
    #[error(r#"The value {value} is already taken on field "{field}""#)]
    UniqueCondition { value: String, field: String },
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

    let item_collections = again
        .retry(|| async { ctx.dynamodb_client.transact_write_items(input.clone()).await })
        .instrument(info_span!("fetch transaction"))
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
                if msg.contains("ConditionalCheckFailed") {
                    if let Some(reasons) = transaction_cancelled_reasons(msg) {
                        for (index, reason) in reasons.iter().enumerate() {
                            if let TransactionCanceledReason::ConditionalCheckFailed = reason {
                                if let TxItemMetadata::Unique { ref value, ref field } = tx[index].metadata {
                                    return TransactionError::UniqueCondition {
                                        value: value.clone(),
                                        field: field.clone(),
                                    };
                                }
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

enum TransactionCanceledReason {
    ConditionalCheckFailed,
    Unknown,
    None,
}

fn transaction_cancelled_reasons(message: String) -> Option<Vec<TransactionCanceledReason>> {
    message
        .strip_prefix("Transaction cancelled, please refer cancellation reasons for specific reasons ")
        // Left with e.g. `[ConditionalCheckFailed, ConditionalCheckFailed]`.
        .and_then(|reasons_string| reasons_string.strip_prefix('['))
        .and_then(|reasons_string| reasons_string.strip_suffix(']'))
        .map(|reasons_string| {
            reasons_string
                .split(", ")
                .map(|reason| match reason {
                    "ConditionalCheckFailed" => TransactionCanceledReason::ConditionalCheckFailed,
                    "None" => TransactionCanceledReason::None,
                    _ => TransactionCanceledReason::Unknown,
                })
                .collect_vec()
        })
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
        |f| Runtime::locate().spawn(f),
        LruCache::new(256),
    )
    .max_batch_size(25)
    .delay(Duration::from_millis(1))
}
