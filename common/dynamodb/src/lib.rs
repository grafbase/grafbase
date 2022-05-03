use async_stream as _;
use batch_getitem::{get_loader_batch_transaction, BatchGetItemLoader};
use composite_id as _;
use dataloader::{DataLoader, LruCache};
use dynomite::AttributeError;
use futures_util as _;
use query::get_loader_query;
use query_by_type::get_loader_query_type;
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
mod query;
mod query_by_type;
mod transaction;
pub use batch_getitem::BatchGetItemLoaderError;
pub use query::{QueryKey, QueryLoader, QueryLoaderError};
pub use query_by_type::{QueryTypeKey, QueryTypeLoader, QueryTypeLoaderError};
pub use transaction::TxItem;

/// The DynamoDBContext that is needed to query the Database
#[derive(Clone)]
pub struct DynamoDBContext {
    // TODO: When going with tracing, remove this trace_id, useless.
    trace_id: String,
    dynamodb_client: rusoto_dynamodb::DynamoDbClient,
    pub dynamodb_table_name: String,
}

/// Describe DynamoDBTables available in a GlobalDB Project.
pub enum DynamoDBRequestedIndex {
    None,
    /// The reverse Index where the PK and SK are reversed.
    ReverseIndex,
    /// The fat partition Index where the PK is stripped of his ULID and is
    /// corresponding of the type.
    FatPartitionIndex,
}

impl DynamoDBRequestedIndex {
    fn to_index_name(&self) -> Option<String> {
        match self {
            DynamoDBRequestedIndex::None => None,
            DynamoDBRequestedIndex::ReverseIndex => Some("gsi2".to_string()),
            DynamoDBRequestedIndex::FatPartitionIndex => Some("gsi1".to_string()),
        }
    }

    fn pk(&self) -> String {
        match self {
            DynamoDBRequestedIndex::None => "__pk".to_string(),
            DynamoDBRequestedIndex::ReverseIndex => "__gsi2pk".to_string(),
            DynamoDBRequestedIndex::FatPartitionIndex => "__gsi1pk".to_string(),
        }
    }

    #[allow(dead_code)]
    fn sk(&self) -> String {
        match self {
            DynamoDBRequestedIndex::None => "__sk".to_string(),
            DynamoDBRequestedIndex::ReverseIndex => "__gsi2sk".to_string(),
            DynamoDBRequestedIndex::FatPartitionIndex => "__gsi1sk".to_string(),
        }
    }
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
    pub ctx: Arc<DynamoDBContext>,
    /// Used to batch transactions.
    pub transaction: DataLoader<TransactionLoader, LruCache>,
    /// Used to load items knowing the `PK` and `SK` from table
    pub loader: DataLoader<BatchGetItemLoader, LruCache>,
    /// Used to load items with only PK from table
    pub query: DataLoader<QueryLoader, LruCache>,
    /// Used to load items with only PK from FatPartition
    pub query_fat: DataLoader<QueryTypeLoader, LruCache>,
}

impl DynamoDBBatchersData {
    pub fn new(ctx: &Arc<DynamoDBContext>) -> Self {
        Self {
            ctx: Arc::clone(ctx),
            transaction: get_loader_transaction(Arc::clone(ctx)),
            loader: get_loader_batch_transaction(Arc::clone(ctx)),
            query: get_loader_query(Arc::clone(ctx), DynamoDBRequestedIndex::None),
            query_fat: get_loader_query_type(Arc::clone(ctx), DynamoDBRequestedIndex::FatPartitionIndex),
        }
    }
}
