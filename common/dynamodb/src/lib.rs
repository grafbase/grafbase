use async_stream as _;
use batch_getitem::{get_loader_batch_transaction, BatchGetItemLoader};
use composite_id as _;
use dataloader::{DataLoader, LruCache};
use dynomite::AttributeError;
use futures_util as _;
use quick_error::quick_error;
use rusoto_core::credential::StaticProvider;
use rusoto_core::{HttpClient, RusotoError};
use rusoto_dynamodb::{DynamoDbClient, GetItemError, PutItemError, QueryError, TransactWriteItemsError};
use serde as _;
use std::sync::Arc;
use strum as _;
use surf as _;
use transaction::{get_loader_transaction, TransactionLoader};

mod batch_getitem;
pub mod dataloader;
mod transaction;
pub use transaction::TxItem;

/// The DynamoDBContext that is needed to query the Database
#[derive(Clone)]
pub struct DynamoDBContext {
    // TODO: When going with tracing, remove this trace_id, useless.
    trace_id: String,
    dynamodb_client: rusoto_dynamodb::DynamoDbClient,
    pub dynamodb_table_name: String,
}

quick_error! {
    #[derive(Debug)]
    pub enum DynamoDBError {
        AttributeConversion(err: AttributeError) {
            source(err)
            display("An internal error happened - EI1")
            from()
        }
        Query(err: RusotoError<QueryError>) {
            source(err)
            display("An internal error happened - EI2")
            from()
        }
        Transaction(err: RusotoError<TransactWriteItemsError>) {
            source(err)
            display("An internal error happened - EI3")
            from()
        }
        ItemNotFound {
            display("An internal error happened - EI4")
        }
        UnexpectedItemCount {
            display("An internal error happened - EI5")
        }
        Write(err: RusotoError<PutItemError>) {
            source(err)
            display("An internal error happened - EI6")
            from()
        }
        ReadItem(err: RusotoError<GetItemError>) {
            source(err)
            display("An internal error happened - EI7")
            from()
        }
        TransactionNoChanges {
            display("An internal error happened - EI8")
        }
    }
}

impl DynamoDBContext {
    /// Create a new context
    ///
    /// # Arguments
    ///
    /// * `trace_id` - Trace id, should be removed as soon as we have tracing.
    /// * `access_key_id` - AWS Access Key.
    /// * `secret_access_key` - AWS Secret Access Key.
    /// * `dynamodb_replication_regions` - The Regions in which the dynamodb table is replicated.
    /// * `dynamodb_table_name` - The DynamoDB TableName.
    /// * `latitude` - Request latitude, to locate the closest region
    /// * `longitude` - Request longitude, to locate the closest region
    ///
    pub fn new(
        // TODO: This should go away with tracing.
        trace_id: String,
        access_key_id: String,
        secret_access_key: String,
        dynamodb_replication_regions: Vec<aws_region_nearby::AwsRegion>,
        dynamodb_table_name: String,
        latitude: f32,
        longitude: f32,
    ) -> Self {
        let provider = StaticProvider::new_minimal(access_key_id, secret_access_key);
        let closest_region: rusoto_core::Region =
            aws_region_nearby::find_region_from_list(latitude, longitude, &dynamodb_replication_regions)
                .name()
                .parse()
                .expect("the name of the region is certainly valid");

        log::debug!(
            &trace_id,
            "Picked the closest region {} for coordinates (lat {}, lon {})",
            closest_region.name(),
            latitude,
            longitude
        );

        let http_client = HttpClient::new().expect("failed to create HTTP client");
        let client = DynamoDbClient::new_with(http_client, provider, closest_region);

        Self {
            trace_id,
            dynamodb_client: client,
            dynamodb_table_name,
        }
    }

    #[allow(dead_code)]
    /// GSI name used to access to items with a specific type.
    pub(crate) const fn index_type() -> &'static str {
        "gsi1"
    }

    #[allow(dead_code)]
    /// GSI name used to reverse lockup
    pub(crate) const fn index_reverse_lockup() -> &'static str {
        "gsi2"
    }
}

pub struct DynamoDBBatchersData {
    pub transaction: DataLoader<TransactionLoader, LruCache>,
    pub loader: DataLoader<BatchGetItemLoader, LruCache>,
}

impl DynamoDBBatchersData {
    pub fn new(ctx: &Arc<DynamoDBContext>) -> Self {
        Self {
            transaction: get_loader_transaction(Arc::clone(ctx)),
            loader: get_loader_batch_transaction(Arc::clone(ctx)),
        }
    }
}
