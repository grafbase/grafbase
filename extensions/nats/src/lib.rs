mod config;
mod subscription;
mod types;

use std::{cell::RefCell, collections::HashMap, rc::Rc, str::FromStr, time::Duration};

use config::AuthConfig;
use grafbase_sdk::{
    Error, Extension, NatsAuth, Resolver, ResolverExtension, SharedContext, Subscription,
    host_io::pubsub::nats::{self, NatsClient, NatsStreamConfig},
    jq_selection::JqSelection,
    types::{Configuration, FieldDefinitionDirective, FieldInputs, FieldOutput, SchemaDirective},
};
use subscription::FilteredSubscription;
use types::{DirectiveKind, KeyValueAction, KeyValueArguments, PublishArguments, RequestArguments, SubscribeArguments};

#[derive(ResolverExtension)]
struct Nats {
    clients: HashMap<String, NatsClient>,
    jq_selection: Rc<RefCell<JqSelection>>,
}

impl Extension for Nats {
    fn new(_: Vec<SchemaDirective>, config: Configuration) -> Result<Self, Box<dyn std::error::Error>> {
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
        _: &str,
        directive: FieldDefinitionDirective,
        _: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        let Ok(directive_kind) = DirectiveKind::from_str(directive.name()) else {
            return Err(format!("Invalid directive: {}", directive.name()).into());
        };

        match directive_kind {
            DirectiveKind::Publish => {
                let args: PublishArguments<'_> = directive
                    .arguments()
                    .map_err(|e| format!("Error deserializing directive arguments: {e}"))?;

                self.publish(args)
            }
            DirectiveKind::Request => {
                let args: RequestArguments<'_> = directive
                    .arguments()
                    .map_err(|e| format!("Error deserializing directive arguments: {e}"))?;

                self.request(args)
            }
            DirectiveKind::KeyValue => {
                let args: KeyValueArguments<'_> = directive
                    .arguments()
                    .map_err(|e| format!("Error deserializing directive arguments: {e}"))?;

                self.key_value(args)
            }
        }
    }

    fn resolve_subscription(
        &mut self,
        _: SharedContext,
        _: &str,
        directive: FieldDefinitionDirective,
    ) -> Result<Box<dyn Subscription>, Error> {
        let args: SubscribeArguments<'_> = directive
            .arguments()
            .map_err(|e| format!("Error deserializing directive arguments: {e}"))?;

        let Some(client) = self.clients.get(args.provider) else {
            return Err(format!("NATS provider not found: {}", args.provider).into());
        };

        let stream_config = args.stream_config.map(|config| {
            let mut stream_config = NatsStreamConfig::new(
                config.stream_name.to_string(),
                config.consumer_name.to_string(),
                config.deliver_policy(),
                Duration::from_millis(config.inactive_threshold_ms),
            );

            if let Some(name) = config.durable_name {
                stream_config = stream_config.with_durable_name(name.to_string());
            }

            if let Some(description) = config.description {
                stream_config = stream_config.with_description(description.to_string());
            }

            stream_config
        });

        let subscriber = client
            .subscribe(args.subject, stream_config)
            .map_err(|e| format!("Failed to subscribe to subject '{}': {e}", args.subject))?;

        Ok(Box::new(FilteredSubscription::new(
            subscriber,
            self.jq_selection.clone(),
            args.selection,
        )))
    }
}

impl Nats {
    fn publish(&self, request: PublishArguments<'_>) -> Result<FieldOutput, Error> {
        let Some(client) = self.clients.get(request.provider) else {
            return Err(format!("NATS provider not found: {}", request.provider).into());
        };

        let body = request.body().unwrap_or(&serde_json::Value::Null);

        let result = client.publish(request.subject, body);

        let mut output = FieldOutput::new();
        output.push_value(result.is_ok());

        Ok(output)
    }

    fn request(&self, request: RequestArguments<'_>) -> Result<FieldOutput, Error> {
        let Some(client) = self.clients.get(request.provider) else {
            return Err(format!("NATS provider not found: {}", request.provider).into());
        };

        let body = request.body().unwrap_or(&serde_json::Value::Null);

        let message = client
            .request::<_, serde_json::Value>(request.subject, body, Some(request.timeout))
            .map_err(|e| format!("Failed to request message: {}", e))?;

        let mut output = FieldOutput::new();

        let selection = match request.selection {
            Some(selection) => selection,
            None => {
                output.push_value(message);
                return Ok(output);
            }
        };

        let mut jq = self.jq_selection.borrow_mut();

        let filtered = jq
            .select(selection, message)
            .map_err(|e| format!("Failed to filter with selection: {}", e))?;

        for payload in filtered {
            match payload {
                Ok(payload) => output.push_value(payload),
                Err(error) => output.push_error(format!("Failed to filter with selection: {}", error)),
            }
        }

        Ok(output)
    }

    fn key_value(&self, args: KeyValueArguments<'_>) -> Result<FieldOutput, Error> {
        let Some(client) = self.clients.get(args.provider) else {
            return Err(format!("NATS provider not found: {}", args.provider).into());
        };

        let store = client
            .key_value(args.bucket)
            .map_err(|e| format!("Failed to get key-value store: {e}"))?;

        let mut output = FieldOutput::new();

        match args.action {
            KeyValueAction::Create => {
                let body = args.body().unwrap_or(&serde_json::Value::Null);

                match store.create(args.key, body) {
                    Ok(sequence) => output.push_value(sequence.to_string()),
                    Err(error) => {
                        return Err(format!("Failed to create key-value pair: {error}").into());
                    }
                }
            }
            KeyValueAction::Put => {
                let body = args.body().unwrap_or(&serde_json::Value::Null);

                match store.put(args.key, body) {
                    Ok(sequence) => output.push_value(sequence.to_string()),
                    Err(error) => {
                        return Err(format!("Failed to put key-value pair: {error}").into());
                    }
                }
            }
            KeyValueAction::Get => {
                let value = match store.get::<serde_json::Value>(args.key) {
                    Ok(Some(value)) => value,
                    Ok(None) => {
                        output.push_value(serde_json::Value::Null);
                        return Ok(output);
                    }
                    Err(error) => {
                        return Err(format!("Failed to get key-value pair: {error}").into());
                    }
                };

                let selection = match args.selection {
                    Some(selection) => selection,
                    None => {
                        output.push_value(value);
                        return Ok(output);
                    }
                };

                let mut jq = self.jq_selection.borrow_mut();

                let selected = jq
                    .select(selection, value)
                    .map_err(|e| format!("Failed to filter with selection: {}", e))?;

                for payload in selected {
                    match payload {
                        Ok(payload) => output.push_value(payload),
                        Err(error) => output.push_error(format!("Failed to filter with selection: {}", error)),
                    }
                }
            }
            KeyValueAction::Delete => match store.delete(args.key) {
                Ok(()) => output.push_value(true),
                Err(error) => return Err(format!("Failed to delete key-value pair: {error}").into()),
            },
        }

        Ok(output)
    }
}
