use async_stream as _;
use composite_id as _;
use dynomite::{AttributeError, AttributeValue};
use futures_util as _;
use log::{error, info};
use quick_error::quick_error;
use rusoto_core::credential::StaticProvider;
use rusoto_core::{HttpClient, RusotoError};
use rusoto_dynamodb::{
    DynamoDb, DynamoDbClient, GetItemError, GetItemInput, GetItemOutput, PutItemError, QueryError,
    TransactWriteItemsError,
};
use serde as _;
use std::collections::HashMap;
use strum as _;
use surf as _;

/// The DynamoDBContext that is needed to query the Database
pub struct DynamoDBContext {
    // TODO: When going with tracing, remove this trace_id, useless.
    trace_id: String,
    dynamodb_client: rusoto_dynamodb::DynamoDbClient,
    dynamodb_table_name: String,
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

    /// We Get an item with PK and SK
    pub async fn get_item_pk_sk<PK: AsRef<str> + Send, SK: AsRef<str> + Send>(
        &self,
        pk: PK,
        sk: SK,
    ) -> Result<GetItemOutput, DynamoDBError> {
        info!(&self.trace_id, "Querying DynamoDB");
        info!(&self.trace_id, "pk: {} | sk : {}", &pk.as_ref(), &sk.as_ref());

        let mut key = HashMap::with_capacity(2);
        key.insert(
            "pk".to_string(),
            AttributeValue {
                s: Some(pk.as_ref().to_string()),
                ..Default::default()
            },
        );
        key.insert(
            "sk".to_string(),
            AttributeValue {
                s: Some(sk.as_ref().to_string()),
                ..Default::default()
            },
        );

        let input = GetItemInput {
            table_name: self.dynamodb_table_name.to_string(),
            consistent_read: Some(false),
            key,
            ..Default::default()
        };

        info!(&self.trace_id, "{:?}", &input);
        let result = self.dynamodb_client.get_item(input).await.map_err(|err| {
            error!(&self.trace_id, "{:?}", err);
            err
        });
        info!(&self.trace_id, "{:?}", &result);

        Ok(result?)
    }
}
