mod config;
mod subscription;
mod types;

use std::{cell::RefCell, collections::HashMap, rc::Rc, str::FromStr, time::Duration};

use config::AuthConfig;
use grafbase_sdk::{
    host_io::pubsub::nats::{self, NatsClient, NatsStreamConfig},
    jq_selection::JqSelection,
    types::{Configuration, FieldDefinitionDirective, FieldInputs, FieldOutput, SchemaDirective},
    Error, Headers, NatsAuth, ResolverExtension, Subscription,
};
use subscription::FilteredSubscription;
use types::{DirectiveKind, KeyValueAction, KeyValueArguments, PublishArguments, RequestArguments, SubscribeArguments};

#[derive(ResolverExtension)]
struct Nats {
    clients: HashMap<String, NatsClient>,
    jq_selection: Rc<RefCell<JqSelection>>,
}

impl ResolverExtension for Nats {
    fn new(_: Vec<SchemaDirective>, config: Configuration) -> Result<Self, Error> {
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

    fn resolve_field(
        &mut self,
        _: Headers,
        _: &str,
        directive: FieldDefinitionDirective,
        inputs: FieldInputs<'_>,
    ) -> Result<FieldOutput, Error> {
        let Ok(directive_kind) = DirectiveKind::from_str(directive.name()) else {
            return Err(format!("Invalid directive: {}", directive.name()).into());
        };

        match directive_kind {
            DirectiveKind::Publish => {
                let args: PublishArguments<'_> = directive
                    .arguments()
                    .map_err(|e| format!("Error deserializing directive arguments: {e}"))?;

                self.publish(args, inputs)
            }
            DirectiveKind::Request => {
                let args: RequestArguments<'_> = directive
                    .arguments()
                    .map_err(|e| format!("Error deserializing directive arguments: {e}"))?;

                self.request(args, inputs)
            }
            DirectiveKind::KeyValue => {
                let args: KeyValueArguments<'_> = directive
                    .arguments()
                    .map_err(|e| format!("Error deserializing directive arguments: {e}"))?;

                self.key_value(args, inputs)
            }
        }
    }

    fn subscription_key(
        &mut self,
        _: Headers,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
    ) -> Option<Vec<u8>> {
        let mut identifier = Vec::new();

        identifier.extend(subgraph_name.as_bytes());
        identifier.extend(directive.name().as_bytes());
        identifier.extend(directive.site().parent_type_name().as_bytes());
        identifier.extend(directive.site().field_name().as_bytes());
        identifier.extend(directive.arguments_bytes());

        Some(identifier)
    }

    fn resolve_subscription(
        &mut self,
        _: Headers,
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
    fn publish(&self, request: PublishArguments<'_>, inputs: FieldInputs<'_>) -> Result<FieldOutput, Error> {
        let Some(client) = self.clients.get(request.provider) else {
            return Err(format!("NATS provider not found: {}", request.provider).into());
        };

        let body = request.body().unwrap_or(&serde_json::Value::Null);

        let result = client.publish(request.subject, body);

        Ok(FieldOutput::new(inputs, result.is_ok())?)
    }

    fn request(&self, request: RequestArguments<'_>, inputs: FieldInputs<'_>) -> Result<FieldOutput, Error> {
        let Some(client) = self.clients.get(request.provider) else {
            return Err(format!("NATS provider not found: {}", request.provider).into());
        };

        let body = request.body().unwrap_or(&serde_json::Value::Null);

        let message = client
            .request::<_, serde_json::Value>(request.subject, body, Some(request.timeout))
            .map_err(|e| format!("Failed to request message: {}", e))?;

        let selection = match request.selection {
            Some(selection) => selection,
            None => return Ok(FieldOutput::new(inputs, message)?),
        };

        let mut jq = self.jq_selection.borrow_mut();

        let filtered = jq
            .select(selection, message)
            .map_err(|e| format!("Failed to filter with selection: {}", e))?
            .collect::<Result<Vec<_>, _>>();

        Ok(match filtered {
            Ok(filtered) => {
                // TODO: We don't whether a list of a single item is expected here... Need engine
                // to help
                if filtered.len() == 1 {
                    FieldOutput::new(inputs, filtered.into_iter().next().unwrap())?
                } else {
                    FieldOutput::new(inputs, filtered)?
                }
            }
            Err(error) => FieldOutput::error(inputs, format!("Failed to filter with selection: {}", error)),
        })
    }

    fn key_value(&self, args: KeyValueArguments<'_>, inputs: FieldInputs<'_>) -> Result<FieldOutput, Error> {
        let Some(client) = self.clients.get(args.provider) else {
            return Err(format!("NATS provider not found: {}", args.provider).into());
        };

        let store = client
            .key_value(args.bucket)
            .map_err(|e| format!("Failed to get key-value store: {e}"))?;

        match args.action {
            KeyValueAction::Create => {
                let body = args.body().unwrap_or(&serde_json::Value::Null);

                match store.create(args.key, body) {
                    Ok(sequence) => Ok(FieldOutput::new(inputs, sequence.to_string())?),
                    Err(error) => Err(format!("Failed to create key-value pair: {error}").into()),
                }
            }
            KeyValueAction::Put => {
                let body = args.body().unwrap_or(&serde_json::Value::Null);

                match store.put(args.key, body) {
                    Ok(sequence) => Ok(FieldOutput::new(inputs, sequence.to_string())?),
                    Err(error) => Err(format!("Failed to put key-value pair: {error}").into()),
                }
            }
            KeyValueAction::Get => {
                let value = match store.get::<serde_json::Value>(args.key) {
                    Ok(Some(value)) => value,
                    Ok(None) => return Ok(FieldOutput::new(inputs, serde_json::Value::Null)?),
                    Err(error) => {
                        return Err(format!("Failed to get key-value pair: {error}").into());
                    }
                };

                let selection = match args.selection {
                    Some(selection) => selection,
                    None => return Ok(FieldOutput::new(inputs, value)?),
                };

                let mut jq = self.jq_selection.borrow_mut();

                let selected = jq
                    .select(selection, value)
                    .map_err(|e| format!("Failed to filter with selection: {}", e))?
                    .collect::<Result<Vec<_>, _>>();

                Ok(match selected {
                    Ok(selected) => {
                        // TODO: We don't whether a list of a single item is expected here... Need engine
                        // to help
                        if selected.len() == 1 {
                            FieldOutput::new(inputs, selected.into_iter().next().unwrap())?
                        } else {
                            FieldOutput::new(inputs, selected)?
                        }
                    }
                    Err(error) => FieldOutput::error(inputs, format!("Failed to filter with selection: {}", error)),
                })
            }
            KeyValueAction::Delete => match store.delete(args.key) {
                Ok(()) => Ok(FieldOutput::new(inputs, true)?),
                Err(error) => Err(format!("Failed to delete key-value pair: {error}").into()),
            },
        }
    }
}
