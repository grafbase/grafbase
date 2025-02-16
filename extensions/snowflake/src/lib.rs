use std::sync::Arc;
use deadpool::managed::{Manager, Pool, PoolError};
use grafbase_sdk::{
    types::{Configuration, Directive, FieldDefinition, FieldInputs, FieldOutput},
    Error, Extension, Resolver, ResolverExtension, SharedContext,
};
use serde::{Deserialize, Serialize};
use snowflake_api::{Client as SnowflakeClient, ClientBuilder, QueryResult as SnowflakeQueryResult};
use thiserror::Error;
use tracing::{debug, error, info};

#[derive(Error, Debug)]
enum SnowflakeError {
    #[error("Failed to connect to Snowflake: {0}")]
    Connection(String),
    #[error("Failed to execute query: {0}")]
    Query(String),
    #[error("Pool error: {0}")]
    Pool(#[from] PoolError),
}

struct SnowflakeManager {
    config: SnowflakeConfig,
}

impl Manager for SnowflakeManager {
    type Type = SnowflakeClient;
    type Error = SnowflakeError;

    async fn create(&self) -> Result<SnowflakeClient, SnowflakeError> {
        let client = ClientBuilder::new()
            .account(&self.config.account)
            .username(&self.config.username)
            .password(&self.config.password)
            .database(&self.config.database)
            .warehouse(&self.config.warehouse)
            .role(self.config.role.as_deref().unwrap_or("PUBLIC"))
            .build()
            .map_err(|e| SnowflakeError::Connection(e.to_string()))?;

        Ok(client)
    }

    async fn recycle(&self, _client: &mut SnowflakeClient) -> Result<(), SnowflakeError> {
        // Could implement connection testing here if needed
        Ok(())
    }
}

#[derive(ResolverExtension)]
struct SnowflakeExtension {
    pool: Arc<Pool<SnowflakeManager>>,
}

#[derive(Deserialize)]
struct SnowflakeConfig {
    account: String,
    username: String,
    password: String,
    database: String,
    warehouse: String,
    role: Option<String>,
}

#[derive(Deserialize)]
struct QueryArgs {
    sql: String,
    params: Option<Vec<String>>,
}

#[derive(Serialize)]
struct QueryResult {
    rows: Vec<serde_json::Value>,
    affected_rows: Option<i64>,
}

impl Extension for SnowflakeExtension {
    fn new(schema_directives: Vec<Directive>, config: Configuration) -> Result<Self, Box<dyn std::error::Error>> {
        let config: SnowflakeConfig = config.deserialize()?;
        info!("Initializing Snowflake extension with account: {}", config.account);

        let manager = SnowflakeManager { config };
        let pool = Pool::builder(manager)
            .max_size(10)
            .build()
            .map_err(|e| Box::new(SnowflakeError::Pool(e)))?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }
}

impl Resolver for SnowflakeExtension {
    fn resolve_field(
        &mut self,
        _context: SharedContext,
        directive: Directive,
        field_definition: FieldDefinition,
        inputs: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        let args: QueryArgs = directive.arguments()?;
        debug!("Executing Snowflake query: {}", args.sql);

        // Get a connection from the pool
        let client = self.pool.get().map_err(|e| Error {
            message: format!("Failed to get connection from pool: {}", e),
            extensions: vec![],
        })?;

        // Replace parameter placeholders with actual values
        let sql = if let Some(params) = args.params {
            let mut sql = args.sql;
            for (i, param) in params.iter().enumerate() {
                // Get the input value for this parameter
                let value = inputs.get(param).ok_or_else(|| Error {
                    message: format!("Missing input value for parameter: {}", param),
                    extensions: vec![],
                })?;
                sql = sql.replace(&format!("?{}", i + 1), &value.to_string());
            }
            sql
        } else {
            args.sql
        };

        // Execute the query
        let result = client.query(&sql).map_err(|e| Error {
            message: format!("Failed to execute query: {}", e),
            extensions: vec![],
        })?;

        // Convert the result to our QueryResult type
        let result = QueryResult {
            rows: result.rows().iter().map(|row| row.to_json()).collect(),
            affected_rows: Some(result.affected_rows() as i64),
        };

        let mut output = FieldOutput::new();
        output.push_value(result);
        Ok(output)
    }
} 