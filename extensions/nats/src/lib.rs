mod config;
mod types;

use std::{collections::HashMap, str::FromStr};

use config::AuthConfig;
use grafbase_sdk::{
    host_io::nats::{self, NatsClient},
    types::{Configuration, Directive, FieldDefinition, FieldInputs, FieldOutput},
    Error, Extension, NatsAuth, Resolver, ResolverExtension, SharedContext,
};
use types::{DirectiveKind, NatsPublishResult, PublishArguments};

#[derive(ResolverExtension)]
struct Nats {
    clients: HashMap<String, NatsClient>,
}

impl Extension for Nats {
    fn new(_: Vec<Directive>, config: Configuration) -> Result<Self, Box<dyn std::error::Error>> {
        let mut clients = HashMap::new();

        let config: config::NatsConfig = config.deserialize()?;

        for endpoint in config.endpoints {
            let auth = match endpoint.authentication {
                Some(AuthConfig::UsernamePassword { username, password }) => {
                    Some(NatsAuth::UsernamePassword((username, password)))
                }
                Some(AuthConfig::Token { token }) => Some(NatsAuth::Token(token)),
                Some(AuthConfig::Credentials { credentials }) => Some(NatsAuth::Credentials(credentials)),
                None => None,
            };

            let client = match auth {
                Some(ref auth) => nats::connect_with_auth(endpoint.servers, auth)?,
                None => nats::connect(endpoint.servers)?,
            };

            clients.insert(endpoint.name, client);
        }

        Ok(Self { clients })
    }
}

impl Resolver for Nats {
    fn resolve_field(
        &mut self,
        _: SharedContext,
        directive: Directive,
        _: FieldDefinition,
        _: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        let Ok(directive_kind) = DirectiveKind::from_str(directive.name()) else {
            return Err(Error {
                extensions: Vec::new(),
                message: format!("Invalid directive: {}", directive.name()),
            });
        };

        match directive_kind {
            DirectiveKind::Publish => {
                let args: PublishArguments<'_> = directive.arguments().map_err(|e| Error {
                    extensions: Vec::new(),
                    message: format!("Error deserializing directive arguments: {e}"),
                })?;

                self.publish(args)
            }
        }
    }
}

impl Nats {
    fn publish(&self, request: PublishArguments<'_>) -> Result<FieldOutput, Error> {
        let Some(client) = self.clients.get(request.provider) else {
            return Err(Error {
                extensions: Vec::new(),
                message: format!("NATS provider not found: {}", request.provider),
            });
        };

        let body = request.body().unwrap_or(&serde_json::Value::Null);

        let result = client.publish(request.subject, body).map_err(|e| Error {
            extensions: Vec::new(),
            message: format!("Failed to publish message: {}", e),
        });

        let mut output = FieldOutput::new();

        output.push_value(NatsPublishResult {
            success: result.is_ok(),
        });

        Ok(output)
    }
}
