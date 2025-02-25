use grafbase_sdk::{
    Error, Extension, Resolver, ResolverExtension, SharedContext,
    host_io::pubsub::Subscription,
    types::{Configuration, Directive, FieldDefinition, FieldInputs, FieldOutput},
};

#[derive(ResolverExtension)]
struct EchoExtension;

impl Extension for EchoExtension {
    fn new(_schema_directives: Vec<Directive>, _config: Configuration) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }
}

#[derive(serde::Deserialize)]
struct HelloArguments {
    to: String,
}

impl Resolver for EchoExtension {
    fn resolve_field(
        &mut self,
        _context: SharedContext,
        directive: Directive,
        _field_definition: FieldDefinition,
        _inputs: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        let value = match directive.name() {
            "hello" => {
                let args: HelloArguments = directive.arguments().map_err(|err| Error {
                    extensions: Vec::new(),
                    message: err.to_string(),
                })?;
                format!("Hello, {}", args.to)
            }
            other => format!("unknown directive `{other}`"),
        };

        let mut output = FieldOutput::new();

        output.push_value(serde_json::json!(value));

        Ok(output)
    }

    fn resolve_subscription(
        &mut self,
        _: SharedContext,
        _: Directive,
        _: FieldDefinition,
    ) -> Result<Box<dyn Subscription>, Error> {
        todo!()
    }
}
