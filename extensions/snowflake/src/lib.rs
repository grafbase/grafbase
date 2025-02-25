mod types;

use grafbase_sdk::{
    Error, Extension, Resolver, ResolverExtension, SharedContext, Subscription,
    types::{Configuration, Directive, FieldDefinition, FieldInputs, FieldOutput},
};
use types::{Snowflake, SnowflakeConnection};

#[derive(ResolverExtension)]
struct SnowflakeExtension {
    connections: Vec<SnowflakeConnection>,
}

impl Extension for SnowflakeExtension {
    fn new(schema_directives: Vec<Directive>, _: Configuration) -> Result<Self, Box<dyn std::error::Error>> {
        let mut connections = Vec::<SnowflakeConnection>::new();

        for directive in schema_directives {
            let connection = SnowflakeConnection::from_directive(
                directive.subgraph_name().to_string(),
                directive.arguments()?,
            )?;

            connections.push(connection);
        }

        connections.sort_by(|a, b| {
            let by_name = a.args.name.cmp(&b.args.name);
            let by_subgraph = a.subgraph_name.cmp(&b.subgraph_name);
            by_name.then(by_subgraph)
        });

        Ok(Self {
            connections,
        })
    }
}

impl SnowflakeExtension {
    pub fn get_connection(&self, name: &str, subgraph_name: &str) -> Option<&SnowflakeConnection> {
        self.connections.iter().find(|connection| {
            connection.args.name == name && connection.subgraph_name == subgraph_name
        })
    }
}

impl Resolver for SnowflakeExtension {
    fn resolve_field(
        &mut self,
        _context: SharedContext,
        directive: Directive,
        _field: FieldDefinition,
        inputs: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        // Parse the directive arguments
        let args: Snowflake = directive.arguments()?;
        
        // Get the connection from the name
        let connection = self
            .get_connection(&args.connection, directive.subgraph_name())
            .ok_or_else(|| {
                Error::runtime(&format!(
                    "Connection '{}' not found in subgraph '{}'",
                    args.connection,
                    directive.subgraph_name()
                ))
            })?;

        // TODO: Implement actual Snowflake query execution here
        // For now, just return an empty JSON object as placeholder
        
        // Return a placeholder result
        let result = serde_json::json!({
            "success": true,
            "message": "Snowflake extension skeleton",
            "connection": {
                "name": connection.args.name,
                "account": connection.args.account,
                "database": connection.args.database,
                "schema": connection.args.schema,
            },
            "query": args.query,
            "params": args.params,
        });

        Ok(FieldOutput::from_json(result))
    }

    fn resolve_subscription(
        &mut self,
        _: SharedContext,
        _: Directive,
        _: FieldDefinition,
    ) -> Result<Box<dyn Subscription>, Error> {
        // Subscriptions not implemented for Snowflake extension
        Err(Error::runtime("Subscriptions are not supported for Snowflake"))
    }
} 