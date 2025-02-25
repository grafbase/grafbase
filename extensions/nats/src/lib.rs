mod config;
mod subscriber;
mod types;

use std::{cell::RefCell, collections::HashMap, rc::Rc, str::FromStr};

use config::AuthConfig;
use grafbase_sdk::{
    host_io::pubsub::{
        nats::{self, NatsClient},
        Subscriber,
    },
    jq_selection::JqSelection,
    types::{Configuration, Directive, FieldDefinition, FieldInputs, FieldOutput},
    Error, Extension, NatsAuth, Resolver, ResolverExtension, SharedContext,
};
use subscriber::FilteringSubscriber;
use types::{DirectiveKind, NatsPublishResult, PublishArguments, SubscribeArguments};

#[derive(ResolverExtension)]
struct Nats {
    clients: HashMap<String, NatsClient>,
    jq_selection: Rc<RefCell<JqSelection>>,
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

        Ok(Self {
            clients,
            jq_selection: Rc::new(RefCell::new(JqSelection::default())),
        })
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

    fn resolve_subscription(
        &mut self,
        _: SharedContext,
        directive: Directive,
        _: FieldDefinition,
    ) -> Result<Box<dyn Subscriber>, Error> {
        let args: SubscribeArguments<'_> = directive.arguments().map_err(|e| Error {
            extensions: Vec::new(),
            message: format!("Error deserializing directive arguments: {e}"),
        })?;

        let Some(client) = self.clients.get(args.provider) else {
            return Err(Error {
                extensions: Vec::new(),
                message: format!("NATS provider not found: {}", args.provider),
            });
        };

        let subscriber = client.subscribe(args.subject).map_err(|e| Error {
            extensions: Vec::new(),
            message: format!("Failed to subscribe to subject '{}': {e}", args.subject),
        })?;

        Ok(Box::new(FilteringSubscriber::new(
            subscriber,
            self.jq_selection.clone(),
            args.selection,
        )))
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
