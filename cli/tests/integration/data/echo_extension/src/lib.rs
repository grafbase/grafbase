use grafbase_sdk::{
    Error, Extension, Resolver, ResolverExtension, SharedContext, Subscription,
    types::{Configuration, FieldDefinitionDirective, FieldInputs, FieldOutput, SchemaDirective},
};

#[derive(ResolverExtension)]
struct EchoExtension;

impl Extension for EchoExtension {
    fn new(
        _schema_directives: Vec<SchemaDirective>,
        _config: Configuration,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
        _: &str,
        directive: FieldDefinitionDirective<'_>,
        _inputs: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        let value = match directive.name() {
            "hello" => {
                let args: HelloArguments = directive.arguments().map_err(|err| err.to_string())?;
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
        _: &str,
        _: FieldDefinitionDirective<'_>,
    ) -> Result<Box<dyn Subscription>, Error> {
        todo!()
    }
}
