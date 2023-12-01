use std::{collections::HashMap, sync::Arc};

use dynamodb::{DynamoDBBatchersData, DynamoDBContext};
use engine::SchemaBuilder;
use rusoto_core::{credential::StaticProvider, HttpClient, RusotoError};
use rusoto_dynamodb::{
    AttributeDefinition, CreateTableError, CreateTableInput, DynamoDb, DynamoDbClient, GlobalSecondaryIndex,
    KeySchemaElement, Projection, ProvisionedThroughput,
};

pub async fn enable_local_dynamo(schema_builder: SchemaBuilder) -> SchemaBuilder {
    let dynamo_context = dynamo_context();

    create_table_if_not_exists(&dynamo_context).await;

    schema_builder.data(batcher_data(&dynamo_context)).data(dynamo_context)
}

async fn create_table_if_not_exists(dynamo_context: &Arc<DynamoDBContext>) {
    let client = DynamoDbClient::new_with(
        HttpClient::new().unwrap(),
        StaticProvider::new_minimal("anaccesskeyid".into(), "asecretaccesskey".into()),
        dynamo_context.closest_region.clone(),
    );

    let result = client
        .create_table(CreateTableInput {
            // billing_mode: Some("OnDemand".into()),
            table_name: dynamo_context.dynamodb_table_name.clone(),
            attribute_definitions: vec![
                string_attr("__pk"),
                string_attr("__sk"),
                string_attr("__gsi1pk"),
                string_attr("__gsi1sk"),
                string_attr("__gsi2pk"),
                string_attr("__gsi2sk"),
                string_attr("__gsi3pk"),
                string_attr("__gsi3sk"),
            ],
            key_schema: vec![pk("__pk"), sk("__sk")],
            global_secondary_indexes: Some(vec![gsi("gsi1"), gsi("gsi2"), gsi("gsi3")]),
            provisioned_throughput: Some(throughput()),
            ..CreateTableInput::default()
        })
        .await;

    match result {
        Ok(_) | Err(RusotoError::Service(CreateTableError::ResourceInUse(_))) => {}
        Err(e) => panic!("Couldn't create table in local dynamo table: {e}"),
    }
}

fn dynamo_context() -> Arc<DynamoDBContext> {
    Arc::new(DynamoDBContext::new(
        String::new(), // irrelevant for this
        "anaccesskeyid".to_string(),
        "asecretaccesskey".to_string(),
        rusoto_core::Region::Custom {
            name: "local".to_string(),
            endpoint: "http://localhost:8000".to_string(),
        },
        "database".to_string(),
        HashMap::new(),
        common_types::auth::ExecutionAuth::ApiKey,
    ))
}

fn batcher_data(dynamo_context: &Arc<DynamoDBContext>) -> Arc<DynamoDBBatchersData> {
    DynamoDBBatchersData::new(dynamo_context, None)
}

fn string_attr(name: impl Into<String>) -> AttributeDefinition {
    AttributeDefinition {
        attribute_name: name.into(),
        attribute_type: "S".into(),
    }
}

fn gsi(name: impl Into<String>) -> GlobalSecondaryIndex {
    let name = name.into();
    GlobalSecondaryIndex {
        index_name: name.clone(),
        key_schema: vec![pk(format!("__{name}pk")), sk(format!("__{name}sk"))],
        projection: Projection {
            projection_type: Some("ALL".into()),
            ..Default::default()
        },
        provisioned_throughput: Some(throughput()),
    }
}

fn pk(name: impl Into<String>) -> KeySchemaElement {
    KeySchemaElement {
        attribute_name: name.into(),
        key_type: "HASH".into(),
    }
}

fn sk(name: impl Into<String>) -> KeySchemaElement {
    KeySchemaElement {
        attribute_name: name.into(),
        key_type: "RANGE".into(),
    }
}

fn throughput() -> ProvisionedThroughput {
    ProvisionedThroughput {
        // Just some nonsense to keep the simulator happy?]
        read_capacity_units: 100,
        write_capacity_units: 100,
    }
}
