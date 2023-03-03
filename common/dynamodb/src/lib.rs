cfg_if::cfg_if! {
    if #[cfg(not(feature = "local"))] {
        mod batch_getitem;
        mod query;
        mod query_by_type;
        mod query_by_type_paginated;
        mod query_single_by_relation;

        use batch_getitem::{get_loader_batch_transaction, BatchGetItemLoader};
        use query::get_loader_query;
        use query_by_type::get_loader_query_type;
        use query_by_type_paginated::{get_loader_paginated_query_type, QueryTypePaginatedLoader};
        use query_single_by_relation::get_loader_single_relation_query;

        pub use query_by_type::{QueryTypeKey, QueryTypeLoader, QueryTypeLoaderError};
        pub use query::{QueryKey, QueryLoader, QueryLoaderError};
        pub use batch_getitem::BatchGetItemLoaderError;
        pub use query_by_type_paginated::{QueryTypePaginatedKey, QueryTypePaginatedValue};
        pub use query_single_by_relation::{QuerySingleRelationKey, QuerySingleRelationLoader, QuerySingleRelationLoaderError};
    } else {
        mod local;

        use local::batch_getitem::{get_loader_batch_transaction, BatchGetItemLoader};
        use local::query::get_loader_query;
        use local::query_single_by_relation::get_loader_single_relation_query;
        use local::query_by_type::get_loader_query_type;
        use local::query_by_type_paginated::{get_loader_paginated_query_type, QueryTypePaginatedLoader};

        pub use local::query_by_type_paginated::{QueryTypePaginatedKey, QueryTypePaginatedValue};
        pub use local::query_by_type::{QueryTypeKey, QueryTypeLoader, QueryTypeLoaderError};
        pub use local::query_single_by_relation::{QuerySingleRelationKey, QuerySingleRelationLoader, QuerySingleRelationLoaderError};
        pub use local::query::{QueryKey, QueryLoader, QueryLoaderError};
        pub use local::local_context::LocalContext;
        pub use local::batch_getitem::BatchGetItemLoaderError;
    }
}

use dataloader::{DataLoader, LruCache};
use dynomite::AttributeError;

use quick_error::quick_error;
use rusoto_core::credential::StaticProvider;
use rusoto_core::{HttpClient, RusotoError};
use rusoto_dynamodb::{DynamoDbClient, GetItemError, PutItemError, QueryError, TransactWriteItemsError};
use std::sync::Arc;
use transaction::{get_loader_transaction, TransactionLoader};

pub mod constant;
pub mod dataloader;

pub mod export {
    pub use graph_entities;
}

mod utils;
pub use utils::current_datetime::CurrentDateTime;
pub use utils::{attribute_to_value, value_to_attribute};

pub mod graph_transaction;
mod paginated;

mod runtime;
mod transaction;

pub use graph_transaction::{get_loader_transaction_new, NewTransactionLoader, PossibleChanges};
pub use paginated::{PaginatedCursor, PaginationOrdering, ParentEdge, QueryValue};

pub use transaction::{TransactionError, TxItem};

/// The DynamoDBContext that is needed to query the Database
#[derive(Clone)]
pub struct DynamoDBContext {
    // TODO: When going with tracing, remove this trace_id, useless.
    pub trace_id: String,
    pub dynamodb_client: rusoto_dynamodb::DynamoDbClient,
    pub dynamodb_table_name: String,
    pub closest_region: rusoto_core::Region,
    // FIXME: Move this to `grafbase-runtime`!
    pub resolver_binding_map: std::collections::HashMap<String, String>,
    pub user_id: Option<String>,
}

/// Describe DynamoDBTables available in a GlobalDB Project.
#[derive(Clone)]
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
            Self::None => None,
            Self::ReverseIndex => Some("gsi2".to_string()),
            Self::FatPartitionIndex => Some("gsi1".to_string()),
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(not(feature = "local"))] {
            fn pk(&self) -> String {
                match self {
                    Self::None => "__pk".to_string(),
                    Self::ReverseIndex => "__gsi2pk".to_string(),
                    Self::FatPartitionIndex => "__gsi1pk".to_string(),
                }
            }

            #[allow(dead_code)]
            fn sk(&self) -> String {
                match self {
                    Self::None => "__sk".to_string(),
                    Self::ReverseIndex => "__gsi2sk".to_string(),
                    Self::FatPartitionIndex => "__gsi1sk".to_string(),
                }
            }
        } else {
            fn pk(&self) -> String {
                match self {
                    Self::None => "pk".to_string(),
                    Self::ReverseIndex => "gsi2pk".to_string(),
                    Self::FatPartitionIndex => "gsi1pk".to_string(),
                }
            }

            #[allow(dead_code)]
            fn sk(&self) -> String {
                match self {
                    Self::None => "sk".to_string(),
                    Self::ReverseIndex => "gsi2sk".to_string(),
                    Self::FatPartitionIndex => "gsi1sk".to_string(),
                }
            }
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
    /// * `user_id` - Optional ID identifying the current user
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        // TODO: This should go away with tracing.
        trace_id: String,
        access_key_id: String,
        secret_access_key: String,
        dynamodb_replication_regions: Vec<aws_region_nearby::AwsRegion>,
        dynamodb_table_name: String,
        latitude: f32,
        longitude: f32,
        // FIXME: Move this to `grafbase-runtime`!
        resolver_binding_map: std::collections::HashMap<String, String>,
        user_id: Option<String>,
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
        let client = DynamoDbClient::new_with(http_client, provider, closest_region.clone());

        Self {
            trace_id,
            dynamodb_client: client,
            dynamodb_table_name,
            closest_region,
            resolver_binding_map,
            user_id,
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
    /// Used to load items for the ReversedIndex with only PK from table
    pub query_reversed: DataLoader<QueryLoader, LruCache>,
    /// Used to load items with only PK from FatPartition
    pub query_fat: DataLoader<QueryTypeLoader, LruCache>,
    /// Used to laod items with only PK from FatPartition with Pagination
    pub paginated_query_fat: DataLoader<QueryTypePaginatedLoader, LruCache>,
    /// Bl
    pub transaction_new: DataLoader<NewTransactionLoader, LruCache>,
    /// Used to load single relations
    pub query_single_relation: DataLoader<QuerySingleRelationLoader, LruCache>,
}

impl DynamoDBBatchersData {
    pub fn clear(&self) {
        self.transaction.clear();
        self.loader.clear();
        self.query.clear();
        self.query_reversed.clear();
        self.query_fat.clear();
        self.paginated_query_fat.clear();
        self.transaction_new.clear();
        self.query_single_relation.clear();
    }
}

#[cfg(not(feature = "local"))]
impl DynamoDBBatchersData {
    pub fn new(ctx: &Arc<DynamoDBContext>) -> Arc<Self> {
        Arc::new_cyclic(|b| Self {
            ctx: Arc::clone(ctx),
            transaction: get_loader_transaction(Arc::clone(ctx)),
            loader: get_loader_batch_transaction(Arc::clone(ctx)),
            query: get_loader_query(Arc::clone(ctx), DynamoDBRequestedIndex::None),
            query_reversed: get_loader_query(Arc::clone(ctx), DynamoDBRequestedIndex::ReverseIndex),
            query_fat: get_loader_query_type(Arc::clone(ctx), DynamoDBRequestedIndex::FatPartitionIndex),
            paginated_query_fat: get_loader_paginated_query_type(
                Arc::clone(ctx),
                DynamoDBRequestedIndex::FatPartitionIndex,
            ),
            transaction_new: get_loader_transaction_new(Arc::clone(ctx), b.clone()),
            query_single_relation: get_loader_single_relation_query(Arc::clone(ctx), DynamoDBRequestedIndex::None),
        })
    }
}

#[cfg(feature = "local")]
impl DynamoDBBatchersData {
    pub fn new(ctx: &Arc<DynamoDBContext>, local_ctx: &Arc<LocalContext>) -> Arc<Self> {
        Arc::new_cyclic(|b| Self {
            ctx: Arc::clone(ctx),
            transaction: get_loader_transaction(Arc::clone(ctx)),
            loader: get_loader_batch_transaction(Arc::clone(local_ctx), Arc::clone(ctx)),
            query: get_loader_query(Arc::clone(local_ctx), DynamoDBRequestedIndex::None),
            query_reversed: get_loader_query(Arc::clone(local_ctx), DynamoDBRequestedIndex::ReverseIndex),
            query_fat: get_loader_query_type(Arc::clone(local_ctx), DynamoDBRequestedIndex::FatPartitionIndex),
            paginated_query_fat: get_loader_paginated_query_type(
                Arc::clone(local_ctx),
                Arc::clone(ctx),
                DynamoDBRequestedIndex::FatPartitionIndex,
            ),
            transaction_new: get_loader_transaction_new(Arc::clone(ctx), b.clone(), Arc::clone(local_ctx)),
            query_single_relation: get_loader_single_relation_query(
                Arc::clone(local_ctx),
                DynamoDBRequestedIndex::None,
            ),
        })
    }
}
