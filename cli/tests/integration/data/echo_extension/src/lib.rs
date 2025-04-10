use grafbase_sdk::{
    FieldResolverExtension, Subscription,
    types::{
        Configuration, Error, FieldDefinitionDirective, FieldInputs, FieldOutputs, SchemaDirective, SubgraphHeaders,
    },
};

#[derive(FieldResolverExtension)]
struct EchoExtension;

#[derive(serde::Deserialize)]
struct HelloArguments {
    to: String,
}

impl FieldResolverExtension for EchoExtension {
    fn new(_schema_directives: Vec<SchemaDirective>, _config: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    fn resolve_field(
        &mut self,
        _headers: SubgraphHeaders,
        _subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
        inputs: FieldInputs,
    ) -> Result<FieldOutputs, Error> {
        let value = match directive.name() {
            "hello" => {
                let args: HelloArguments = directive.arguments().map_err(|err| err.to_string())?;
                format!("Hello, {}", args.to)
            }
            other => format!("unknown directive `{other}`"),
        };

        Ok(FieldOutputs::new(inputs, serde_json::json!(value))?)
    }

    fn resolve_subscription(
        &mut self,
        _: SubgraphHeaders,
        _: &str,
        _: FieldDefinitionDirective<'_>,
    ) -> Result<Box<dyn Subscription>, Error> {
        todo!()
    }
}
