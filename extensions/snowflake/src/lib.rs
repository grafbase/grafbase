mod auth;
mod config;
mod directives;
mod statements;

use self::config::{Authentication, SnowflakeConfig};
use grafbase_sdk::{
    Error, Headers, ResolverExtension, Subscription,
    types::{Configuration, FieldDefinitionDirective, FieldInputs, FieldOutput, SchemaDirective},
};

#[derive(ResolverExtension)]
struct Snowflake {
    jwt: String,
    config: SnowflakeConfig,
}

impl ResolverExtension for Snowflake {
    fn new(_schema_directives: Vec<SchemaDirective>, config: Configuration) -> Result<Self, Error> {
        let config: SnowflakeConfig = config.deserialize()?;

        Ok(Self {
            jwt: auth::generate_jwt(&config),
            config,
        })
    }

    fn resolve_field(
        &mut self,
        _headers: Headers,
        _subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
        field_inputs: FieldInputs<'_>,
    ) -> Result<FieldOutput, Error> {
        match directive.name() {
            "snowflakeQuery" => {
                let directives::SnowflakeQueryDirective { sql, bindings } = directive.arguments()?;

                let bindings = bindings
                    .map(|binding| {
                        serde_json::from_str(&binding)
                            .map_err(|err| Error::new(format!("Failed to parse bindings: {err}")))
                    })
                    .unwrap_or(Ok(vec![]))?;

                let response = self.execute_statement(&sql, &bindings)?;

                let Some(data) = response.data else {
                    return Err(Error::new(format!(
                        "No data returned from Snowflake query. SQL State: {}, Code: {}. Message: {}",
                        response.sql_state, response.code, response.message
                    )));
                };

                Ok(FieldOutput::new(field_inputs, data)?)
            }
            other => Err(Error::new(format!("Unsupported directive \"{other}\""))),
        }
    }

    fn resolve_subscription(
        &mut self,
        _headers: Headers,
        _subgraph_name: &str,
        _directive: FieldDefinitionDirective<'_>,
    ) -> Result<Box<dyn Subscription>, Error> {
        unreachable!("No subscriptions support in the snowflake extension")
    }
}
